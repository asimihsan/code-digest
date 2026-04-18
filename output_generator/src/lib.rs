/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use language_parsers::metrics_contract::{
    BlobSummary, CommitDelta, FixtureCorpusPlan, ProjectContextSnapshot,
    METRICS_REPORT_FORMAT_VERSION, METRIC_SCHEMA_VERSION, NORMALIZATION_RULE_VERSION,
};
use serde::{Deserialize, Serialize};

pub mod static_metrics;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractOwnership {
    pub parsing: String,
    pub aggregation: String,
    pub project_context_detection: String,
    pub report_rendering: String,
}

impl Default for ContractOwnership {
    fn default() -> Self {
        Self {
            parsing: "language_parsers".into(),
            aggregation: "output_generator".into(),
            project_context_detection: "file_system".into(),
            report_rendering: "output_generator".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigestCompatibilityPolicy {
    pub additive_to_existing_digest: bool,
    pub metrics_mode_must_be_explicit: bool,
    pub preserve_existing_digest_output_by_default: bool,
}

impl Default for DigestCompatibilityPolicy {
    fn default() -> Self {
        Self {
            additive_to_existing_digest: true,
            metrics_mode_must_be_explicit: true,
            preserve_existing_digest_output_by_default: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricsReportContract {
    pub format_version: u32,
    pub metric_schema_version: u32,
    pub normalization_rule_version: u32,
    pub ownership: ContractOwnership,
    pub digest_compatibility: DigestCompatibilityPolicy,
    pub project_context: ProjectContextSnapshot,
    pub fixture_corpus: FixtureCorpusPlan,
    pub blob_summaries: Vec<BlobSummary>,
    pub commit_deltas: Vec<CommitDelta>,
}

impl MetricsReportContract {
    pub fn empty(repository_root: impl Into<String>) -> Self {
        Self {
            format_version: METRICS_REPORT_FORMAT_VERSION,
            metric_schema_version: METRIC_SCHEMA_VERSION,
            normalization_rule_version: NORMALIZATION_RULE_VERSION,
            ownership: ContractOwnership::default(),
            digest_compatibility: DigestCompatibilityPolicy::default(),
            project_context: ProjectContextSnapshot {
                repository_root: repository_root.into(),
                manifests: Vec::new(),
            },
            fixture_corpus: FixtureCorpusPlan::default_contract(),
            blob_summaries: Vec::new(),
            commit_deltas: Vec::new(),
        }
    }

    pub fn with_blob_summaries(
        repository_root: impl Into<String>,
        blob_summaries: Vec<BlobSummary>,
    ) -> Self {
        let mut contract = Self::empty(repository_root);
        contract.blob_summaries = blob_summaries;
        contract
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_parsers::metrics_contract::ProjectContextManifestKind;

    #[test]
    fn empty_contract_defaults_to_additive_digest_compatibility() {
        let contract = MetricsReportContract::empty(".");

        assert_eq!(contract.format_version, METRICS_REPORT_FORMAT_VERSION);
        assert_eq!(contract.metric_schema_version, METRIC_SCHEMA_VERSION);
        assert_eq!(
            contract.normalization_rule_version,
            NORMALIZATION_RULE_VERSION
        );
        assert!(contract.digest_compatibility.additive_to_existing_digest);
        assert!(contract.digest_compatibility.metrics_mode_must_be_explicit);
        assert!(
            contract
                .digest_compatibility
                .preserve_existing_digest_output_by_default
        );
        assert_eq!(contract.ownership.parsing, "language_parsers");
        assert_eq!(contract.ownership.aggregation, "output_generator");
        assert_eq!(contract.ownership.project_context_detection, "file_system");
        assert_eq!(contract.ownership.report_rendering, "output_generator");
    }

    #[test]
    fn empty_contract_carries_fixture_corpus_and_project_context_shape() {
        let contract = MetricsReportContract::empty("repo-root");

        assert_eq!(contract.project_context.repository_root, "repo-root");
        assert!(contract.project_context.manifests.is_empty());
        assert!(contract
            .fixture_corpus
            .cases
            .iter()
            .any(|case| { case.manifest_kind == Some(ProjectContextManifestKind::CargoToml) }));
        assert!(contract
            .fixture_corpus
            .cases
            .iter()
            .any(|case| { case.manifest_kind == Some(ProjectContextManifestKind::PackageJson) }));
    }

    #[test]
    fn with_blob_summaries_keeps_additive_contract_defaults() {
        let contract = MetricsReportContract::with_blob_summaries("repo-root", Vec::new());

        assert_eq!(contract.project_context.repository_root, "repo-root");
        assert!(contract.blob_summaries.is_empty());
        assert!(contract.digest_compatibility.additive_to_existing_digest);
    }
}
