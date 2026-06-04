# Formal verification (Verus)

This directory holds **Verus** proofs about the cryptographic foundations used by
`lib-crypto` (Shamir secret sharing in `../src/shamir.rs`, XOR/one-time-pad
masking). They are *machine-checked proofs*, not unit tests: Verus discharges
every `requires`/`ensures`/`assert ... by(...)` obligation with the Z3 SMT
solver at CI time.

## Why it lives outside `src/`

These files use the `verus!` macro and `vstd`, which are **not** normal Cargo
dependencies. Keeping them in `verus/` (not `src/`, `tests/`, `benches/`, or
`examples/`) means Cargo never tries to compile them — `cargo build`/`clippy`/
`fmt` ignore this directory entirely. **Do not** add a `mod` for these files.

## What is proven (`secret_sharing_proof.rs`)

- **Shamir invariant:** the secret is `f(0)` — evaluating the share polynomial
  (Horner form, mirroring `evaluate_polynomial_horner`) at `x = 0` recovers
  exactly the constant term / secret.
- **`(u8, XOR)` is an abelian group** (commutative, associative, identity `0`,
  self-inverse) — the additive group underlying GF(2^8) Shamir and the OTP.
- **Additive (XOR) 2-of-2 sharing correctness:** `combine(split(secret, key))
  == secret` for every byte and key.

## Running locally

```bash
# Verus pins an exact rustc per release (see VERUS_TAG in .github/workflows/security.yml).
rustup toolchain install 1.95.0-x86_64-unknown-linux-gnu
verus lib/crypto/verus/secret_sharing_proof.rs
# → verification results:: 15 verified, 0 errors
```

CI runs this automatically: the `Verus Formal Verification` job in
`.github/workflows/security.yml` auto-detects any `verus!`-annotated file and
verifies it.
