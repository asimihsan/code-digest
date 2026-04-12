# UI Development

The UI crate is a Dioxus application within the main workspace.

Use the repository task surface from the repo root instead of invoking ad hoc
commands directly:

```bash
mise run ui:serve
```

That task runs `dx serve --platform desktop` from `ui/`.

Common root-level commands while working on the UI:

- `mise run fmt`
- `mise run lint`
- `mise run test`
- `mise run build`
