---
name: leptos-expert
description: Develop SIMPel's Leptos 0.8 CSR/WASM frontends well — reactive model (signals/Memo/Resource/Action), Suspense/Transition, ErrorBoundary, context vs prop-drilling, component/props idioms, and the repo's feature-first conventions + lib/ui usage. Use when building/refactoring frontend components or flows, or debugging reactivity (stale views, borrow panics, lost reactivity). NOT for backend/HTTP-contract work.
---

# Leptos expert (SIMPel antarmuka)

Make SIMPel's Leptos frontends idiomatic and correct — not just scaffold a route.
**Conventions (always-active) live in `antarmuka/AGENTS.md`** (feature-first layout,
auth flow, perf) — read it first; this Skill adds the reactive depth + pitfalls.

## Reactive model (Leptos 0.8)

- **State:** `signal()` for owned state; `Memo`/derived signals for computed values
  (don't clone+recompute). Prefer `Signal::derive` over cloning state into closures.
- **Async data:** `Resource` + `<Suspense>`/`<Transition>` — **not** ad-hoc
  `spawn_local` + manual signal juggling. `Transition` to avoid flicker on refetch.
- **Mutations:** `Action` (`Action::new`) + `<ActionForm>`/`.dispatch()`; read
  `.pending()`/`.value()` for UI state.
- **Errors:** wrap fallible views in `<ErrorBoundary>`; surface `Result` to it.
- **Cross-component state:** `provide_context`/`use_context` (like `Auth::user()`),
  not deep prop-drilling. SIMPel puts `UserSession` in context at `app.rs`.

## Component idioms

- `#[component]`, `#[prop(into)]` (ergonomic conversions), `#[prop(optional)]`,
  `#[prop(default = …)]`; `Callback`/`Children`/`ChildrenFn` for composition.
- A `ChildrenFn` body must stay `Fn` — capture `Copy` handles (`StoredValue`,
  signals), not moved `String`s (see `lib/ui` `Tooltip` for the pattern).
- Keep views small; split into child `#[component]`s; stable keys in `<For key=…>`.

## Repo conventions

- **Feature-first:** put the view in `src/features/<domain>/` (page/components/api/
  state), `components/` only for dumb/shared; shared-worthy → `lib/ui`. (Details:
  `antarmuka/AGENTS.md` → "Struktur feature-first".)
- **Reuse `lib/ui`:** layout/forms/feedback/nav components, `hooks::use_auth`,
  floating primitives (`Popover`/`Tooltip`), `DataTable`+`SimpelTableClasses`.
- **DTOs from `lib-perlengkapan`** (WASM-safe) — never pull backend types.
- **No gRPC from FE:** REST/JSON only; JWT in `localStorage` (cross-tab), 401→login.
- **Build/verify:** `cargo check -p <app> --target wasm32-unknown-unknown` (fast),
  `trunk build` at milestones. Release recursion can blow the default limit — crates
  set `#![recursion_limit = "256"]`.

## Common pitfalls (debug reactivity)

- **View not updating** → you read the signal *outside* a reactive context (captured
  the value, not the signal). Read inside the view/closure, or use a derived signal.
- **Borrow/already-borrowed panic** → holding a `read()` guard across a `.set()`;
  scope the read or use `.get()`/`.with()`.
- **Lost reactivity through props** → pass `Signal<T>`/`Memo<T>`, not a snapshot `T`.
- **`<For>` re-rendering everything** → missing/unstable `key`.
- **WASM-only API on server path** → gate with `#[cfg(target_arch = "wasm32")]`;
  use `gloo_net` for fetch.

## Sub-procedure: add a feature route

Scaffold under `features/<domain>/` (page + optional api/state), import the view in
the router, wrap with the right guard layout (`PortalAuthLayout`/`AdminLayout` etc.),
verify wasm check + e2e. For the full backend side, pair with `add-backend-feature`.
