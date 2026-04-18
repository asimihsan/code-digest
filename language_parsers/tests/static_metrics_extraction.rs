use language_parsers::metrics_contract::{CodeUnitExecutionKind, CodeUnitKind, MetricEventKind};
use language_parsers::static_metrics::{
    extract_static_metrics_observations, supports_static_metrics,
    FIRST_SUPPORTED_STATIC_METRICS_LANGUAGES,
};
use language_parsers::Language;

#[test]
fn extracts_units_and_events_for_rust_fixture() {
    let extraction = extract_static_metrics_observations(
        "fixtures/static_metrics/rust/core.rs",
        Language::Rust,
        include_str!("fixtures/static_metrics/rust/core.rs"),
    )
    .unwrap();

    assert_eq!(extraction.file.language, Language::Rust);
    assert_eq!(extraction.cache_key.language, Language::Rust);
    assert!(extraction
        .units
        .iter()
        .any(|unit| unit.kind == CodeUnitKind::Import));
    assert_eq!(
        extraction
            .units
            .iter()
            .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
            .count(),
        2
    );
    assert!(extraction
        .events
        .iter()
        .any(|event| event.kind == MetricEventKind::Branch));
    assert!(extraction
        .events
        .iter()
        .any(|event| event.kind == MetricEventKind::Loop));
}

#[test]
fn supports_first_language_slice_and_rejects_deferred_languages() {
    for supported in FIRST_SUPPORTED_STATIC_METRICS_LANGUAGES {
        assert!(supports_static_metrics(supported.language));
    }
    assert!(!supports_static_metrics(Language::Python));
    assert!(!supports_static_metrics(Language::Java));
    assert!(!supports_static_metrics(Language::Hcl));
}

#[test]
fn extracts_non_executable_json_document_rollup() {
    let extraction = extract_static_metrics_observations(
        "fixtures/static_metrics/json/config.json",
        Language::Json,
        include_str!("fixtures/static_metrics/json/config.json"),
    )
    .unwrap();

    assert_eq!(extraction.units.len(), 1);
    assert_eq!(extraction.units[0].kind, CodeUnitKind::JsonDocument);
    assert!(extraction
        .events
        .iter()
        .any(|event| event.kind == MetricEventKind::Comment));
}

#[test]
fn extracts_smoke_metrics_for_go_javascript_and_tsx() {
    let cases = vec![
        (
            Language::Go,
            "sample.go",
            r#"package sample

import "fmt"

func Render(flag bool) int {
    if flag {
        return 1
    }
    return 0
}
"#,
        ),
        (
            Language::JavaScript,
            "module.js",
            r#"import fs from "fs";

export function render(flag) {
  if (flag) {
    return 1;
  }
  return 0;
}
"#,
        ),
        (
            Language::Tsx,
            "component.tsx",
            r#"export function Component({ ready }: { ready: boolean }) {
  return <div>{ready ? "ready" : "waiting"}</div>;
}
"#,
        ),
    ];

    for (language, path, source) in cases {
        let extraction = extract_static_metrics_observations(path, language, source).unwrap();
        assert!(
            !extraction.units.is_empty(),
            "{language:?} should produce units"
        );
    }
}

#[test]
fn rust_trait_and_extern_signatures_stay_non_executable() {
    let source = r#"pub trait Service {
    fn render(&self, flag: bool) -> usize;
}

extern "C" {
    pub fn foreign_render(input: *const u8);
}
"#;

    let extraction =
        extract_static_metrics_observations("signatures.rs", Language::Rust, source).unwrap();

    let signature_units: Vec<_> = extraction
        .units
        .iter()
        .filter(|unit| unit.display_name == "render" || unit.display_name == "foreign_render")
        .collect();

    assert_eq!(signature_units.len(), 2);
    assert!(signature_units
        .iter()
        .all(|unit| unit.execution_kind == CodeUnitExecutionKind::NonExecutable));
    assert_eq!(
        extraction
            .units
            .iter()
            .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
            .count(),
        0
    );
}

#[test]
fn javascript_arrow_and_function_expression_units_are_executable() {
    let source = r#"export const render = flag => {
  if (flag) {
    return 1;
  }
  return 0;
};

const helper = function helper(value) {
  return value;
};

assignLater = item => item;
"#;

    let extraction =
        extract_static_metrics_observations("component.js", Language::JavaScript, source).unwrap();

    let executable_units: Vec<_> = extraction
        .units
        .iter()
        .filter(|unit| unit.execution_kind == CodeUnitExecutionKind::Executable)
        .collect();

    assert_eq!(executable_units.len(), 3);
    assert!(executable_units.iter().any(|unit| {
        unit.display_name == "render"
            && unit.exposure == language_parsers::metrics_contract::CodeUnitExposure::PublicApi
    }));
    assert!(executable_units
        .iter()
        .any(|unit| unit.display_name == "helper"));
    assert!(executable_units
        .iter()
        .any(|unit| unit.display_name == "assignLater"));
    assert!(extraction.events.iter().any(|event| {
        event.kind == MetricEventKind::Parameter
            && event.detail.as_deref() == Some("1")
            && event
                .stable_unit_id
                .as_ref()
                .is_some_and(|stable_id| stable_id.0.contains("render"))
    }));
}

#[test]
fn tsx_exported_component_arrow_function_is_public_executable_unit() {
    let source = r#"export const Component = ({ ready }: { ready: boolean }) => {
  return <div>{ready ? "ready" : "waiting"}</div>;
};
"#;

    let extraction =
        extract_static_metrics_observations("component.tsx", Language::Tsx, source).unwrap();

    let component = extraction
        .units
        .iter()
        .find(|unit| unit.display_name == "Component")
        .unwrap();

    assert_eq!(component.execution_kind, CodeUnitExecutionKind::Executable);
    assert_eq!(
        component.exposure,
        language_parsers::metrics_contract::CodeUnitExposure::PublicApi
    );
}

#[test]
fn javascript_commonjs_export_assignments_are_public_executable_units() {
    let source = r#"module.exports = function render(value) {
  return value;
};

exports.helper = item => item;
module.exports.worker = function worker(flag) {
  return flag;
};
"#;

    let extraction =
        extract_static_metrics_observations("commonjs.js", Language::JavaScript, source).unwrap();

    let public_units: Vec<_> = extraction
        .units
        .iter()
        .filter(|unit| {
            unit.execution_kind == CodeUnitExecutionKind::Executable
                && unit.exposure == language_parsers::metrics_contract::CodeUnitExposure::PublicApi
        })
        .collect();

    assert_eq!(public_units.len(), 3);
    assert!(public_units
        .iter()
        .any(|unit| unit.display_name == "module.exports"));
    assert!(public_units
        .iter()
        .any(|unit| unit.display_name == "exports.helper"));
    assert!(public_units
        .iter()
        .any(|unit| unit.display_name == "module.exports.worker"));
}
