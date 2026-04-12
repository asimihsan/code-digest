# Final Summary

- Refreshed all existing vendored tree-sitter grammar submodules to current upstream `HEAD` as of 2026-04-12.
- Added `tree-sitter-javascript`, `tree-sitter-typescript` (for both TypeScript and TSX), and `tree-sitter-json` as new vendored submodules.
- Wired JavaScript, TypeScript, TSX, and JSON through `language_parsers` and the CLI extension router.
- Left HCL, Java, and Python vendored but still not digest-configured; their grammars build cleanly, but this repo still lacks language-specific selector choices worth exposing as first-class digest output.
- Validation: `cargo test -p language_parsers -p code-digest` and `cargo test --workspace` both passed.

# Tree-sitter Vendor Refresh Audit

## Existing submodules updated

| Grammar | Previous SHA | New SHA |
| --- | --- | --- |
| `tree-sitter-go` | `64457ea6b73ef5422ed1687178d4545c3e91334a` | `2346a3ab1bb3857b48b29d779a1ef9799a248cd7` |
| `tree-sitter-hcl` | `becebebd3509c02e871c99be55556269be1def1b` | `64ad62785d442eb4d45df3a1764962dafd5bc98b` |
| `tree-sitter-java` | `c194ee5e6ede5f26cf4799feead4a8f165dcf14d` | `e10607b45ff745f5f876bfa3e94fbcc6b44bdc11` |
| `tree-sitter-python` | `62827156d01c74dc1538266344e788da74536b8a` | `26855eabccb19c6abf499fbc5b8dc7cc9ab8bc64` |
| `tree-sitter-rust` | `0a70e15da977489d954c219af9b50b8a722630ee` | `77a3747266f4d621d0757825e6b11edcbf991ca5` |

## New submodules added

| Grammar | SHA | Notes |
| --- | --- | --- |
| `tree-sitter-javascript` | `58404d8cf191d69f2674a8fd507bd5776f46cb11` | JavaScript digest support |
| `tree-sitter-typescript` | `75b3874edb2dc714fb1fd77a32013d0f8699989f` | TypeScript and TSX digest support |
| `tree-sitter-json` | `001c28d7a29832b06b0e831ec77845553c89b56d` | JSON whole-document digest support |

## Wiring decisions

- JavaScript: supported for `.js`, `.mjs`, and `.cjs`.
- TypeScript: supported for `.ts`.
- TSX: supported for `.tsx`.
- JSON: supported for `.json` by emitting the full parsed document as one digest unit.
- HCL, Java, and Python: kept vendored and buildable, but still not routed through digest selectors because the repo does not yet define a stable output policy for those languages.
