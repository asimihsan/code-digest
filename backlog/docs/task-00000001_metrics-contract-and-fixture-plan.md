# Task `00000001` Metrics Contract And Fixture Plan

## Summary

Task `00000001` stays inside the current workspace crates instead of creating a
new `sqm-*` split immediately.

- `language_parsers` owns the normalized parser-facing domain model.
- `output_generator` owns the report contract and additive digest compatibility
  policy.
- `file_system` remains the home for project-context file discovery.
- Future aggregation can stay repo-local until task `00000002` proves the
  static metrics slice.

## Stable Records And IDs

The contract now defines direct equivalents for the Phase 0 record set:

- `FileSnapshot`
- `CodeUnitDescriptor`
- `MetricRow`
- `BlobSummary`
- `CodeUnitDelta`
- `FileDelta`
- `CommitDelta`
- `ProjectContextSnapshot`
- `MetricsReportContract`

Stable code-unit ids are intentionally blob-based rather than commit-based.
Each `StableCodeUnitId` is derived from:

1. the blob id
2. the normalized unit kind
3. the qualified name, falling back to `anonymous`
4. the start and end coordinates

That keeps repeated history mining content-addressed and avoids tying ids to a
specific checkout path or commit hash.

Parent-child relationships stay explicit on `CodeUnitDescriptor.parent_stable_id`
instead of being hidden inside the id format.

Because those stable ids are blob-scoped, file-level change records use
`CodeUnitDelta { old_stable_id, new_stable_id }` rather than a single ambiguous
list of touched units.

## Adapter Boundary

`MetricsLanguageAdapterContract` splits the parser work into two layers:

1. `structural_query_anchors()` for Tree-sitter query anchors and cheap
   structural selection
2. `emit_metric_events()` for recursive walking of nesting-sensitive events like
   branching, loops, closures, and parse-recovery markers

That boundary keeps structural extraction reusable while making later metrics
passes consume a shared event stream instead of repeatedly traversing raw ASTs.

## Cache Contract

`BlobCacheKey` is keyed by:

- `blob_id`
- `language`
- `grammar_version`
- `normalization_rule_version`
- `metric_schema_version`

This makes the intended invalidation story explicit before any on-disk cache is
implemented.

## Additive Digest Compatibility

`MetricsReportContract` sets an explicit additive compatibility policy:

- metrics remain additive to existing digest behavior
- metrics mode must be explicit
- existing digest output is preserved by default

Task `00000002` can now build static extraction without having to renegotiate
the compatibility rules.

## Fixture Corpus Plan

`FixtureCorpusPlan::default_contract()` now defines the first required fixture
surface:

- language fixtures for Rust, Go, Python, JavaScript, TypeScript, TSX, JSON,
  Java, and HCL
- edge-case coverage for nesting, branching, comments, parameters, multiple
  returns, multiple exits, imports, exports, closures, parse-recovery, and
  public/API surface
- project-context manifests for `Cargo.toml`, `go.mod`, `pyproject.toml`, and
  `package.json`

The fixture paths are contract placeholders for the next implementation slices,
not a claim that task `00000001` already extracts those metrics.
