/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use sha1::{Digest, Sha1};
use tree_sitter as ts;

use crate::metrics_contract::{
    BlobCacheKey, CodeUnitExecutionKind, CodeUnitExposure, CodeUnitKind, FileSnapshot, SourceSpan,
};
use crate::Language;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticMetricsLanguageSupport {
    pub language: Language,
    pub caveats: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricExtraction {
    pub file: FileSnapshot,
    pub cache_key: BlobCacheKey,
    pub units: Vec<crate::metrics_contract::CodeUnitDescriptor>,
    pub events: Vec<crate::metrics_contract::MetricEvent>,
}

#[derive(Clone, Debug)]
pub(crate) struct RawUnit {
    pub(crate) kind: CodeUnitKind,
    pub(crate) execution_kind: CodeUnitExecutionKind,
    pub(crate) exposure: CodeUnitExposure,
    pub(crate) display_name: String,
    pub(crate) parent_index: Option<usize>,
    pub(crate) span: SourceSpan,
}

#[derive(Clone, Copy)]
pub(crate) struct LanguageSpec {
    pub(crate) language: Language,
    pub(crate) grammar_version: &'static str,
    pub(crate) query: &'static str,
    pub(crate) comment_query: &'static str,
    pub(crate) branch_nodes: &'static [&'static str],
    pub(crate) loop_nodes: &'static [&'static str],
    pub(crate) return_nodes: &'static [&'static str],
    pub(crate) closure_nodes: &'static [&'static str],
}

pub const FIRST_SUPPORTED_STATIC_METRICS_LANGUAGES: &[StaticMetricsLanguageSupport] = &[
    StaticMetricsLanguageSupport {
        language: Language::Go,
        caveats: &[
            "Cyclomatic and cognitive counts are syntax-only approximations.",
            "Export detection uses Go naming conventions rather than package typing.",
        ],
    },
    StaticMetricsLanguageSupport {
        language: Language::Rust,
        caveats: &[
            "Methods are detected from impl blocks, but trait and extern declarations stay non-executable in this slice.",
            "Cyclomatic and cognitive counts ignore semantic match exhaustiveness.",
        ],
    },
    StaticMetricsLanguageSupport {
        language: Language::JavaScript,
        caveats: &[
            "Public API detection only treats explicit export wrappers as public.",
            "Named variable-assigned arrow/function expressions are normalized as executable units, but anonymous export-default functions still remain coarse in this slice.",
        ],
    },
    StaticMetricsLanguageSupport {
        language: Language::TypeScript,
        caveats: &[
            "Type-aware coupling and symbol analysis are intentionally excluded.",
            "Public API detection only treats explicit export wrappers as public.",
        ],
    },
    StaticMetricsLanguageSupport {
        language: Language::Tsx,
        caveats: &[
            "JSX/TSX nodes participate in file rollups, but UI semantics are intentionally ignored.",
            "Public API detection only treats explicit export wrappers as public.",
        ],
    },
    StaticMetricsLanguageSupport {
        language: Language::Json,
        caveats: &[
            "JSON only exposes document-level rollups; executable-unit metrics are not applicable.",
            "Comment support depends on the permissive tree-sitter-json grammar, not strict JSON semantics.",
        ],
    },
];

pub fn supports_static_metrics(language: Language) -> bool {
    spec_for(language).is_some()
}

pub(crate) fn blob_id_for_source(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", bytes.len()).as_bytes());
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn build_qualified_names(raw_units: &[RawUnit]) -> Vec<Option<String>> {
    let mut qualified = vec![None; raw_units.len()];
    for (index, unit) in raw_units.iter().enumerate() {
        let Some(name) = (!unit.display_name.is_empty()).then_some(unit.display_name.clone())
        else {
            continue;
        };
        qualified[index] = Some(
            match unit
                .parent_index
                .and_then(|parent| qualified[parent].clone())
            {
                Some(parent) => format!("{parent}::{name}"),
                None => name,
            },
        );
    }
    qualified
}

pub(crate) fn code_unit_kind(
    language: Language,
    capture_name: &str,
    node: &ts::Node<'_>,
) -> CodeUnitKind {
    match capture_name {
        "unit.class" => CodeUnitKind::Class,
        "unit.import" => CodeUnitKind::Import,
        "unit.export" => CodeUnitKind::Export,
        "unit.document" => CodeUnitKind::JsonDocument,
        "unit.method" => CodeUnitKind::Method,
        "unit.function" if language == Language::Rust && has_ancestor(node, "impl_item") => {
            CodeUnitKind::Method
        }
        _ => CodeUnitKind::Function,
    }
}

pub(crate) fn exposure_for_node(
    language: Language,
    node: &ts::Node<'_>,
    display_name: &str,
    source: &[u8],
) -> CodeUnitExposure {
    match language {
        Language::Rust => node
            .child_by_field_name("visibility")
            .or_else(|| child_with_kind(node, "visibility_modifier"))
            .map(|_| CodeUnitExposure::PublicApi)
            .unwrap_or(CodeUnitExposure::Internal),
        Language::Go => display_name
            .chars()
            .next()
            .filter(|ch| ch.is_uppercase())
            .map(|_| CodeUnitExposure::PublicApi)
            .unwrap_or(CodeUnitExposure::Internal),
        Language::JavaScript | Language::TypeScript | Language::Tsx => {
            if is_direct_js_exported_declaration(node)
                || is_commonjs_export_assignment(node, source)
            {
                CodeUnitExposure::PublicApi
            } else {
                CodeUnitExposure::Internal
            }
        }
        _ => CodeUnitExposure::Internal,
    }
}

pub(crate) fn default_display_name(capture_name: &str, fallback_kind: &str) -> String {
    capture_name
        .strip_prefix("unit.")
        .unwrap_or(fallback_kind)
        .replace('.', "-")
}

pub(crate) fn is_parent_candidate(kind: &CodeUnitKind) -> bool {
    matches!(
        kind,
        CodeUnitKind::Class | CodeUnitKind::Struct | CodeUnitKind::Enum | CodeUnitKind::Interface
    )
}

pub(crate) fn span_contains(parent: &SourceSpan, child: &SourceSpan) -> bool {
    parent.start.byte_offset <= child.start.byte_offset
        && parent.end.byte_offset >= child.end.byte_offset
}

pub(crate) fn spec_for(language: Language) -> Option<LanguageSpec> {
    Some(match language {
        Language::Go => LanguageSpec {
            language,
            grammar_version: "tree-sitter-go@workspace-vendored",
            query: r#"
                (import_declaration) @unit.import
                (function_declaration name: (identifier) @name) @unit.function
                (method_declaration name: (_) @name) @unit.method
            "#,
            comment_query: "(comment) @comment",
            branch_nodes: &[
                "if_statement",
                "expression_case",
                "default_case",
                "communication_case",
            ],
            loop_nodes: &["for_statement"],
            return_nodes: &["return_statement"],
            closure_nodes: &["func_literal"],
        },
        Language::Rust => LanguageSpec {
            language,
            grammar_version: "tree-sitter-rust@workspace-vendored",
            query: r#"
                (use_declaration) @unit.import
                (function_item name: (identifier) @name) @unit.function
                (function_signature_item name: (identifier) @name) @unit.function
            "#,
            comment_query: "[(line_comment) (block_comment)] @comment",
            branch_nodes: &["if_expression", "match_expression"],
            loop_nodes: &["for_expression", "while_expression", "loop_expression"],
            return_nodes: &["return_expression"],
            closure_nodes: &["closure_expression"],
        },
        Language::JavaScript | Language::TypeScript | Language::Tsx => LanguageSpec {
            language,
            grammar_version: match language {
                Language::JavaScript => "tree-sitter-javascript@workspace-vendored",
                Language::TypeScript => "tree-sitter-typescript@workspace-vendored",
                Language::Tsx => "tree-sitter-tsx@workspace-vendored",
                _ => unreachable!(),
            },
            query: r#"
                (import_statement) @unit.import
                (export_statement) @unit.export
                (class_declaration) @unit.class
                (function_declaration name: (identifier) @name) @unit.function
                (generator_function_declaration name: (identifier) @name) @unit.function
                (method_definition name: (_) @name) @unit.method
                (variable_declarator
                  name: (identifier) @name
                  value: [
                    (arrow_function)
                    (function_expression)
                    (generator_function)
                  ] @unit.function)
                (assignment_expression
                  left: [
                    (identifier)
                    (member_expression)
                  ] @name
                  right: [
                    (arrow_function)
                    (function_expression)
                    (generator_function)
                  ] @unit.function)
            "#,
            comment_query: "[(comment) (html_comment)] @comment",
            branch_nodes: &[
                "if_statement",
                "switch_case",
                "catch_clause",
                "conditional_expression",
            ],
            loop_nodes: &[
                "for_statement",
                "for_in_statement",
                "while_statement",
                "do_statement",
                "for_of_statement",
            ],
            return_nodes: &["return_statement"],
            closure_nodes: &[
                "arrow_function",
                "function_expression",
                "generator_function",
            ],
        },
        Language::Json => LanguageSpec {
            language,
            grammar_version: "tree-sitter-json@workspace-vendored",
            query: "(document) @unit.document",
            comment_query: "(comment) @comment",
            branch_nodes: &[],
            loop_nodes: &[],
            return_nodes: &[],
            closure_nodes: &[],
        },
        Language::Hcl | Language::Java | Language::Python => return None,
    })
}

fn child_with_kind<'a>(node: &ts::Node<'a>, expected_kind: &str) -> Option<ts::Node<'a>> {
    let mut cursor = node.walk();
    let child = node
        .children(&mut cursor)
        .find(|child| child.kind() == expected_kind);
    child
}

fn is_direct_js_exported_declaration(node: &ts::Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "variable_declarator" | "lexical_declaration" | "variable_declaration" => {
                current = parent.parent();
            }
            "export_statement" => return true,
            _ => return false,
        }
    }
    false
}

fn is_commonjs_export_assignment(node: &ts::Node<'_>, source: &[u8]) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "assignment_expression" {
            let Some(left) = parent.child_by_field_name("left") else {
                return false;
            };
            let Ok(left_text) = left.utf8_text(source) else {
                return false;
            };
            let left_text = left_text.trim();
            return left_text == "module.exports"
                || left_text.starts_with("module.exports.")
                || left_text.starts_with("exports.");
        }
        current = parent.parent();
    }
    false
}

fn has_ancestor(node: &ts::Node<'_>, kind: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == kind {
            return true;
        }
        current = parent.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_language_set_is_explicit_and_excludes_semantic_languages() {
        let supported: Vec<_> = FIRST_SUPPORTED_STATIC_METRICS_LANGUAGES
            .iter()
            .map(|entry| entry.language)
            .collect();

        assert_eq!(
            supported,
            vec![
                Language::Go,
                Language::Rust,
                Language::JavaScript,
                Language::TypeScript,
                Language::Tsx,
                Language::Json
            ]
        );
        assert!(!supports_static_metrics(Language::Python));
        assert!(!supports_static_metrics(Language::Java));
        assert!(!supports_static_metrics(Language::Hcl));
    }

    #[test]
    fn blob_ids_match_git_blob_hashes() {
        assert_eq!(
            blob_id_for_source(""),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        assert_eq!(
            blob_id_for_source("hello world\n"),
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad"
        );
    }
}
