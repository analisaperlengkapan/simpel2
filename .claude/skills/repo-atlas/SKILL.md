---
name: repo-atlas
description: Produce a DeepWiki-style navigable architecture map of the SIMPel codebase or a subsystem — what each crate/service/module is, how they fit together, data/control flow, entry points, and cross-references with file:line. Use for onboarding, "explain how X works end-to-end", "map this subsystem", or a fresh architecture overview. NOT for rule/convention lookup (that's AGENTS.md) or routine code edits.
---

# repo-atlas — codebase architecture map (DeepWiki-style)

Generate a navigable, accurate "atlas" of the codebase (or one subsystem): the
mental model a new engineer needs — *what exists, how it connects, where to look* —
grounded in real code, not guesses.

## Method

1. **Start from the maps, don't re-derive.** Read root `AGENTS.md` (AI Routing
   Guide) + the relevant per-domain `AGENTS.md` first; they encode the intended
   structure. The atlas *explains/visualizes* that + verifies it against code.
2. **Fan out with the `Explore` subagent** for breadth (crates, services, module
   layout, naming conventions) — you want conclusions + `file:line` anchors, not
   whole-file dumps. Scope it ("medium" vs "very thorough").
3. **Trace, don't just list.** For a subsystem, follow one real flow end-to-end
   (e.g. request → route → handler → service → repository → DB; or FE page →
   `features/<domain>/api` → backend endpoint). Name the actual symbols + files.
4. **Verify before asserting.** Open the cited files; AGENTS.md/memories are
   point-in-time and may be stale. Cite `path:line` so claims are checkable.

## Output structure (per atlas / per subsystem)

- **Overview** — one paragraph: purpose + where it sits in the monorepo.
- **Components table** — module/crate · responsibility · key entry point (`file:line`).
- **Architecture diagram** — a **mermaid** `flowchart`/`sequenceDiagram` of the real
  components & data flow (matches the code, not aspirational).
- **Key flows** — numbered walkthroughs of the main paths, citing symbols/files.
- **Boundaries & contracts** — public API, DTOs/traits, cross-crate dependencies,
  WASM-safety boundaries, ports & adapters.
- **Entry points & "start here"** — where to begin reading; where to add things
  (link the relevant `add-backend-feature` / feature-first conventions).
- **Gotchas** — non-obvious coupling, drift, TODOs (verified, not speculative).

## Output location & guardrails

- Default: render the atlas **in the response** (markdown + mermaid). Persist only
  if the user names a target. **Do NOT write into `docs/`** (curated, hands-off per
  repo rules) — if persisting, use a path the user chooses (e.g. a new dir) or
  append to the relevant `AGENTS.md` only when the user asks.
- **Atlas ≠ rules.** Conventions/"must do X" belong in `AGENTS.md`; the atlas is the
  descriptive map. Don't duplicate AGENTS.md content — link to it.
- Match repo language norms (these docs are largely Indonesian); keep diagrams
  faithful to current code and note anything you couldn't verify.

## Scope control

Ask/confirm the scope before a big sweep: whole monorepo (high-level), one domain
(`layanan/perlengkapan`, `antarmuka/portal`, `lib/`), or one flow. A focused,
verified subsystem atlas beats a shallow everything-map.
