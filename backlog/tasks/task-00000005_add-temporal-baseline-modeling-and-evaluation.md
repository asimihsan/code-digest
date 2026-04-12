# Task `00000005` - Add temporal baseline modeling and evaluation

## Summary

Add a simple, interpretable evaluation path for the exported metrics so `code-digest` can test whether its static and change-level features support useful out-of-time predictive baselines.

## Motivation

The hardest part of defect prediction is not parsing code, it is avoiding misleading evaluation. This task keeps the roadmap honest by requiring temporal validation, explicit label assumptions, interpretable baselines, and clear evidence before any embedding-heavy layer is allowed to become central.

## Detailed Requirements / Acceptance Criteria

### A) Baseline modeling surface

- [ ] Add a reproducible path to train and evaluate at least one interpretable baseline such as logistic regression, random forest, or gradient boosting on exported features.
- [ ] Prefer a pragmatic model stack such as `smartcore` or `linfa` before introducing heavier dependencies.
- [ ] Keep the modeling interface decoupled from parser internals so datasets can be regenerated independently.
- [ ] Record feature provenance clearly enough that baseline results can be inspected and challenged.

### B) Temporal evaluation and explanation discipline

- [ ] Use time-aware train and test splits rather than random splits.
- [ ] Document leakage risks, label quality assumptions, and any known validity limits.
- [ ] Add a reproducible command or workflow that can be rerun on the same dataset export.
- [ ] Produce at least one explanation-oriented output that can say, in substance, which features and changed units drove a risky score.
- [ ] Avoid claims that the baseline generalizes beyond the repositories and labels actually evaluated.

### C) Embedding-fusion guardrails

- [ ] State explicitly that learned embeddings or deep models are deferred until they can beat the simple baseline out of time.
- [ ] If embedding work is explored later, frame it as optional fusion on top of the tabular feature set rather than as the replacement for the platform.
- [ ] Record that literal CodeBERT should not be the unexamined default for Rust- and TypeScript-heavy repos.
- [ ] Record whether later inference would live behind ONNX or another bounded serving path, while allowing training to happen outside Rust if that is the practical choice.
- [ ] Keep benchmark reporting honest about runtime cost versus performance gain.

## Task Dependencies

- `task-00000004`

## Implementation Notes

- Created as the disciplined modeling capstone for plan `00000001`.
- This task exists to prevent the roadmap from jumping from parsing straight to flashy but weakly validated ML claims.
