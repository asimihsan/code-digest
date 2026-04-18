/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::collections::{BTreeSet, HashMap};

use language_parsers::metrics_contract::{
    BlobSummary, CodeUnitExecutionKind, MetricEventKind, MetricRow, MetricSubject, MetricValue,
};
use language_parsers::static_metrics::MetricExtraction;

#[derive(Clone, Default)]
struct UnitAccumulator {
    parameter_count: i64,
    branch_count: i64,
    loop_count: i64,
    return_count: i64,
    closure_count: i64,
    comment_lines: BTreeSet<usize>,
    cognitive_complexity_proxy: i64,
    cyclomatic_complexity: i64,
}

struct FileMetricRollup {
    source_line_count: i64,
    logical_code_line_count: i64,
    comment_line_count: i64,
    executable_unit_count: i64,
    public_api_unit_count: i64,
    import_count: i64,
    export_count: i64,
    branch_count: i64,
    loop_count: i64,
    return_count: i64,
    closure_count: i64,
}

pub fn summarize_static_metrics(extraction: &MetricExtraction, source: &str) -> BlobSummary {
    let mut rows = Vec::new();
    let mut file_comment_lines = BTreeSet::new();
    let mut file_branch_count = 0_i64;
    let mut file_loop_count = 0_i64;
    let mut file_return_count = 0_i64;
    let mut file_closure_count = 0_i64;
    let mut file_import_count = 0_i64;
    let mut file_export_count = 0_i64;
    let mut units_by_id: HashMap<_, _> = extraction
        .units
        .iter()
        .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
        .map(|unit| {
            (
                unit.stable_id.clone(),
                UnitAccumulator {
                    cyclomatic_complexity: 1,
                    ..Default::default()
                },
            )
        })
        .collect();

    for event in &extraction.events {
        let lines = event.span.start.line..=event.span.end.line;
        match event.kind {
            MetricEventKind::Comment => {
                for line in lines.clone() {
                    file_comment_lines.insert(line);
                }
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        for line in lines {
                            unit.comment_lines.insert(line);
                        }
                    }
                }
            }
            MetricEventKind::Branch => {
                file_branch_count += 1;
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        unit.branch_count += 1;
                        unit.cyclomatic_complexity += 1;
                        unit.cognitive_complexity_proxy += 1 + event_depth(event);
                    }
                }
            }
            MetricEventKind::Loop => {
                file_loop_count += 1;
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        unit.loop_count += 1;
                        unit.cyclomatic_complexity += 1;
                        unit.cognitive_complexity_proxy += 1 + event_depth(event);
                    }
                }
            }
            MetricEventKind::Return => {
                file_return_count += 1;
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        unit.return_count += 1;
                    }
                }
            }
            MetricEventKind::Closure => {
                file_closure_count += 1;
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        unit.closure_count += 1;
                    }
                }
            }
            MetricEventKind::Parameter => {
                if let Some(stable_id) = &event.stable_unit_id {
                    if let Some(unit) = units_by_id.get_mut(stable_id) {
                        unit.parameter_count += event
                            .detail
                            .as_deref()
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(1);
                    }
                }
            }
            MetricEventKind::Import => file_import_count += 1,
            MetricEventKind::Export => file_export_count += 1,
            MetricEventKind::EnterUnit
            | MetricEventKind::ExitUnit
            | MetricEventKind::ParseErrorRecovery => {}
        }
    }

    let source_line_count = source.lines().count() as i64;
    let logical_code_line_count = logical_code_line_count(source, &extraction.events);
    let comment_line_count = file_comment_lines.len() as i64;
    let executable_unit_count = extraction
        .units
        .iter()
        .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
        .count() as i64;
    let public_api_unit_count = extraction
        .units
        .iter()
        .filter(|unit| {
            unit.exposure == language_parsers::metrics_contract::CodeUnitExposure::PublicApi
        })
        .count() as i64;

    let file_rollup = FileMetricRollup {
        source_line_count,
        logical_code_line_count,
        comment_line_count,
        executable_unit_count,
        public_api_unit_count,
        import_count: file_import_count,
        export_count: file_export_count,
        branch_count: file_branch_count,
        loop_count: file_loop_count,
        return_count: file_return_count,
        closure_count: file_closure_count,
    };

    rows.extend(file_metric_rows(
        &extraction.file.blob_id,
        &extraction.file.relative_path,
        &file_rollup,
    ));

    for unit in extraction
        .units
        .iter()
        .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
    {
        let stats = units_by_id
            .get(&unit.stable_id)
            .cloned()
            .unwrap_or_default();
        let length_lines = (unit.span.end.line.saturating_sub(unit.span.start.line) + 1) as i64;
        rows.push(integer_metric(
            "function_length_lines",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            length_lines,
            Some("lines"),
            Some("Inclusive source span for the executable unit."),
        ));
        rows.push(integer_metric(
            "parameter_count",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.parameter_count,
            Some("parameters"),
            Some("Syntactic parameter count from the unit signature."),
        ));
        rows.push(integer_metric(
            "branch_count",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.branch_count,
            Some("branches"),
            Some("Decision-point count from syntax-only branch anchors."),
        ));
        rows.push(integer_metric(
            "loop_count",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.loop_count,
            Some("loops"),
            Some("Loop-node count from syntax-only anchors."),
        ));
        rows.push(integer_metric(
            "return_count",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.return_count,
            Some("returns"),
            Some("Explicit return statements only."),
        ));
        rows.push(integer_metric(
            "cyclomatic_complexity",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.cyclomatic_complexity.max(1),
            Some("score"),
            Some("Approximate cyclomatic complexity from branch and loop anchors."),
        ));
        rows.push(integer_metric(
            "cognitive_complexity_proxy",
            MetricSubject::CodeUnit {
                stable_id: unit.stable_id.clone(),
            },
            stats.cognitive_complexity_proxy,
            Some("score"),
            Some("Nesting-weighted proxy from syntax-only branch and loop anchors."),
        ));
    }

    BlobSummary {
        file: extraction.file.clone(),
        cache_key: extraction.cache_key.clone(),
        units: extraction.units.clone(),
        metrics: rows,
    }
}

fn event_depth(event: &language_parsers::metrics_contract::MetricEvent) -> i64 {
    event
        .detail
        .as_deref()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0)
}

fn logical_code_line_count(
    source: &str,
    events: &[language_parsers::metrics_contract::MetricEvent],
) -> i64 {
    let mut uncommented = source.as_bytes().to_vec();
    for event in events {
        if event.kind != MetricEventKind::Comment {
            continue;
        }
        let start = event.span.start.byte_offset.min(uncommented.len());
        let end = event.span.end.byte_offset.min(uncommented.len());
        for byte in &mut uncommented[start..end] {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
    }

    String::from_utf8(uncommented)
        .expect("comment-masked source should remain valid UTF-8")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as i64
}

fn file_metric_rows(
    blob_id: &str,
    relative_path: &str,
    rollup: &FileMetricRollup,
) -> Vec<MetricRow> {
    let subject = MetricSubject::File {
        blob_id: blob_id.into(),
        relative_path: relative_path.into(),
    };
    let comment_density = if rollup.source_line_count == 0 {
        0.0
    } else {
        rollup.comment_line_count as f64 / rollup.source_line_count as f64
    };
    vec![
        integer_metric(
            "source_line_count",
            subject.clone(),
            rollup.source_line_count,
            Some("lines"),
            None,
        ),
        integer_metric(
            "logical_code_line_count",
            subject.clone(),
            rollup.logical_code_line_count,
            Some("lines"),
            Some("Non-empty lines excluding comment-only lines."),
        ),
        integer_metric(
            "comment_line_count",
            subject.clone(),
            rollup.comment_line_count,
            Some("lines"),
            Some("Lines touched by Tree-sitter comment anchors."),
        ),
        MetricRow {
            metric_name: "comment_density".into(),
            subject: subject.clone(),
            value: MetricValue::Float(comment_density),
            unit: Some("ratio".into()),
            description: Some("Comment lines divided by total source lines.".into()),
        },
        integer_metric(
            "function_count",
            subject.clone(),
            rollup.executable_unit_count,
            Some("units"),
            None,
        ),
        integer_metric(
            "public_api_unit_count",
            subject.clone(),
            rollup.public_api_unit_count,
            Some("units"),
            Some("Units marked public by syntax-only exposure heuristics."),
        ),
        integer_metric(
            "import_count",
            subject.clone(),
            rollup.import_count,
            Some("imports"),
            None,
        ),
        integer_metric(
            "export_count",
            subject.clone(),
            rollup.export_count,
            Some("exports"),
            None,
        ),
        integer_metric(
            "branch_count",
            subject.clone(),
            rollup.branch_count,
            Some("branches"),
            None,
        ),
        integer_metric(
            "loop_count",
            subject.clone(),
            rollup.loop_count,
            Some("loops"),
            None,
        ),
        integer_metric(
            "return_count",
            subject.clone(),
            rollup.return_count,
            Some("returns"),
            None,
        ),
        integer_metric(
            "closure_count",
            subject,
            rollup.closure_count,
            Some("closures"),
            None,
        ),
    ]
}

fn integer_metric(
    metric_name: &str,
    subject: MetricSubject,
    value: i64,
    unit: Option<&str>,
    description: Option<&str>,
) -> MetricRow {
    MetricRow {
        metric_name: metric_name.into(),
        subject,
        value: MetricValue::Integer(value),
        unit: unit.map(str::to_string),
        description: description.map(str::to_string),
    }
}
