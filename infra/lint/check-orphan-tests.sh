#!/usr/bin/env bash
#
# Orphaned-test guard.
#
# Cargo only turns `tests/*.rs` into test targets. Files in *subdirectories* of
# `tests/` are compiled only if some target declares them (`mod integration;`,
# `#[path = "..."] mod x;`, or an explicit `[[test]] path =` in Cargo.toml).
# A subdirectory nobody declares is invisible: it never compiles, never runs,
# and no gate ever notices it rotting.
#
# That is not hypothetical. layanan/secreton/tests/{integration,unit,security,
# performance,common} held 8,640 lines in exactly this state (#103/#104). It had
# decayed three ways at once, all silently:
#   - syntax errors (an automated edit stripped `}` from json!/format! literals),
#   - calls to APIs refactored long ago (create_lease: 11 args -> 1 request struct),
#   - reads of fields that are now correctly private.
# One file's header advertised "verifies all 16 secrets engines are functional"
# while asserting `assert!(r.is_ok() || r.is_err())` — a tautology, in a file that
# had never once been built.
#
# The rule: if a .rs file under a tests/ subdirectory is not reachable from any
# target, it is dead weight pretending to be coverage. Delete it, or wire it up.
#
# Waiver: add the file to ORPHAN_ALLOWLIST below with a reason.

set -euo pipefail

fail=0
declare -a ORPHAN_ALLOWLIST=(
  # "path/to/tests/helpers/fixture.rs"  # reason
)

is_allowlisted() {
  local needle="$1"
  for entry in "${ORPHAN_ALLOWLIST[@]:-}"; do
    [[ "$entry" == "$needle" ]] && return 0
  done
  return 1
}

# Every tests/ dir that belongs to a crate (has a sibling Cargo.toml).
while IFS= read -r tests_dir; do
  crate_dir="$(dirname "$tests_dir")"
  [[ -f "$crate_dir/Cargo.toml" ]] || continue

  # Text of every root-level test target plus the manifest: anything that could
  # declare a submodule.
  declarations=""
  shopt -s nullglob
  for root_test in "$tests_dir"/*.rs; do
    declarations+="$(cat "$root_test")"$'\n'
  done
  declarations+="$(cat "$crate_dir/Cargo.toml")"$'\n'
  shopt -u nullglob

  while IFS= read -r orphan; do
    rel="${orphan#./}"
    is_allowlisted "$rel" && continue

    base="$(basename "$orphan" .rs)"
    parent="$(basename "$(dirname "$orphan")")"

    # Reachable if some target names the directory as a module, references the
    # file by #[path], or the manifest points a target at it.
    if grep -qE "(^|[^[:alnum:]_])mod[[:space:]]+${parent}[[:space:]]*;" <<<"$declarations" \
    || grep -qF "$parent/$base.rs" <<<"$declarations" \
    || grep -qE "(^|[^[:alnum:]_])mod[[:space:]]+${base}[[:space:]]*;" <<<"$declarations"; then
      continue
    fi

    echo "ORPHAN: $rel — under tests/ but no target declares it, so it never compiles or runs."
    fail=1
  done < <(find "$tests_dir" -mindepth 2 -name '*.rs' -print)
done < <(find . -type d -name tests -not -path '*/target/*' -not -path '*/node_modules/*' -print)

if [[ "$fail" -ne 0 ]]; then
  cat <<'EOF'

Each file above is unreachable from any cargo target. It is not tested code —
it is code that merely looks tested. Either delete it, or make it reachable:
  - move it to tests/<name>.rs (cargo auto-discovers root-level files), or
  - declare it from a root-level target (`mod <dir>;` / `#[path = "..."] mod x;`), or
  - add an explicit [[test]] entry in Cargo.toml.
Then confirm it actually compiles and passes before relying on it.
EOF
  exit 1
fi

echo "OK: no orphaned test files."
