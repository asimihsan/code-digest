# Task `00000002` - Add static Tree-sitter metrics extraction

## Summary

Implement the first additive static-metrics pipeline on top of the existing parser infrastructure, using normalized code-unit facts, a shared event stream, and blob-keyed summaries rather than ad hoc per-metric AST walks.

## Motivation

This is the first product slice that turns the roadmap into actual capability. It should prove that `code-digest` can compute useful software quality metrics without abandoning its current digest behavior or overreaching into semantics-heavy analysis.

## Detailed Requirements / Acceptance Criteria

### A) Normalized extraction and caching

- [ ] Implement a language-adapter layer or direct equivalent that emits normalized code units and shared metric-observation events instead of making every metric traverse raw Tree-sitter nodes independently.
- [ ] Use Tree-sitter queries for structural anchors such as functions, methods, classes, imports, exports, and comments, then use a manual recursive walker where nesting-sensitive metrics need it.
- [ ] Produce blob summaries keyed by blob id so later history mining can reuse static analysis across repeated file contents.
- [ ] Keep traversal logic testable and decoupled from CLI presentation.

### B) Initial static metric set

- [ ] Implement an initial common metrics set such as cyclomatic complexity, cognitive complexity or a documented proxy, SLOC or LLOC or CLOC where practical, function count, function length, parameter counts, branch counts, exits or returns, and comment density.
- [ ] Distinguish all executable units from public or API-facing units where the language model supports it.
- [ ] Support function or method-level aggregation and file-level rollups.
- [ ] Add tests that validate metric counts on fixture inputs.

### C) Language rollout and scope discipline

- [ ] Reuse the existing Tree-sitter-backed parsing flow instead of introducing a second parser stack.
- [ ] Make the first supported language set explicit, with a likely minimum of the repo’s currently wired grammars and a recorded decision on JavaScript, TypeScript, TSX, and JSON onboarding.
- [ ] Document language-specific caveats where a metric is unavailable, approximate, or intentionally omitted.
- [ ] Explicitly exclude semantic-heavy features such as precise call graphs, type-aware coupling, alias analysis, and interprocedural data flow from this slice.
- [ ] Record follow-up work rather than burying unsupported cases in ad hoc heuristics.

## Task Dependencies

- `task-00000001`

## Implementation Notes

- Created as the first implementation slice for plan `00000001`.
- This task should focus on parser-, normalization-, and aggregation-level behavior, not on CLI or UI polish.
