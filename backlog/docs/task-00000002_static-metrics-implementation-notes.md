# Task `00000002` Static Metrics Implementation Notes

## Working Summary

- Scope is limited to the first static Tree-sitter metrics slice on top of the
  task `00000001` contracts.
- Implementation will stay additive to the current digest flow and avoid CLI/UI
  polish in this task.
- Planned first-pass language set: `go`, `rust`, `javascript`, `typescript`,
  `tsx`, and `json`.
- Deferred languages for this slice: `python`, `java`, and `hcl`.
- Semantic-heavy features such as call graphs, type-aware coupling, alias
  analysis, and interprocedural data flow remain explicit non-goals.

## Evidence Log

- `2026-04-18`: Read `AGENTS.md`, the task and plan docs, and the landed task
  `00000001` contract commit summary (`git show --stat --summary --oneline
  6c97c1c91308` used as a sandbox-safe substitute for `jj show`).
- `2026-04-18`: Confirmed the repo was initially clean apart from the
  pre-existing untracked `.codex` path (`git status --short --branch`), then
  removed that transient artifact before final validation.
- `2026-04-18`: Traced the current parser and report boundaries in
  `language_parsers/src/lib.rs`, `language_parsers/src/metrics_contract.rs`,
  and `output_generator/src/lib.rs`.
- `2026-04-18`: Implemented the first static-metrics slice in
  `language_parsers/src/static_metrics*.rs` plus `output_generator/src/static_metrics.rs`,
  with fixture-backed coverage in `language_parsers/tests/static_metrics_extraction.rs`
  and `output_generator/tests/static_metrics_summary.rs`.
- `2026-04-18`: Proactively fixed the pre-existing `cargo audit` blocker in the
  same branch by updating `rustls-webpki` to `0.103.12` in `Cargo.lock`.
- `2026-04-18`: Independent review surfaced three initial correctness fixes
  that were applied before packaging: use Git-compatible blob ids, keep Rust
  `function_signature_item` declarations non-executable, and preserve inline
  comment lines in `logical_code_line_count`.
- `2026-04-18`: Final re-review caught one more blocking gap in the first-pass
  JavaScript/TypeScript/TSX slice: common named arrow/function-expression units
  were missing from executable rollups. Fixed by teaching the structural query
  layer to normalize variable-assigned and assignment-based function
  expressions, restoring direct CommonJS export exposure for assignment-based
  units, plus coverage for TSX exported components and single-parameter arrow
  functions.
- `2026-04-18`: Final validation passed with `cargo fmt --all`,
  `cargo test -p language_parsers -p output_generator`,
  `cargo test --workspace`, and `mise run ci`.

## Deferred Follow-ups

- Start `task-00000003` to expose the landed metrics through additive CLI,
  output, and UI surfaces.
- Expand deferred language coverage (`python`, `java`, `hcl`) fixture by
  fixture instead of widening this slice with ad hoc heuristics.
- Revisit the `public_api_unit_count` naming/semantics when the next surface
  task defines user-facing metrics output.
