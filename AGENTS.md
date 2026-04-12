# AGENTS.md - Repository Guide for Large-Language-Model Agents

Our goal is a small, reproducible Rust workspace for digesting codebases, with
standardized local workflows through `mise`, tracked backlog notes, and
Git-compatible local development through jujutsu (`jj`).

Use plan tool to track tasks. Use jujutsu (`jj`) for version control. Use
independent sub-agents as needed. If available in this Codex session, use the
`$jujutsu` skill for command semantics.

Use `jj` to inspect recent changes and open backlog notes before making
substantial edits. Keep work scoped, verifiable, and aligned with the repo task
surface.

- Fast iteration: `mise run dev`
- Canonical local/CI pipeline: `mise run ci`
- Keep task implementation details and verification notes current while coding.

## Backlog plans and tasks

Backlog is tracked in Git in this repository. Keep backlog docs, plans, and
tasks auditable alongside code changes.

When creating a new plan or task, once you think you are finished, iteratively
review with an independent sub-agent and make fix-ups and re-reviews until both
of you are satisfied.

### Plans

A plan is a broader idea that is broken down into tasks. Plans are stored in
`backlog/plans/`.

Plans should use the template in `backlog/plans/_template.md`.

### Tasks

A task is a specific, actionable item that is part of a plan. Tasks are stored
in `backlog/tasks/`.

- Each commit/PR should reference a task ID from
  `backlog/tasks/task-<eight-digit-id>_<title>.md`.
- Do not start implementing before converting an idea into a task with clear
  acceptance criteria.
- Tasks must be atomic, independent, and testable.
- New tasks must follow the template in `backlog/tasks/_template.md`.
- While implementing, update the task's Implementation Notes and validation
  evidence.
- After a task is complete:
  1. ensure `mise run ci` passes, or capture the blocker clearly.
  2. use an independent sub-agent and iterate on review findings until both of
     you are satisfied.
  3. tick all acceptance criteria and add implementation notes.
  4. keep the parent plan up to date; add/remove/edit tasks and update Markdown
     checklists.
  5. move completed tasks into `backlog/tasks/completed/` when the repo’s
     backlog bookkeeping for that change is finalized.
  6. use jujutsu (`jj`) for local history management while preserving the Git
     remote.
