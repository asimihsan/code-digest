use language_parsers::metrics_contract::MetricValue;
use language_parsers::static_metrics::extract_static_metrics_observations;
use language_parsers::Language;
use output_generator::static_metrics::summarize_static_metrics;

fn integer_metric(metrics: &[language_parsers::metrics_contract::MetricRow], name: &str) -> i64 {
    metrics
        .iter()
        .find(|row| row.metric_name == name)
        .and_then(|row| match row.value {
            MetricValue::Integer(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

fn integer_code_unit_metric(
    metrics: &[language_parsers::metrics_contract::MetricRow],
    stable_id_fragment: &str,
    name: &str,
) -> i64 {
    metrics
        .iter()
        .find(|row| {
            row.metric_name == name
                && matches!(
                    &row.subject,
                    language_parsers::metrics_contract::MetricSubject::CodeUnit { stable_id }
                        if stable_id.0.contains(stable_id_fragment)
                )
        })
        .and_then(|row| match row.value {
            MetricValue::Integer(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

#[test]
fn summarizes_rust_fixture_into_file_rollups_and_unit_metrics() {
    let source = include_str!("../../language_parsers/tests/fixtures/static_metrics/rust/core.rs");
    let extraction = extract_static_metrics_observations(
        "fixtures/static_metrics/rust/core.rs",
        Language::Rust,
        source,
    )
    .unwrap();
    let summary = summarize_static_metrics(&extraction, source);

    let file_metrics: Vec<_> = summary
        .metrics
        .iter()
        .filter(|row| {
            matches!(
                row.subject,
                language_parsers::metrics_contract::MetricSubject::File { .. }
            )
        })
        .cloned()
        .collect();
    assert_eq!(integer_metric(&file_metrics, "function_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "public_api_unit_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "import_count"), 1);
    assert_eq!(integer_metric(&file_metrics, "branch_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "loop_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "return_count"), 1);
}

#[test]
fn summarizes_typescript_fixture_with_exports() {
    let source =
        include_str!("../../language_parsers/tests/fixtures/static_metrics/typescript/service.ts");
    let extraction = extract_static_metrics_observations(
        "fixtures/static_metrics/typescript/service.ts",
        Language::TypeScript,
        source,
    )
    .unwrap();
    let summary = summarize_static_metrics(&extraction, source);

    let file_metrics: Vec<_> = summary
        .metrics
        .iter()
        .filter(|row| {
            matches!(
                row.subject,
                language_parsers::metrics_contract::MetricSubject::File { .. }
            )
        })
        .cloned()
        .collect();
    assert_eq!(integer_metric(&file_metrics, "function_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "public_api_unit_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "import_count"), 1);
    assert_eq!(integer_metric(&file_metrics, "export_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "branch_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "loop_count"), 1);
    assert_eq!(integer_metric(&file_metrics, "return_count"), 4);
}

#[test]
fn logical_code_lines_keep_inline_comment_lines() {
    let source = r#"pub fn sample() { // inline comment
    let x = 1; /* trailing block */
    /* comment only */
    let y = 2;
}
"#;
    let extraction =
        extract_static_metrics_observations("inline_comments.rs", Language::Rust, source).unwrap();
    let summary = summarize_static_metrics(&extraction, source);

    let file_metrics: Vec<_> = summary
        .metrics
        .iter()
        .filter(|row| {
            matches!(
                row.subject,
                language_parsers::metrics_contract::MetricSubject::File { .. }
            )
        })
        .cloned()
        .collect();

    assert_eq!(integer_metric(&file_metrics, "source_line_count"), 5);
    assert_eq!(integer_metric(&file_metrics, "comment_line_count"), 3);
    assert_eq!(integer_metric(&file_metrics, "logical_code_line_count"), 4);
}

#[test]
fn summary_counts_exported_arrow_functions_as_units_with_parameters() {
    let source = r#"export const render = flag => {
  if (flag) {
    return 1;
  }
  return 0;
};
"#;
    let extraction =
        extract_static_metrics_observations("render.js", Language::JavaScript, source).unwrap();
    let summary = summarize_static_metrics(&extraction, source);

    let file_metrics: Vec<_> = summary
        .metrics
        .iter()
        .filter(|row| {
            matches!(
                row.subject,
                language_parsers::metrics_contract::MetricSubject::File { .. }
            )
        })
        .cloned()
        .collect();

    assert_eq!(integer_metric(&file_metrics, "function_count"), 1);
    assert_eq!(integer_metric(&file_metrics, "public_api_unit_count"), 1);
    assert_eq!(
        integer_code_unit_metric(&summary.metrics, "render", "parameter_count"),
        1
    );
}

#[test]
fn summary_counts_commonjs_exported_functions_as_public_units() {
    let source = r#"module.exports = function render(value) {
  return value;
};

exports.helper = item => item;
"#;
    let extraction =
        extract_static_metrics_observations("commonjs.js", Language::JavaScript, source).unwrap();
    let summary = summarize_static_metrics(&extraction, source);

    let file_metrics: Vec<_> = summary
        .metrics
        .iter()
        .filter(|row| {
            matches!(
                row.subject,
                language_parsers::metrics_contract::MetricSubject::File { .. }
            )
        })
        .cloned()
        .collect();

    assert_eq!(integer_metric(&file_metrics, "function_count"), 2);
    assert_eq!(integer_metric(&file_metrics, "public_api_unit_count"), 2);
}
