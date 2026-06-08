---
name: deep-planning
description: Plan a large/cross-cutting SIMPel feature or change before coding — staged analysis → requirements → DB → backend → frontend → task breakdown, with cross-referenced artifacts and per-task testing. Use for multi-file/multi-layer features, migrations touching several services, or anything needing a confirmed plan before implementation. NOT for small/local edits (just do them).
---

# Deep planning (SIMPel)

A disciplined plan-before-build flow for big changes, adapted to Claude Code
primitives. Adapted from `.github/agents/Deep Plan.agent.md` (the legacy
Cursor-style agent); this is the lean Claude Code version. For interactive plan
approval, use **plan mode** (EnterPlanMode/ExitPlanMode) to present the final plan.

## Primitive mapping (vs the old agent)
- `@codebase` / broad search → **`Explore` subagent** (fan-out; conclusions + file:line).
- `@think`/`@sequentialthinking` → your own reasoning; for a structured strategy use
  the **`Plan` subagent**.
- `@web` → **WebSearch/WebFetch**.
- task tracking → **TaskCreate/TaskUpdate** (or plan mode todos).
- writing the plan/docs → Write/Edit (don't touch `docs/`; a scratch plan file is fine).

## Stages (confirm at each checkpoint before proceeding)
1. **Analysis** — map the affected area with `repo-atlas`/`Explore`; state the
   problem, current behavior, constraints (read the relevant `AGENTS.md`).
2. **Requirements** `[Req]` — enumerate functional + non-functional requirements,
   acceptance criteria. **Checkpoint: confirm with the user.**
3. **Data / DB** `[DB]` — schema changes as refinery migrations (idempotent,
   no-ALTER; see `add-backend-feature`); cross-service ordering.
4. **Backend** `[BE]` — endpoints/services/repositories per service; cite where each
   lands; reference `[Req]`/`[DB]`.
5. **Frontend** `[FE]` — feature-first components/pages/api; reference `[Req]`/`[BE]`
   (see `leptos-expert`).
6. **Task breakdown** — ordered, independently-verifiable tasks, each with its
   cross-refs (`[Req-n]`/`[DB-n]`/`[BE-n]`/`[FE-n]`) and **explicit test step**.
7. **Implementation** — execute tasks in order; **every task ships with its test**;
   keep each step compiling/green; commit incrementally.

## Principles (match repo norms)
- **Verify against real code/data**, not assumptions; cite `file:line`. Diagrams in
  **mermaid**. Avoid speculative features (justify each technical proposal).
- Respect deploy/lib/feature-first guardrails (staging→prod, WASM-safe lib, Helm-only,
  immutable tags) — surface them in the plan.
- Right-size: a 2-file change does **not** need this; reserve for genuinely
  cross-cutting work.
- Present the final plan via **plan mode** for approval before large edits.
