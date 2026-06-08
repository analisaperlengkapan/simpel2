---
name: verus-formal-verification
description: Write, run, or debug Verus machine-checked proofs for SIMPel's cryptographic/security-sensitive Rust (lib-crypto Shamir, OTP/XOR, key handling). Use when adding a `verus!` proof, the "Verus Formal Verification" CI job fails, or a `requires`/`ensures`/`assert ... by(...)` obligation won't discharge. NOT for ordinary unit tests (use `cargo test`).
---

# Verus formal verification (SIMPel)

Verus discharges `requires`/`ensures`/`assert ... by(...)` obligations with the Z3
SMT solver at CI time — machine-checked proofs, not tests. SIMPel's first proof is
`lib/crypto/verus/shamir_proof.rs` (Shamir `secret == f(0)`, `(u8, XOR)` abelian
group, 2-of-2 XOR sharing correctness).

**Read first (don't duplicate):** `lib/crypto/verus/README.md` (conventions, what's
proven, gotchas) and the `project-verus-formal-verification` memory.

## Conventions (always-active facts live in `lib/AGENTS.md`)
- Proof file = `<crate>/verus/<source>_proof.rs` (e.g. `verus/shamir_proof.rs`
  proves `src/shamir.rs`). **Outside** `src/`/`tests/`/`benches/`/`examples/` so
  Cargo ignores it — **never** add a `mod` for it.
- **Do NOT inline `verus!` into `src/`**: it forces the crate onto pinned rustc
  1.95 + `vstd`, breaking the normal stable build, and the proof is an abstract
  model (int / GF(2)), not the real 256-bit `Scalar`. Add a
  `// FORMAL VERIFICATION:` comment in the source pointing at the proof instead.
- CI (`.github/workflows/security.yml` job **Verus Formal Verification**)
  auto-detects any `*.rs` containing `verus!` and verifies it; advisory (no `verus!`
  anywhere ⇒ green no-op). No workflow edits needed to add a proof.

## Toolchain (Verus is NOT self-contained)
Each Verus release pins an exact rustc and shells out to `rustup run`. Current
pin: **Verus `release/0.2026.05.31.5dd6d83`** needs **rustc 1.95.0**. Releases are
date-tagged `.zip`s (`release/0.YYYY.MM.DD.hash`). Bump rustc in lockstep with the
tag (see `VERUS_TAG`/`setup-rust toolchain: "1.95.0"` in security.yml).

Run locally:
```bash
rustup toolchain install 1.95.0
# download + unzip the matching Verus release, then:
RUSTUP_TOOLCHAIN=1.95.0 ./verus-x86-linux/verus path/to/<name>_proof.rs
```

## Writing a proof (subset)
- Wrap everything in `verus! { ... }`. Model with `spec fn` (pure, math-int),
  prove with `proof fn` carrying `requires`/`ensures`.
- Mirror the real algorithm's *structure* in `spec` (e.g. Horner form for
  polynomial eval) so the invariant is meaningful.
- `assert(P) by(bit_vector);` for fixed-width bit facts (XOR self-inverse, u8
  masking); `by(nonlinear_arith)` for multiplication/modular facts.

## Diagnostics (when it won't verify)
- **postcondition not satisfied** → the `ensures` isn't implied; add intermediate
  `assert`s to find where the chain breaks; check `spec` actually models the code.
- **recommends not met** → a `spec fn` precondition (e.g. index in range) is
  unproven at the call site; strengthen `requires`.
- **definition not unfolded** → `open spec fn` or `reveal(f);` so Z3 sees the body.
- **extensional equality** → use `=~=` for seqs/maps instead of `==`.
- **bit-vector vs nonlinear** → pick the right `by(...)`; `bit_vector` can't reason
  about `*`/`/`, `nonlinear_arith` can't about bitwise ops. Split the lemma.
- CI flake (toolchain/zip download) is infra, not the proof — re-run; bump the pin
  only when intentionally upgrading Verus.
