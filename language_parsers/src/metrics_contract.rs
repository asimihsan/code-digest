/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::path::Path;

use serde::{Deserialize, Serialize};
use tree_sitter as ts;

use crate::{Language, ParseError};

pub const METRICS_REPORT_FORMAT_VERSION: u32 = 1;
pub const METRIC_SCHEMA_VERSION: u32 = 1;
pub const NORMALIZATION_RULE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    /// Zero-based line number to match Tree-sitter coordinates.
    pub line: usize,
    /// Zero-based column number to match Tree-sitter coordinates.
    pub column: usize,
    pub byte_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceSpan {
    pub fn from_node(node: &ts::Node<'_>) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            start: SourcePosition {
                line: start.row,
                column: start.column,
                byte_offset: node.start_byte(),
            },
            end: SourcePosition {
                line: end.row,
                column: end.column,
                byte_offset: node.end_byte(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeUnitKind {
    Module,
    Namespace,
    Import,
    Export,
    Struct,
    Enum,
    Interface,
    TypeAlias,
    Class,
    Function,
    Method,
    Constructor,
    Lambda,
    Closure,
    Constant,
    Field,
    Property,
    JsonDocument,
}

impl CodeUnitKind {
    pub fn slug(&self) -> &'static str {
        match self {
            CodeUnitKind::Module => "module",
            CodeUnitKind::Namespace => "namespace",
            CodeUnitKind::Import => "import",
            CodeUnitKind::Export => "export",
            CodeUnitKind::Struct => "struct",
            CodeUnitKind::Enum => "enum",
            CodeUnitKind::Interface => "interface",
            CodeUnitKind::TypeAlias => "type-alias",
            CodeUnitKind::Class => "class",
            CodeUnitKind::Function => "function",
            CodeUnitKind::Method => "method",
            CodeUnitKind::Constructor => "constructor",
            CodeUnitKind::Lambda => "lambda",
            CodeUnitKind::Closure => "closure",
            CodeUnitKind::Constant => "constant",
            CodeUnitKind::Field => "field",
            CodeUnitKind::Property => "property",
            CodeUnitKind::JsonDocument => "json-document",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeUnitExecutionKind {
    Container,
    Executable,
    NonExecutable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeUnitExposure {
    Internal,
    PublicApi,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StableCodeUnitId(pub String);

impl StableCodeUnitId {
    pub fn from_parts(
        blob_id: &str,
        kind: &CodeUnitKind,
        qualified_name: Option<&str>,
        span: &SourceSpan,
    ) -> Self {
        let qualified_name = sanitize_key_segment(qualified_name.unwrap_or("anonymous"));
        Self(format!(
            "{}::{}::{}@{}:{}-{}:{}",
            sanitize_key_segment(blob_id),
            kind.slug(),
            qualified_name,
            span.start.line,
            span.start.column,
            span.end.line,
            span.end.column,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub relative_path: String,
    pub language: Language,
    /// Blob identity is keyed to file content, not `(commit, path)`.
    pub blob_id: String,
    pub byte_count: usize,
    pub line_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeUnitDescriptor {
    pub stable_id: StableCodeUnitId,
    pub file_blob_id: String,
    pub kind: CodeUnitKind,
    pub execution_kind: CodeUnitExecutionKind,
    pub exposure: CodeUnitExposure,
    pub display_name: String,
    pub qualified_name: Option<String>,
    pub parent_stable_id: Option<StableCodeUnitId>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricSubject {
    File {
        blob_id: String,
        relative_path: String,
    },
    CodeUnit {
        stable_id: StableCodeUnitId,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricRow {
    pub metric_name: String,
    pub subject: MetricSubject,
    pub value: MetricValue,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobCacheKey {
    pub blob_id: String,
    pub language: Language,
    pub grammar_version: String,
    pub normalization_rule_version: u32,
    pub metric_schema_version: u32,
}

impl BlobCacheKey {
    pub fn for_file(file: &FileSnapshot, grammar_version: impl Into<String>) -> Self {
        Self {
            blob_id: file.blob_id.clone(),
            language: file.language,
            grammar_version: grammar_version.into(),
            normalization_rule_version: NORMALIZATION_RULE_VERSION,
            metric_schema_version: METRIC_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlobSummary {
    pub file: FileSnapshot,
    pub cache_key: BlobCacheKey,
    pub units: Vec<CodeUnitDescriptor>,
    pub metrics: Vec<MetricRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeUnitDeltaKind {
    Added,
    Modified,
    Deleted,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeUnitDelta {
    pub change_kind: CodeUnitDeltaKind,
    /// Stable id in the old blob, when the unit existed before the change.
    pub old_stable_id: Option<StableCodeUnitId>,
    /// Stable id in the new blob, when the unit exists after the change.
    pub new_stable_id: Option<StableCodeUnitId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDelta {
    pub change_kind: FileChangeKind,
    pub relative_path: String,
    pub previous_path: Option<String>,
    pub old_blob_id: Option<String>,
    pub new_blob_id: Option<String>,
    pub unit_deltas: Vec<CodeUnitDelta>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitDelta {
    pub commit_id: String,
    pub parent_commit_ids: Vec<String>,
    pub files: Vec<FileDelta>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectContextManifestKind {
    CargoToml,
    GoMod,
    PyProjectToml,
    PackageJson,
}

impl ProjectContextManifestKind {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.file_name()?.to_str()? {
            "Cargo.toml" => Some(Self::CargoToml),
            "go.mod" => Some(Self::GoMod),
            "pyproject.toml" => Some(Self::PyProjectToml),
            "package.json" => Some(Self::PackageJson),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectContextManifest {
    pub kind: ProjectContextManifestKind,
    pub relative_path: String,
    pub ecosystem: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectContextSnapshot {
    pub repository_root: String,
    pub manifests: Vec<ProjectContextManifest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralQueryAnchor {
    pub node_kind: String,
    pub unit_kind: CodeUnitKind,
    pub purpose: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricEventKind {
    EnterUnit,
    ExitUnit,
    Branch,
    Loop,
    Comment,
    Parameter,
    Return,
    Import,
    Export,
    Closure,
    ParseErrorRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricEvent {
    pub kind: MetricEventKind,
    pub stable_unit_id: Option<StableCodeUnitId>,
    pub span: SourceSpan,
    pub detail: Option<String>,
}

pub trait MetricsLanguageAdapterContract {
    /// Structural anchors should come from Tree-sitter queries or similarly
    /// cheap node-kind lookups.
    fn structural_query_anchors(&self) -> Vec<StructuralQueryAnchor>;

    /// Code-unit extraction should normalize names, spans, and parent-child
    /// relationships into the shared contract.
    fn extract_code_units(
        &self,
        file: &FileSnapshot,
        tree: &ts::Tree,
        source: &str,
    ) -> Result<Vec<CodeUnitDescriptor>, ParseError>;

    /// Recursive walking for nesting-sensitive metrics stays separate from
    /// structural anchor extraction so later metrics can reuse the same event
    /// stream without re-querying the syntax tree.
    fn emit_metric_events(
        &self,
        file: &FileSnapshot,
        tree: &ts::Tree,
        source: &str,
        units: &[CodeUnitDescriptor],
    ) -> Result<Vec<MetricEvent>, ParseError>;

    fn grammar_version(&self) -> &'static str;

    fn language(&self) -> Language;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureEdgeCase {
    NestedControlFlow,
    Branching,
    Comments,
    ParameterCounts,
    MultipleReturns,
    MultipleExits,
    Imports,
    Exports,
    Closures,
    ParseErrorTolerance,
    PublicApiSurface,
    ProjectContextManifest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureCase {
    pub name: String,
    pub path_hint: String,
    pub language: Option<Language>,
    pub manifest_kind: Option<ProjectContextManifestKind>,
    pub edge_cases: Vec<FixtureEdgeCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureCorpusPlan {
    pub cases: Vec<FixtureCase>,
}

impl FixtureCorpusPlan {
    pub fn default_contract() -> Self {
        Self {
            cases: vec![
                FixtureCase {
                    name: "rust-core-metrics".into(),
                    path_hint: "fixtures/metrics_contract/rust/core.rs".into(),
                    language: Some(Language::Rust),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::NestedControlFlow,
                        FixtureEdgeCase::Branching,
                        FixtureEdgeCase::Comments,
                        FixtureEdgeCase::ParameterCounts,
                        FixtureEdgeCase::MultipleReturns,
                        FixtureEdgeCase::MultipleExits,
                        FixtureEdgeCase::PublicApiSurface,
                    ],
                },
                FixtureCase {
                    name: "go-core-metrics".into(),
                    path_hint: "fixtures/metrics_contract/go/core.go".into(),
                    language: Some(Language::Go),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::NestedControlFlow,
                        FixtureEdgeCase::Branching,
                        FixtureEdgeCase::Comments,
                        FixtureEdgeCase::Imports,
                        FixtureEdgeCase::MultipleReturns,
                        FixtureEdgeCase::MultipleExits,
                    ],
                },
                FixtureCase {
                    name: "python-parse-tolerance".into(),
                    path_hint: "fixtures/metrics_contract/python/recovery.py".into(),
                    language: Some(Language::Python),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::Closures,
                        FixtureEdgeCase::Comments,
                        FixtureEdgeCase::MultipleExits,
                        FixtureEdgeCase::ParseErrorTolerance,
                    ],
                },
                FixtureCase {
                    name: "typescript-node-surface".into(),
                    path_hint: "fixtures/metrics_contract/typescript/service.ts".into(),
                    language: Some(Language::TypeScript),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::Imports,
                        FixtureEdgeCase::Exports,
                        FixtureEdgeCase::Closures,
                        FixtureEdgeCase::PublicApiSurface,
                    ],
                },
                FixtureCase {
                    name: "tsx-component-surface".into(),
                    path_hint: "fixtures/metrics_contract/tsx/component.tsx".into(),
                    language: Some(Language::Tsx),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::Imports,
                        FixtureEdgeCase::Exports,
                        FixtureEdgeCase::Closures,
                    ],
                },
                FixtureCase {
                    name: "javascript-module-surface".into(),
                    path_hint: "fixtures/metrics_contract/javascript/module.js".into(),
                    language: Some(Language::JavaScript),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::Imports,
                        FixtureEdgeCase::Exports,
                        FixtureEdgeCase::Closures,
                    ],
                },
                FixtureCase {
                    name: "json-document".into(),
                    path_hint: "fixtures/metrics_contract/json/config.json".into(),
                    language: Some(Language::Json),
                    manifest_kind: None,
                    edge_cases: vec![FixtureEdgeCase::PublicApiSurface],
                },
                FixtureCase {
                    name: "java-api-surface".into(),
                    path_hint: "fixtures/metrics_contract/java/Service.java".into(),
                    language: Some(Language::Java),
                    manifest_kind: None,
                    edge_cases: vec![
                        FixtureEdgeCase::NestedControlFlow,
                        FixtureEdgeCase::Comments,
                        FixtureEdgeCase::PublicApiSurface,
                    ],
                },
                FixtureCase {
                    name: "hcl-module-surface".into(),
                    path_hint: "fixtures/metrics_contract/hcl/main.tf".into(),
                    language: Some(Language::Hcl),
                    manifest_kind: None,
                    edge_cases: vec![FixtureEdgeCase::Comments, FixtureEdgeCase::PublicApiSurface],
                },
                FixtureCase {
                    name: "cargo-manifest-context".into(),
                    path_hint: "fixtures/project_context/rust/Cargo.toml".into(),
                    language: None,
                    manifest_kind: Some(ProjectContextManifestKind::CargoToml),
                    edge_cases: vec![FixtureEdgeCase::ProjectContextManifest],
                },
                FixtureCase {
                    name: "gomod-context".into(),
                    path_hint: "fixtures/project_context/go/go.mod".into(),
                    language: None,
                    manifest_kind: Some(ProjectContextManifestKind::GoMod),
                    edge_cases: vec![FixtureEdgeCase::ProjectContextManifest],
                },
                FixtureCase {
                    name: "pyproject-context".into(),
                    path_hint: "fixtures/project_context/python/pyproject.toml".into(),
                    language: None,
                    manifest_kind: Some(ProjectContextManifestKind::PyProjectToml),
                    edge_cases: vec![FixtureEdgeCase::ProjectContextManifest],
                },
                FixtureCase {
                    name: "packagejson-context".into(),
                    path_hint: "fixtures/project_context/node/package.json".into(),
                    language: None,
                    manifest_kind: Some(ProjectContextManifestKind::PackageJson),
                    edge_cases: vec![FixtureEdgeCase::ProjectContextManifest],
                },
            ],
        }
    }
}

fn sanitize_key_segment(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            ':' | '@' | '\n' | '\r' | '\t' | ' ' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_code_unit_id_uses_blob_kind_name_and_span() {
        let span = SourceSpan {
            start: SourcePosition {
                line: 10,
                column: 4,
                byte_offset: 120,
            },
            end: SourcePosition {
                line: 18,
                column: 1,
                byte_offset: 240,
            },
        };

        let stable_id = StableCodeUnitId::from_parts(
            "sha256:abc123",
            &CodeUnitKind::Function,
            Some("crate::metrics::summarize"),
            &span,
        );

        assert_eq!(
            stable_id.0,
            "sha256_abc123::function::crate__metrics__summarize@10:4-18:1"
        );
    }

    #[test]
    fn blob_cache_key_tracks_contract_versions() {
        let file = FileSnapshot {
            relative_path: "src/lib.rs".into(),
            language: Language::Rust,
            blob_id: "blob-1".into(),
            byte_count: 42,
            line_count: 7,
        };

        let cache_key = BlobCacheKey::for_file(&file, "tree-sitter-rust@0.24.0");

        assert_eq!(cache_key.blob_id, "blob-1");
        assert_eq!(cache_key.language, Language::Rust);
        assert_eq!(cache_key.grammar_version, "tree-sitter-rust@0.24.0");
        assert_eq!(
            cache_key.normalization_rule_version,
            NORMALIZATION_RULE_VERSION
        );
        assert_eq!(cache_key.metric_schema_version, METRIC_SCHEMA_VERSION);
    }

    #[test]
    fn project_context_manifest_kind_matches_expected_files() {
        assert_eq!(
            ProjectContextManifestKind::from_path(Path::new("/repo/Cargo.toml")),
            Some(ProjectContextManifestKind::CargoToml)
        );
        assert_eq!(
            ProjectContextManifestKind::from_path(Path::new("/repo/go.mod")),
            Some(ProjectContextManifestKind::GoMod)
        );
        assert_eq!(
            ProjectContextManifestKind::from_path(Path::new("/repo/pyproject.toml")),
            Some(ProjectContextManifestKind::PyProjectToml)
        );
        assert_eq!(
            ProjectContextManifestKind::from_path(Path::new("/repo/package.json")),
            Some(ProjectContextManifestKind::PackageJson)
        );
        assert_eq!(
            ProjectContextManifestKind::from_path(Path::new("/repo/README.md")),
            None
        );
    }

    #[test]
    fn file_delta_can_represent_blob_scoped_unit_changes() {
        let old_unit = StableCodeUnitId("blob-old::function::pkg__work@1:0-4:1".into());
        let new_unit = StableCodeUnitId("blob-new::function::pkg__work@1:0-6:1".into());

        let delta = FileDelta {
            change_kind: FileChangeKind::Modified,
            relative_path: "src/lib.rs".into(),
            previous_path: None,
            old_blob_id: Some("blob-old".into()),
            new_blob_id: Some("blob-new".into()),
            unit_deltas: vec![CodeUnitDelta {
                change_kind: CodeUnitDeltaKind::Modified,
                old_stable_id: Some(old_unit.clone()),
                new_stable_id: Some(new_unit.clone()),
            }],
        };

        assert_eq!(delta.unit_deltas.len(), 1);
        assert_eq!(delta.unit_deltas[0].old_stable_id.as_ref(), Some(&old_unit));
        assert_eq!(delta.unit_deltas[0].new_stable_id.as_ref(), Some(&new_unit));
    }

    #[test]
    fn default_fixture_corpus_covers_required_languages_manifests_and_edge_cases() {
        let corpus = FixtureCorpusPlan::default_contract();
        let languages: Vec<_> = corpus
            .cases
            .iter()
            .filter_map(|case| case.language)
            .collect();
        let manifest_kinds: Vec<_> = corpus
            .cases
            .iter()
            .filter_map(|case| case.manifest_kind.clone())
            .collect();
        let edge_cases: Vec<_> = corpus
            .cases
            .iter()
            .flat_map(|case| case.edge_cases.iter().cloned())
            .collect();

        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::Tsx));
        assert!(languages.contains(&Language::Json));
        assert!(languages.contains(&Language::Java));
        assert!(languages.contains(&Language::Hcl));

        assert!(manifest_kinds.contains(&ProjectContextManifestKind::CargoToml));
        assert!(manifest_kinds.contains(&ProjectContextManifestKind::GoMod));
        assert!(manifest_kinds.contains(&ProjectContextManifestKind::PyProjectToml));
        assert!(manifest_kinds.contains(&ProjectContextManifestKind::PackageJson));

        assert!(edge_cases.contains(&FixtureEdgeCase::NestedControlFlow));
        assert!(edge_cases.contains(&FixtureEdgeCase::Branching));
        assert!(edge_cases.contains(&FixtureEdgeCase::Comments));
        assert!(edge_cases.contains(&FixtureEdgeCase::ParameterCounts));
        assert!(edge_cases.contains(&FixtureEdgeCase::MultipleExits));
        assert!(edge_cases.contains(&FixtureEdgeCase::Imports));
        assert!(edge_cases.contains(&FixtureEdgeCase::Exports));
        assert!(edge_cases.contains(&FixtureEdgeCase::Closures));
        assert!(edge_cases.contains(&FixtureEdgeCase::ParseErrorTolerance));
    }
}
