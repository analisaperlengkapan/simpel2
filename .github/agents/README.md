# `.github/agents/` — legacy custom-mode agents (DEPRECATED for Claude Code)

`Deep Plan.agent.md` and `Arsitektur.agent.md` are **Cursor/Cline custom-mode**
definitions (JSON `customModes`). They are **invisible to Claude Code** and use tools
that do not exist here (`@codebase`, `@think`, `@sequentialthinking`, `@web`).

**Claude Code equivalents (use these):**

| Legacy agent | Claude Code skill |
|---|---|
| `Deep Plan.agent.md` | skill **`deep-planning`** (`.claude/skills/deep-planning/`) |
| `Arsitektur.agent.md` | skill **`repo-atlas`** (`.claude/skills/repo-atlas/`) |

These files are **kept only for non-Claude tooling** (Cursor/Cline users). They are
**not** the source of truth — the source of truth for conventions is the relevant
`AGENTS.md` (precedence: root > domain > README), and on-demand procedures live in
`.claude/skills/`. Do not extend or rely on these `.agent.md` files for Claude Code work.
