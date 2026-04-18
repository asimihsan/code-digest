# Task `00000001` - Define metrics domain model, report contract, and fixtures

## Summary

Define the additive feature-platform contract for `code-digest` so later parser, metrics, Git, and model work all share the same records, ids, cache semantics, and report boundaries.

## Motivation

The repo already has Tree-sitter-backed parsing and digest-oriented output, but it does not yet have a shared concept of file snapshots, code units, metric rows, blob summaries, commit deltas, or optional labels. Locking that contract first prevents later implementation from fragmenting into incompatible one-off data shapes.

## Detailed Requirements / Acceptance Criteria

### A) Core records and stable ids

- [x] Define the initial platform record types or their direct equivalents for file snapshots, code units, metric rows, blob summaries, file deltas, and commit deltas.
- [x] Define language and unit-kind enums that can cover the current repo surface and later Node-family support without forcing immediate grammar expansion.
- [x] Define stable ids for code units based on blob identity plus structural location, with documented rules for names, qualified names, parent-child relationships, and spans.
- [x] Distinguish executable units from public or API units in the contract so later reports can talk about both internal complexity and exposed surface area.

### B) Adapter, event, and integration contract

- [x] Define the normalized adapter contract that language-specific Tree-sitter logic must satisfy, including code-unit extraction and a shared event stream or equivalent abstraction for metric passes.
- [x] Record the boundary between Tree-sitter query usage for structural anchors and manual recursive walking for nesting-sensitive metrics.
- [x] Decide which current crates own parsing, aggregation, project-context detection, and report rendering, while keeping open whether a later `sqm-*` split is warranted.
- [x] Define compatibility rules so metrics remain additive to the current digest behavior by default.

### C) Cache, project context, and fixture plan

- [x] Specify that history-oriented static analysis is cached by blob id rather than by `(commit, path)`.
- [x] Document what additional cache-version inputs matter, such as grammar version, normalization rules, or metric schema version.
- [x] Define the first fixture corpus, including representative samples for Rust, Go, Python, and the current decision on HCL, Java, and Node-family grammars.
- [x] Include edge cases for nesting, branching, comments, parameter counts, multiple exits, imports, exports, closures, and parse-error tolerance.
- [x] Define how project context like `Cargo.toml`, `go.mod`, `pyproject.toml`, and `package.json` will be discovered or represented for later feature extraction.

## Task Dependencies

- none

## Implementation Notes

- Created as the contract-first task for plan `00000001`.
- This task should finish before implementation code changes that expose metrics or Git features publicly.
- Added the shared metrics/domain contract in `language_parsers/src/metrics_contract.rs`, including blob-scoped stable ids, file/code-unit/metric/blob summary records, explicit `CodeUnitDelta` before/after ids, cache-key versioning, project-context manifest records, the parser adapter boundary, and the initial fixture corpus plan.
- Extended `language_parsers::Language` with serde support plus helpers for stable slugs and digest-selector capability checks so the contract can serialize across crates cleanly.
- Added the report-layer contract in `output_generator/src/lib.rs`, including additive digest compatibility defaults and crate-ownership decisions for parsing, aggregation, project-context detection, and rendering.
- Documented the Phase 0 decisions in `backlog/docs/task-00000001_metrics-contract-and-fixture-plan.md` and refreshed the parent plan to reflect current JS/TS/TSX/JSON grammar support plus `task-00000001` completion.
- Validation:
  - `cargo test -p language_parsers -p output_generator`
  - `cargo test --workspace`
  - `mise run ci` reached the audit step and then failed on pre-existing `rustls-webpki 0.103.11` advisories `RUSTSEC-2026-0098` and `RUSTSEC-2026-0099` through the existing `ui`/`dioxus` dependency stack; fmt, lint, build, and test stages passed before that blocker.
- Independent review: a sub-agent review flagged two real gaps, which were fixed before completion: ambiguous file-unit deltas with blob-scoped ids and missing fixture coverage for multiple exits.
