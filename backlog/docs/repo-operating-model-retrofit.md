# Repo Operating Model Retrofit Notes

## Summary

This note captures the retrofit that aligns `code-digest` with the standard
operating model used in newer Rust repos in this workspace.

## Current State Before Retrofit

- `code-digest` already built and tested successfully through raw Cargo when the
  inherited `RUSTC_WRAPPER` issue was bypassed.
- The repo had a minimal `mise.toml` used only as a tool list.
- Local task entrypoints lived in `Justfile`.
- CI installed Rust tools ad hoc instead of routing through a standardized repo
  task surface.
- The repo had no tracked backlog, no repo-local `AGENTS.md`, no colocated
  `jj` repo, and no committed lockfiles for Cargo or mise.

## Comparison Against `repo-backlog-bootstrap`

- Missing before retrofit:
  - `AGENTS.md`
  - `backlog/docs`, `backlog/plans`, `backlog/tasks`
  - `mise-tasks/`
  - `scripts/env.sh`
  - `scripts/postinstall.sh`
  - `.jj`
- Bootstrap defaults assume backlog is local-only unless deliberately tracked.
  This repo chooses tracked backlog artifacts instead.

## Comparison Against `sheeptext`

- `sheeptext` uses `mise` as the primary interface for local development and
  CI, with `mise run ci` as the canonical pipeline.
- `sheeptext` keeps a colocated `.jj` repo and a tracked backlog.
- `sheeptext` pins a richer tool surface than `code-digest` needs right now.
  This retrofit intentionally adopts only the smaller subset needed for
  reproducible builds, linting, auditing, and UI serving.

## Deliberate Boundaries

- Keep the application architecture intact.
- Do not migrate Dioxus from `0.6` to `0.7`.
- Do not introduce a larger lint/test toolchain than needed for the standardized
  task surface.
- Prefer minimal fixes required to make strict linting and standardized CI work.

## Verification Targets

- `mise trust`
- `mise install`
- `mise tasks`
- `mise run ci`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `cargo build --workspace --release`
- `jj st --no-pager`
