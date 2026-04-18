/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::collections::HashMap;

use tree_sitter as ts;
use tree_sitter::StreamingIterator;

use crate::metrics_contract::{
    BlobCacheKey, CodeUnitDescriptor, CodeUnitExecutionKind, FileSnapshot, MetricEvent,
    MetricEventKind, MetricsLanguageAdapterContract, SourceSpan, StableCodeUnitId,
    StructuralQueryAnchor,
};
use crate::tree_sitter_parse::{from_language, to_tree};
use crate::{Language, ParseError};

use super::support::{
    blob_id_for_source, build_qualified_names, code_unit_kind, default_display_name,
    exposure_for_node, is_parent_candidate, span_contains, spec_for, LanguageSpec,
    MetricExtraction, RawUnit,
};

pub fn extract_static_metrics_observations(
    relative_path: &str,
    language: Language,
    source: &str,
) -> Result<MetricExtraction, ParseError> {
    let spec = spec_for(language).ok_or(ParseError::UnsupportedStaticMetricsLanguage(language))?;
    let file = FileSnapshot {
        relative_path: relative_path.into(),
        language,
        blob_id: blob_id_for_source(source),
        byte_count: source.len(),
        line_count: source.lines().count(),
    };
    let tree = to_tree(source, &from_language(language)).ok_or_else(|| {
        ParseError::CustomActionFailed("tree-sitter returned no parse tree".into())
    })?;
    let adapter = StaticMetricsAdapter { spec };
    let units = adapter.extract_code_units(&file, &tree, source)?;
    let events = adapter.emit_metric_events(&file, &tree, source, &units)?;
    let cache_key = BlobCacheKey::for_file(&file, adapter.grammar_version());
    Ok(MetricExtraction {
        file,
        cache_key,
        units,
        events,
    })
}

struct StaticMetricsAdapter {
    spec: LanguageSpec,
}

impl MetricsLanguageAdapterContract for StaticMetricsAdapter {
    fn structural_query_anchors(&self) -> Vec<StructuralQueryAnchor> {
        vec![
            StructuralQueryAnchor {
                node_kind: "function".into(),
                unit_kind: crate::metrics_contract::CodeUnitKind::Function,
                purpose: "function-level executable units".into(),
            },
            StructuralQueryAnchor {
                node_kind: "method".into(),
                unit_kind: crate::metrics_contract::CodeUnitKind::Method,
                purpose: "method-level executable units".into(),
            },
            StructuralQueryAnchor {
                node_kind: "class".into(),
                unit_kind: crate::metrics_contract::CodeUnitKind::Class,
                purpose: "container units for parent-child normalization".into(),
            },
            StructuralQueryAnchor {
                node_kind: "import".into(),
                unit_kind: crate::metrics_contract::CodeUnitKind::Import,
                purpose: "file-level dependency surface".into(),
            },
            StructuralQueryAnchor {
                node_kind: "export".into(),
                unit_kind: crate::metrics_contract::CodeUnitKind::Export,
                purpose: "file-level public API surface".into(),
            },
        ]
    }

    fn extract_code_units(
        &self,
        file: &FileSnapshot,
        tree: &ts::Tree,
        source: &str,
    ) -> Result<Vec<CodeUnitDescriptor>, ParseError> {
        let language = from_language(self.spec.language).language;
        let query = ts::Query::new(&language, self.spec.query)
            .map_err(|err| ParseError::TreeSitterQueryError(err.to_string()))?;
        let mut cursor = ts::QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut raw_units = Vec::new();

        matches.advance();
        while let Some(query_match) = matches.get() {
            let mut unit_capture = None;
            let mut name_capture = None;
            for capture in query_match.captures {
                let capture_name = &query.capture_names()[capture.index as usize];
                if capture_name.starts_with("unit.") {
                    unit_capture = Some((capture_name.to_string(), capture.node));
                } else if *capture_name == "name" {
                    name_capture = Some(capture.node);
                }
            }

            let Some((capture_name, node)) = unit_capture else {
                matches.advance();
                continue;
            };

            let display_name = name_capture
                .or_else(|| node.child_by_field_name("name"))
                .and_then(|name| name.utf8_text(source.as_bytes()).ok())
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| default_display_name(&capture_name, node.kind()));
            let kind = code_unit_kind(self.spec.language, &capture_name, &node);
            let execution_kind = match kind {
                crate::metrics_contract::CodeUnitKind::Function
                | crate::metrics_contract::CodeUnitKind::Method
                | crate::metrics_contract::CodeUnitKind::Constructor
                    if !(self.spec.language == Language::Rust
                        && node.kind() == "function_signature_item") =>
                {
                    CodeUnitExecutionKind::Executable
                }
                crate::metrics_contract::CodeUnitKind::Function
                | crate::metrics_contract::CodeUnitKind::Method
                | crate::metrics_contract::CodeUnitKind::Constructor
                | crate::metrics_contract::CodeUnitKind::JsonDocument => {
                    CodeUnitExecutionKind::NonExecutable
                }
                _ => CodeUnitExecutionKind::Container,
            };
            raw_units.push(RawUnit {
                kind,
                execution_kind,
                exposure: exposure_for_node(
                    self.spec.language,
                    &node,
                    &display_name,
                    source.as_bytes(),
                ),
                display_name,
                parent_index: None,
                span: SourceSpan::from_node(&node),
            });
            matches.advance();
        }

        raw_units.sort_by(|left, right| {
            left.span
                .start
                .byte_offset
                .cmp(&right.span.start.byte_offset)
                .then(right.span.end.byte_offset.cmp(&left.span.end.byte_offset))
        });

        for index in 0..raw_units.len() {
            raw_units[index].parent_index = (0..index)
                .rev()
                .find(|candidate| is_parent_candidate(&raw_units[*candidate].kind))
                .filter(|candidate| {
                    span_contains(&raw_units[*candidate].span, &raw_units[index].span)
                });
        }

        let qualified_names = build_qualified_names(&raw_units);
        let stable_ids: Vec<_> = raw_units
            .iter()
            .zip(qualified_names.iter())
            .map(|(unit, qualified_name)| {
                StableCodeUnitId::from_parts(
                    &file.blob_id,
                    &unit.kind,
                    qualified_name.as_deref(),
                    &unit.span,
                )
            })
            .collect();

        Ok(raw_units
            .into_iter()
            .enumerate()
            .map(|(index, unit)| CodeUnitDescriptor {
                stable_id: stable_ids[index].clone(),
                file_blob_id: file.blob_id.clone(),
                kind: unit.kind,
                execution_kind: unit.execution_kind,
                exposure: unit.exposure,
                display_name: unit.display_name,
                qualified_name: qualified_names[index].clone(),
                parent_stable_id: unit.parent_index.map(|parent| stable_ids[parent].clone()),
                span: unit.span,
            })
            .collect())
    }

    fn emit_metric_events(
        &self,
        _file: &FileSnapshot,
        tree: &ts::Tree,
        source: &str,
        units: &[CodeUnitDescriptor],
    ) -> Result<Vec<MetricEvent>, ParseError> {
        let mut events = Vec::new();
        let mut unit_lookup = HashMap::new();
        let executable_units: Vec<_> = units
            .iter()
            .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
            .collect();

        for unit in units {
            unit_lookup.insert(
                (unit.span.start.byte_offset, unit.span.end.byte_offset),
                (unit.stable_id.clone(), unit.execution_kind.clone()),
            );
            if unit.kind == crate::metrics_contract::CodeUnitKind::Import {
                events.push(MetricEvent {
                    kind: MetricEventKind::Import,
                    stable_unit_id: None,
                    span: unit.span.clone(),
                    detail: None,
                });
            } else if unit.kind == crate::metrics_contract::CodeUnitKind::Export {
                events.push(MetricEvent {
                    kind: MetricEventKind::Export,
                    stable_unit_id: None,
                    span: unit.span.clone(),
                    detail: None,
                });
            }
        }

        emit_comment_events(
            self.spec,
            tree.root_node(),
            source,
            &executable_units,
            &mut events,
        )?;
        walk_tree(
            self.spec,
            tree.root_node(),
            &unit_lookup,
            None,
            0,
            &mut events,
        );
        Ok(events)
    }

    fn grammar_version(&self) -> &'static str {
        self.spec.grammar_version
    }

    fn language(&self) -> Language {
        self.spec.language
    }
}

fn emit_comment_events(
    spec: LanguageSpec,
    root: ts::Node<'_>,
    source: &str,
    executable_units: &[&CodeUnitDescriptor],
    events: &mut Vec<MetricEvent>,
) -> Result<(), ParseError> {
    let language = from_language(spec.language).language;
    let query = ts::Query::new(&language, spec.comment_query)
        .map_err(|err| ParseError::TreeSitterQueryError(err.to_string()))?;
    let mut cursor = ts::QueryCursor::new();
    let mut matches = cursor.matches(&query, root, source.as_bytes());

    matches.advance();
    while let Some(query_match) = matches.get() {
        for capture in query_match.captures {
            let span = SourceSpan::from_node(&capture.node);
            let stable_unit_id = executable_units
                .iter()
                .filter(|unit| span_contains(&unit.span, &span))
                .min_by_key(|unit| unit.span.end.byte_offset - unit.span.start.byte_offset)
                .map(|unit| unit.stable_id.clone());
            events.push(MetricEvent {
                kind: MetricEventKind::Comment,
                stable_unit_id,
                span,
                detail: None,
            });
        }
        matches.advance();
    }
    Ok(())
}

fn walk_tree(
    spec: LanguageSpec,
    node: ts::Node<'_>,
    unit_lookup: &HashMap<(usize, usize), (StableCodeUnitId, CodeUnitExecutionKind)>,
    current_executable: Option<StableCodeUnitId>,
    nesting_depth: usize,
    events: &mut Vec<MetricEvent>,
) {
    let unit = unit_lookup.get(&(node.start_byte(), node.end_byte()));
    let mut current_executable = current_executable;

    if let Some((stable_id, execution_kind)) = unit {
        events.push(MetricEvent {
            kind: MetricEventKind::EnterUnit,
            stable_unit_id: Some(stable_id.clone()),
            span: SourceSpan::from_node(&node),
            detail: None,
        });
        if *execution_kind == CodeUnitExecutionKind::Executable {
            current_executable = Some(stable_id.clone());
            let parameter_count = count_parameters(spec.language, &node);
            if parameter_count > 0 {
                events.push(MetricEvent {
                    kind: MetricEventKind::Parameter,
                    stable_unit_id: current_executable.clone(),
                    span: SourceSpan::from_node(&node),
                    detail: Some(parameter_count.to_string()),
                });
            }
        }
    }

    if let Some(stable_unit_id) = current_executable.clone() {
        let event_kind = if spec.branch_nodes.contains(&node.kind()) {
            Some(MetricEventKind::Branch)
        } else if spec.loop_nodes.contains(&node.kind()) {
            Some(MetricEventKind::Loop)
        } else if spec.return_nodes.contains(&node.kind()) {
            Some(MetricEventKind::Return)
        } else if spec.closure_nodes.contains(&node.kind()) {
            Some(MetricEventKind::Closure)
        } else {
            None
        };
        if let Some(kind) = event_kind {
            events.push(MetricEvent {
                kind,
                stable_unit_id: Some(stable_unit_id),
                span: SourceSpan::from_node(&node),
                detail: Some(nesting_depth.to_string()),
            });
        }
    }

    let next_depth = if current_executable.is_some()
        && (spec.branch_nodes.contains(&node.kind()) || spec.loop_nodes.contains(&node.kind()))
    {
        nesting_depth + 1
    } else {
        nesting_depth
    };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_tree(
            spec,
            child,
            unit_lookup,
            current_executable.clone(),
            next_depth,
            events,
        );
    }

    if let Some((stable_id, _)) = unit {
        events.push(MetricEvent {
            kind: MetricEventKind::ExitUnit,
            stable_unit_id: Some(stable_id.clone()),
            span: SourceSpan::from_node(&node),
            detail: None,
        });
    }
}

fn count_parameters(language: Language, node: &ts::Node<'_>) -> usize {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return usize::from(node.child_by_field_name("parameter").is_some());
    };
    match language {
        Language::Go => {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter_map(|child| match child.kind() {
                    "parameter_declaration" => {
                        Some(child.child_by_field_name("name").map_or(1, |name_list| {
                            let mut names = name_list.walk();
                            name_list.named_children(&mut names).count().max(1)
                        }))
                    }
                    "variadic_parameter_declaration" => Some(1),
                    _ => None,
                })
                .sum()
        }
        _ => {
            let mut cursor = parameters.walk();
            parameters
                .named_children(&mut cursor)
                .filter(|child| {
                    !matches!(child.kind(), "," | ")" | "(")
                        && !child.kind().contains("comment")
                        && !child.kind().contains("type")
                })
                .count()
        }
    }
}
