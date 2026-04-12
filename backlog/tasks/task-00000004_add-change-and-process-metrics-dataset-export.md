# Task `00000004` - Add change and process metrics dataset export

## Summary

Extend `code-digest` beyond static syntax-derived metrics by adding Git-history-based change and process metrics plus an export path suitable for later evaluation work.

## Motivation

Static code metrics and change-risk features are related but not identical. If the repo is going to support later defect-prediction or change-risk experimentation, it needs a disciplined intermediate layer that extracts change and process features without pretending that metric extraction alone is prediction.

## Detailed Requirements / Acceptance Criteria

### A) Git layer and commit-delta extraction

- [ ] Add a commit and file-delta extraction layer with rename-aware metadata, hunk counts, added or deleted lines, and parent-child commit relationships.
- [ ] Treat non-merge commits against their main parent as the first supported history-mining slice, with merge handling documented separately.
- [ ] Prefer a primary `gix`-based implementation unless a concrete gap justifies a thin `git2` fallback.
- [ ] Use blob-summary joins so static metrics are reused across history instead of re-parsing every file revision.

### B) Change and process feature set

- [ ] Define and implement an initial commit-level feature set spanning size and diffusion, history and experience, commit-text heuristics, ownership or hotspot signals, and static deltas on changed units.
- [ ] Include static delta features that operate on changed units, not only on coarse file averages, such as jumps in complexity, arguments, exits, or touched public API units.
- [ ] Include repo-context signals such as manifest changes, dependency-change hints, and language-mix changes where practical.
- [ ] Add tests or reproducible fixtures for at least a small synthetic or repo-local history slice.

### C) Export and optional label boundary

- [ ] Add an export path that can produce a feature dataset for later analysis.
- [ ] Include enough metadata to support temporal ordering, provenance, and later label joins.
- [ ] Define an optional label boundary that carries source and confidence instead of baking any single labeling heuristic into the dataset contract.
- [ ] Document label quality limits, SZZ-style heuristic caveats, and leakage concerns as explicit downstream constraints.
- [ ] Avoid coupling the export format to a specific modeling library.

## Task Dependencies

- `task-00000002`
- `task-00000003`

## Implementation Notes

- Created as the bridge from static metrics into later modeling work for plan `00000001`.
- This task should prefer honest, inspectable exports over premature benchmark optimization.
