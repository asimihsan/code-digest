# Task `00000003` - Expose metrics through CLI, output, and UI

## Summary

Add an operator-facing path to request, persist, and inspect metrics so the new capability is usable without displacing the current digest-oriented workflows.

## Motivation

Metrics are only valuable if they can be requested and interpreted consistently. This task turns the parser-level metrics pipeline into a practical repo feature by connecting it to the CLI, output generator, and a minimal UI summary surface while preserving default digest behavior.

## Detailed Requirements / Acceptance Criteria

### A) CLI surface

- [ ] Add an explicit CLI path for metrics that does not silently change current default output.
- [ ] Support at least digest-only, metrics-only, and clearly documented combined behavior.
- [ ] Record whether the first operator surface is flag-based, subcommand-based, or another additive shape, and document why.
- [ ] Leave room for later platform verbs such as scan, diff, export, train, or score without forcing them all into this slice.
- [ ] Add CLI tests or smoke checks for the chosen surface.

### B) Output generator and persistence contract

- [ ] Extend the output layer with a stable structured metrics report shape that later Git and modeling tasks can consume.
- [ ] Render summaries that are useful for humans and structured enough for later machine joins.
- [ ] Keep room for per-function detail, file-level rollups, and later commit-level explanations without forcing all consumers to pay for maximum detail.
- [ ] Validate the serialized or rendered output against fixture expectations.

### C) UI and operator inspection

- [ ] Add a minimal UI summary or inspection path for metrics if the existing Dioxus app can support it without large architectural churn.
- [ ] If the UI surface is deferred, document the decision explicitly and keep the task scoped to CLI and output integration with a recorded follow-up.
- [ ] Ensure the first operator experience makes it easy to answer questions like which files or functions are most complex and which metrics changed the most.
- [ ] Avoid a large UI rewrite in this slice.

## Task Dependencies

- `task-00000001`
- `task-00000002`

## Implementation Notes

- Created as the product-surface slice for plan `00000001`.
- The key constraint is additive behavior: existing digest users should not get surprising output changes.
