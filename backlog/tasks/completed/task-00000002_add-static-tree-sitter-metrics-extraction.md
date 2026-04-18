# Task `00000002` - Add static Tree-sitter metrics extraction

## Summary

Implement the first additive static-metrics pipeline on top of the existing parser infrastructure, using normalized code-unit facts, a shared event stream, and blob-keyed summaries rather than ad hoc per-metric AST walks.

## Motivation

This is the first product slice that turns the roadmap into actual capability. It should prove that `code-digest` can compute useful software quality metrics without abandoning its current digest behavior or overreaching into semantics-heavy analysis.

## Detailed Requirements / Acceptance Criteria

### A) Normalized extraction and caching

- [x] Implement a language-adapter layer or direct equivalent that emits normalized code units and shared metric-observation events instead of making every metric traverse raw Tree-sitter nodes independently.
- [x] Use Tree-sitter queries for structural anchors such as functions, methods, classes, imports, exports, and comments, then use a manual recursive walker where nesting-sensitive metrics need it.
- [x] Produce blob summaries keyed by blob id so later history mining can reuse static analysis across repeated file contents.
- [x] Keep traversal logic testable and decoupled from CLI presentation.

### B) Initial static metric set

- [x] Implement an initial common metrics set such as cyclomatic complexity, cognitive complexity or a documented proxy, SLOC or LLOC or CLOC where practical, function count, function length, parameter counts, branch counts, exits or returns, and comment density.
- [x] Distinguish all executable units from public or API-facing units where the language model supports it.
- [x] Support function or method-level aggregation and file-level rollups.
- [x] Add tests that validate metric counts on fixture inputs.

### C) Language rollout and scope discipline

- [x] Reuse the existing Tree-sitter-backed parsing flow instead of introducing a second parser stack.
- [x] Make the first supported language set explicit, with a likely minimum of the repo’s currently wired grammars and a recorded decision on JavaScript, TypeScript, TSX, and JSON onboarding.
- [x] Document language-specific caveats where a metric is unavailable, approximate, or intentionally omitted.
- [x] Explicitly exclude semantic-heavy features such as precise call graphs, type-aware coupling, alias analysis, and interprocedural data flow from this slice.
- [x] Record follow-up work rather than burying unsupported cases in ad hoc heuristics.

## Task Dependencies

- `task-00000001`

## Implementation Notes

- Created as the first implementation slice for plan `00000001`.
- This task should focus on parser-, normalization-, and aggregation-level behavior, not on CLI or UI polish.
- Added a first-pass static-metrics extraction path in `language_parsers`, centered on shared normalized code units plus a shared metric-event stream instead of per-metric AST walks.
- Kept the initial language rollout explicit: `go`, `rust`, `javascript`, `typescript`, `tsx`, and `json` are supported in this slice; `python`, `java`, and `hcl` remain deferred until later follow-up work.
- Used Tree-sitter queries for structural anchors and comment capture, then a recursive syntax walk for nesting-sensitive events such as branch, loop, return, and closure observations.
- Added output-layer aggregation in `output_generator` that turns extracted observations into blob-keyed summaries with file rollups and executable-unit metrics.
- Implemented the first metric set as syntax-derived approximations: source lines, logical code lines, comment lines, comment density, function count, public API unit count, import/export counts, branch/loop/return/closure counts, function length, parameter count, cyclomatic complexity, and a documented nesting-weighted cognitive-complexity proxy.
- Added fixture-backed tests for Rust and TypeScript rollups plus smoke coverage for Go, JavaScript, TSX, and JSON extraction.
- Caveats and non-goals are documented in code and notes: public API detection is syntax-only, JSON is document-rollup-only, named variable-assigned arrow and function expressions are normalized as executable units while more anonymous JavaScript forms stay intentionally coarse, and semantic-heavy analyses such as call graphs, type-aware coupling, alias analysis, and interprocedural data flow remain out of scope.
- Follow-up work remains for fuller multi-language parity and richer semantic metrics; unsupported languages and semantics were left explicit rather than hidden behind heuristics.
- Independent review follow-up fixes tightened five correctness edges before packaging: blob summaries now use Git-compatible blob ids, Rust `function_signature_item` declarations stay non-executable, logical code lines no longer drop code lines that happen to contain inline comments, JavaScript/TypeScript/TSX now normalize common named arrow/function-expression executable units instead of dropping them from function rollups, and direct CommonJS export assignments now stay public when those normalized units feed file rollups.
- Proactively fixed the previously blocking `cargo audit` failure in the same branch by updating `rustls-webpki` from `0.103.11` to `0.103.12` in `Cargo.lock`, which cleared advisories `RUSTSEC-2026-0098` and `RUSTSEC-2026-0099` in the `ui` / `dioxus` dependency chain.
- Validation:
  - `cargo test -p language_parsers -p output_generator`
  - `cargo fmt --all`
  - `cargo test --workspace`
  - `mise run ci` ✅
