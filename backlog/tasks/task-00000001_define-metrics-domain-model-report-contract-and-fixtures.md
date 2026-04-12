# Task `00000001` - Define metrics domain model, report contract, and fixtures

## Summary

Define the additive feature-platform contract for `code-digest` so later parser, metrics, Git, and model work all share the same records, ids, cache semantics, and report boundaries.

## Motivation

The repo already has Tree-sitter-backed parsing and digest-oriented output, but it does not yet have a shared concept of file snapshots, code units, metric rows, blob summaries, commit deltas, or optional labels. Locking that contract first prevents later implementation from fragmenting into incompatible one-off data shapes.

## Detailed Requirements / Acceptance Criteria

### A) Core records and stable ids

- [ ] Define the initial platform record types or their direct equivalents for file snapshots, code units, metric rows, blob summaries, file deltas, and commit deltas.
- [ ] Define language and unit-kind enums that can cover the current repo surface and later Node-family support without forcing immediate grammar expansion.
- [ ] Define stable ids for code units based on blob identity plus structural location, with documented rules for names, qualified names, parent-child relationships, and spans.
- [ ] Distinguish executable units from public or API units in the contract so later reports can talk about both internal complexity and exposed surface area.

### B) Adapter, event, and integration contract

- [ ] Define the normalized adapter contract that language-specific Tree-sitter logic must satisfy, including code-unit extraction and a shared event stream or equivalent abstraction for metric passes.
- [ ] Record the boundary between Tree-sitter query usage for structural anchors and manual recursive walking for nesting-sensitive metrics.
- [ ] Decide which current crates own parsing, aggregation, project-context detection, and report rendering, while keeping open whether a later `sqm-*` split is warranted.
- [ ] Define compatibility rules so metrics remain additive to the current digest behavior by default.

### C) Cache, project context, and fixture plan

- [ ] Specify that history-oriented static analysis is cached by blob id rather than by `(commit, path)`.
- [ ] Document what additional cache-version inputs matter, such as grammar version, normalization rules, or metric schema version.
- [ ] Define the first fixture corpus, including representative samples for Rust, Go, Python, and the current decision on HCL, Java, and Node-family grammars.
- [ ] Include edge cases for nesting, branching, comments, parameter counts, multiple exits, imports, exports, closures, and parse-error tolerance.
- [ ] Define how project context like `Cargo.toml`, `go.mod`, `pyproject.toml`, and `package.json` will be discovered or represented for later feature extraction.

## Task Dependencies

- none

## Implementation Notes

- Created as the contract-first task for plan `00000001`.
- This task should finish before implementation code changes that expose metrics or Git features publicly.
