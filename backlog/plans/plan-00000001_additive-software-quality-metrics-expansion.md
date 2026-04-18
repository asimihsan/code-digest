# Plan `00000001` - Additive software quality metrics expansion

## Metadata

- Date: `2026-04-12`
- Status: `draft`
- Owner: `Asim / OpenClaw`
- Related Plans: `none`
- Related Tasks: `task-00000001`, `task-00000002`, `task-00000003`, `task-00000004`, `task-00000005`

## Summary

`code-digest` already has the right shape for syntax-aware source analysis: a Rust workspace, a `language_parsers` crate built on Tree-sitter, a CLI, an output generator, and an early UI surface. This plan keeps the existing code-digestion product intact and expands it into an additive feature platform for software quality and change-risk analysis. The intended pipeline is `source files -> Tree-sitter parse -> normalized code-unit facts -> static metrics -> Git/change features -> optional labels -> model training/inference`, with each layer reusable on its own. Learned embeddings and semantic-heavy analyses remain downstream, optional layers until the simpler platform proves itself.

## Decision Summary (Locked)

- [x] Preserve the current digest and extraction workflows as first-class functionality.
- [x] Build this as a reusable feature platform, not just a one-off metrics library.
- [x] Cache static analysis by Git blob id rather than by `(commit, path)` wherever history mining is involved.
- [x] Treat labels and embeddings as optional downstream inputs, not as part of the core extraction engine.

## Goals

- Add a repo-local roadmap for software quality features without destabilizing the current digest pipeline.
- Define stable normalized records for files, code units, metric rows, file deltas, commit deltas, and later labels.
- Ship a credible first slice of syntax-derived static metrics using the existing Tree-sitter parser foundation.
- Expose metrics through the existing CLI and output surfaces in a way that composes with digest output instead of replacing it.
- Create an incremental path toward Git/process metrics, interpretable change-risk baselines, and only later optional embedding fusion.

## Non-Goals

- Rewriting `code-digest` into a defect-prediction product immediately.
- Promising compiler-grade semantic analysis such as precise call graphs, alias analysis, or interprocedural data flow in v1.
- Performing a big-bang workspace rewrite into a new `sqm-*` crate layout before the contract and initial slices prove their value.
- Making CodeBERT-style embeddings, deep models, or semantic enrichment the default early architecture.
- Removing or weakening the existing digest-oriented CLI and library behaviors.

## Current State / Baseline

- The Rust workspace currently contains `language_parsers`, `file_system`, `output_generator`, `config_manager`, `cli_tool`, and `ui`.
- `language_parsers` already depends on Tree-sitter and currently wires grammars for Go, HCL, Java, JavaScript, JSON, Python, Rust, TypeScript, and TSX.
- The README and CLI describe the current product as code digestion and extraction for LLM-oriented workflows.
- The repo has no normalized code-unit fact model, no blob-summary cache, no Git mining layer, and no label abstraction today.
- The repo backlog structure exists but is nearly empty, so this plan has to anchor directly to the current crate layout instead of extending an existing roadmap.
- User research points toward a broader multi-language platform shape, including JavaScript, TypeScript, TSX, and JSON for Node-centric repos, and those grammars are now wired in this checkout.

## Gap Analysis

### Platform contract and normalized facts

- Missing: stable records for file snapshots, code units, metric rows, blob summaries, file deltas, commit deltas, and optional labels.
- Impact: without this layer, later metrics, Git features, and model evaluation will each invent their own incompatible shapes.
- Notes: this layer should define stable ids, spans, and unit kinds before broader implementation begins.

### Static metrics engine

- Missing: a language adapter layer, normalized AST event stream, blob-keyed cache, aggregation pipeline, and fixture-based validation for syntax-derived metrics.
- Impact: the repo cannot produce complexity or maintainability-style outputs beyond the current digest view.
- Notes: this should reuse the existing Tree-sitter path and avoid `O(metrics × nodes)` walkers where every metric re-traverses raw ASTs independently.

### Change and dataset layer

- Missing: commit walking, rename-aware diffs, blame-backed ownership features, static metric deltas on changed units, dataset export, and label boundaries.
- Impact: the repo would not have a disciplined bridge from syntax metrics to change-risk analysis.
- Notes: this layer must be explicit about merges, temporal ordering, and label uncertainty.

### Product surface integration

- Missing: CLI modes, output schemas, and UI/reporting views for metrics and later feature reports alongside digest output.
- Impact: even a good extractor will be hard to use or validate if it cannot be surfaced coherently.
- Notes: additive composition matters more than inventing a second disconnected tool.

## Strategy

- Use a feature-platform pipeline: parse source once, normalize it into reusable code-unit facts, compute static metrics from those facts, join them with Git/change features, then optionally attach labels and models.
- Keep the first implementation in the current workspace crates where practical, but shape interfaces so they could later be extracted into a clearer split like `sqm-core`, `sqm-lang`, `sqm-project`, `sqm-metrics`, `sqm-git`, `sqm-dataset`, `sqm-model`, and `sqm-cli` if the platform hardens.
- Reuse Tree-sitter-backed parsing in `language_parsers`, but add language adapters that emit shared units and event streams. Use Tree-sitter queries for structural anchors like functions, classes, imports, exports, and comments, then use a recursive walker for nesting-sensitive metrics like cyclomatic and cognitive complexity.
- Cache blob summaries by blob id so history mining can reuse static analysis across repeated content instead of re-parsing every checked-out revision.
- Treat project context as first-class metadata, especially for `Cargo.toml`, `go.mod`, `pyproject.toml`, and `package.json`, so later change-risk features can detect manifest and dependency changes.
- Favor likely pragmatic building blocks: `ignore` for repo walking, `rayon` for parallelism, `serde` for persistent records, `cargo_metadata` for Rust workspace detection, `gix` as the primary Git layer with a documented escape hatch if a thin `git2` fallback is needed, and `smartcore` or `linfa` before any heavier model stack.
- Keep labels and embeddings behind explicit interfaces. Label sources should carry provenance and confidence. Embeddings should be optional fusion inputs only after tabular baselines are trustworthy.

## Phase Plan

### Phase 0 - Platform contract and cache-friendly foundations

- Objective: define the feature-platform contract and normalized records without changing the current digest contract.
- Candidate Tasks:
  - `task-00000001` Define the metrics domain model, report contract, and fixtures
- Exit Criteria:
  - [x] Stable records and ids are documented for files, units, metrics, and later change features.
  - [x] The additive boundary with the existing digest flow is explicit.
  - [x] Blob-id caching rules and fixture strategy are specified.

### Phase 1 - Static syntax-derived metrics

- Objective: ship a first set of Tree-sitter-derived metrics on top of normalized code-unit facts.
- Candidate Tasks:
  - `task-00000002` Add static Tree-sitter metrics extraction
  - `task-00000003` Expose metrics through CLI, output, and UI
- Exit Criteria:
  - [ ] Static metrics can be produced without regressing digest output.
  - [ ] Blob summaries and metric results are covered by fixtures or integration tests.
  - [ ] The target language rollout is explicit, including what happens with Node-family grammars.

### Phase 2 - Change and process metrics

- Objective: add commit-level Git features and dataset export that reuse static summaries instead of re-analyzing entire history repeatedly.
- Candidate Tasks:
  - `task-00000004` Add change and process metrics dataset export
- Exit Criteria:
  - [ ] Repo history can be transformed into a feature dataset with documented assumptions.
  - [ ] Static deltas are computed on changed units, not only on coarse file averages.
  - [ ] Labels remain optional and provenance-bearing.

### Phase 3 - Simple predictive baselines

- Objective: prove whether the extracted features support useful out-of-time baselines before trying learned semantic features.
- Candidate Tasks:
  - `task-00000005` Add temporal baseline modeling and evaluation
- Exit Criteria:
  - [ ] Baseline evaluation uses temporal splits and leakage-aware documentation.
  - [ ] The repo can explain a score in terms of changed features and affected units.
  - [ ] Embedding fusion remains deferred unless the tabular baseline earns a stronger layer.

## Backlog Task Map

| Task ID | Title | Phase | Depends On | Status |
|---|---|---|---|---|
| task-`00000001` | Define the metrics domain model, report contract, and fixtures | `Phase 0` | `none` | `completed` |
| task-`00000002` | Add static Tree-sitter metrics extraction | `Phase 1` | `task-00000001` | `proposed` |
| task-`00000003` | Expose metrics through CLI, output, and UI | `Phase 1` | `task-00000001`, `task-00000002` | `proposed` |
| task-`00000004` | Add change and process metrics dataset export | `Phase 2` | `task-00000002`, `task-00000003` | `proposed` |
| task-`00000005` | Add temporal baseline modeling and evaluation | `Phase 3` | `task-00000004` | `proposed` |

## Validation Strategy

- Unit/Integration: parser-, record-, and report-level tests in the relevant crates, plus fixture-based expectations for representative language samples.
- CLI/manual checks: `mise run test`, `mise run ci`, and targeted CLI runs that show digest-only, metrics-only, combined, and later dataset-export flows.
- Regression safeguards: snapshot or fixture parity for existing digest behavior, blob-id cache reuse checks, and explicit validation that metrics mode is opt-in or additive.
- History-mining safeguards: test first on non-merge commits against the main parent, then widen only with documented merge behavior.
- Definition of Done:
  - [ ] The plan is decomposed into atomic tasks with acceptance criteria.
  - [ ] The additive boundary to current functionality is preserved in docs and implementation.
  - [ ] Later modeling work is blocked on temporal-validation discipline instead of enthusiasm alone.

## Risks and Mitigations

| Risk | Trigger | Mitigation | Owner |
|---|---|---|---|
| Metrics scope explodes into semantic analysis | Requests for call graphs, type coupling, or data flow appear during v1 work | Keep semantic-heavy features out of scope and document them as future research | `OpenClaw` |
| Existing digest workflows regress | New metrics code rewires parser or CLI defaults | Keep metrics behind additive contracts and add regression checks for current digest output | `OpenClaw` |
| Blob-summary cache becomes unsafe or stale | Grammar versions, config, or normalization rules change without cache versioning | Include cache version inputs in the persisted key or metadata and validate with fixtures | `OpenClaw` |
| Predictive results look better than they really are | Random splits, noisy labels, or leakage creep into experiments | Require temporal validation and document label limitations before claiming signal | `OpenClaw` |
| Big-platform ambition outruns the repo | Immediate pressure to create many new crates and model layers at once | Stage the work through the current workspace first and extract only after interfaces stabilize | `OpenClaw` |
| Multi-language support gets uneven | Metric definitions diverge across Tree-sitter grammars | Start with a small common metric set and document language-specific gaps explicitly | `OpenClaw` |

## Open Questions

| Question | Needed By | Owner | Resolution |
|---|---|---|---|
| Should the first implementation stay inside the current workspace crates, or should any part of the `sqm-*` split happen immediately? | `task-00000001` | `OpenClaw` | `open` |
| Should the first static-metrics slice target all currently wired grammars immediately, or start with a smaller parity subset and expand fixture-by-fixture? | `task-00000002` | `OpenClaw` | `open` |
| Is `gix` sufficient as the primary Git layer in practice, or is a thin `git2` fallback feature worth adding from the start? | `task-00000004` | `OpenClaw` | `open` |
| Should `polars` stay optional until dataset ergonomics demand it, or become part of the first export and modeling workflow? | `task-00000004` | `OpenClaw` | `open` |

## Immediate Next Actions

1. Start `task-00000002` from the now-landed contract, beginning with the smallest static-metrics slice that exercises blob summaries plus the shared fixture corpus.
2. Keep metrics rollout additive to digest output, with explicit scope on which currently wired grammars get first-pass metric parity.
3. Delay change-risk modeling claims until change-metric extraction, label boundaries, and temporal evaluation scaffolding exist.

## Implementation Notes

- `2026-04-12`: Initial plan created from current repo structure plus user-provided research direction.
- `2026-04-12`: Expanded to reflect the feature-platform architecture, blob-id caching strategy, Git mining layer, optional label boundary, and deferred embedding fusion guidance.
- `2026-04-18`: Phase 0 contract work landed in `task-00000001`, including normalized records, blob-scoped stable ids, additive digest compatibility, and the initial fixture corpus plan.
- The plan intentionally preserves the repo’s digest identity and treats metrics as an adjacent capability.

## Completion Checklist

- [ ] All planned tasks created under `backlog/tasks/task-<id>.md`
- [ ] All task acceptance criteria checked
- [ ] Validation strategy executed
- [ ] Plan status updated to `completed`
- [ ] Plan moved to `backlog/plans/completed/`

## Changelog

- `2026-04-12` Initial draft created for additive quality-metrics expansion
- `2026-04-12` Incorporated platform, caching, Git-layer, and modeling research into the draft
- `2026-04-18` Updated the baseline to reflect current Node-family grammar support and marked `task-00000001` complete
