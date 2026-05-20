#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


ROOT_DIR = Path(__file__).resolve().parents[2]
ROOT_MANIFEST = ROOT_DIR / "Cargo.toml"
MODE = sys.argv[1] if len(sys.argv) > 1 else "check"


def fail_usage() -> int:
    print("Usage: check-cargo-versioning.py [check|fix]", file=sys.stderr)
    return 2


if MODE not in {"check", "fix"}:
    raise SystemExit(fail_usage())


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def is_workspace_member(manifest: Path, workspace_members: list[str]) -> bool:
    if manifest == ROOT_MANIFEST:
        return True

    relative = manifest.relative_to(ROOT_DIR).as_posix()
    for member in workspace_members:
        if relative == member or relative.startswith(member.rstrip("/") + "/"):
            return True
    return False


def root_dependency_names(root_data: dict) -> dict[str, str]:
    workspace = root_data.get("workspace", {})
    deps = workspace.get("dependencies", {})
    names: dict[str, str] = {}
    for name, value in deps.items():
        if isinstance(value, str):
            names[name] = value
        elif isinstance(value, dict) and "version" in value:
            names[name] = str(value["version"])
    return names


def package_version_is_workspace(lines: list[str]) -> bool:
    in_package = False
    for line in lines:
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        if stripped.startswith("[") and stripped != "[package]":
            in_package = False
        if in_package and stripped == "version.workspace = true":
            return True
        if in_package and re.match(r'^version\s*=\s*"', stripped):
            return False
    return False


def update_package_version(lines: list[str]) -> tuple[list[str], bool]:
    new_lines: list[str] = []
    in_package = False
    changed = False
    for line in lines:
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            new_lines.append(line)
            continue
        if stripped.startswith("[") and stripped != "[package]":
            in_package = False
        if in_package and re.match(r'^version\s*=\s*"', stripped):
            indent = line[: len(line) - len(line.lstrip())]
            new_lines.append(f"{indent}version.workspace = true\n")
            changed = True
            continue
        new_lines.append(line)
    return new_lines, changed


def parse_section_header(line: str) -> tuple[str | None, str | None]:
    stripped = line.strip()
    if not (stripped.startswith("[") and stripped.endswith("]")):
        return None, None
    if stripped.startswith("[["):
        return None, None
    inner = stripped[1:-1]
    if inner in {"dependencies", "dev-dependencies", "build-dependencies"}:
        return inner, None
    if "." in inner:
        head, tail = inner.split(".", 1)
        if head in {"dependencies", "dev-dependencies", "build-dependencies"}:
            return head, tail
    return None, None


def check_or_fix_manifest(path: Path, root_deps: dict[str, str]) -> tuple[bool, list[str]]:
    original = path.read_text()
    lines = original.splitlines(keepends=True)
    changed = False
    violations: list[str] = []
    new_lines = list(lines)

    if path == ROOT_MANIFEST:
        workspace = load_toml(path).get("workspace", {})
        workspace_package = workspace.get("package", {})
        if not workspace_package or "version" not in workspace_package:
            violations.append("::error file=Cargo.toml::Missing [workspace.package].version in root manifest")
        return False, violations

    if not package_version_is_workspace(lines):
        violations.append(
            f"::error file={path.relative_to(ROOT_DIR)}::Package version must come from the root workspace. Replace the package version with version.workspace = true."
        )
        if MODE == "fix":
            new_lines, pkg_changed = update_package_version(new_lines)
            changed = changed or pkg_changed

    section: str | None = None
    subsection: str | None = None
    for idx, line in enumerate(new_lines):
        parsed_section, parsed_subsection = parse_section_header(line)
        if parsed_section is not None or parsed_subsection is not None:
            section = parsed_section
            subsection = parsed_subsection
            continue

        if section not in {"dependencies", "dev-dependencies", "build-dependencies"}:
            continue

        if subsection is None:
            dep_match = re.match(r'^\s*([A-Za-z0-9_\-]+)\s*=\s*(.+)$', line)
            if not dep_match:
                continue
            dep_name, rhs = dep_match.group(1), dep_match.group(2)
            if dep_name not in root_deps:
                continue
            if rhs.strip().startswith('"'):
                if MODE == "check":
                    violations.append(
                        f"::error file={path.relative_to(ROOT_DIR)}::Dependency '{dep_name}' must use workspace version {root_deps[dep_name]} from the root workspace."
                    )
                else:
                    indent = line[: len(line) - len(line.lstrip())]
                    new_lines[idx] = f"{indent}{dep_name} = {{ workspace = true }}\n"
                    changed = True
                continue

            if "version = " in rhs and "workspace = true" not in rhs:
                if MODE == "check":
                    violations.append(
                        f"::error file={path.relative_to(ROOT_DIR)}::Dependency '{dep_name}' must use workspace version {root_deps[dep_name]} from the root workspace."
                    )
                else:
                    new_rhs = re.sub(r'version\s*=\s*"[^"]+"\s*,?', 'workspace = true,', rhs)
                    new_rhs = re.sub(r',\s*}$', ' }', new_rhs)
                    new_lines[idx] = f"{line[: line.index(dep_name)]}{dep_name} = {new_rhs.rstrip()}\n"
                    changed = True
                continue

        else:
            if subsection not in root_deps:
                continue
            if re.match(r'^\s*version\s*=\s*"[^"]+"', line):
                if MODE == "check":
                    violations.append(
                        f"::error file={path.relative_to(ROOT_DIR)}::Dependency '{subsection}' must use workspace version {root_deps[subsection]} from the root workspace."
                    )
                else:
                    indent = line[: len(line) - len(line.lstrip())]
                    new_lines[idx] = f"{indent}workspace = true\n"
                    changed = True

    if MODE == "fix" and changed:
        new_text = "".join(new_lines)
        if new_text != original:
            path.write_text(new_text)

    return changed, violations


def main() -> int:
    root_data = load_toml(ROOT_MANIFEST)
    workspace = root_data.get("workspace", {})
    workspace_members = workspace.get("members", [])
    root_deps = root_dependency_names(root_data)

    manifests = sorted(ROOT_DIR.rglob("Cargo.toml"))
    all_violations: list[str] = []
    for manifest in manifests:
        if manifest == ROOT_MANIFEST:
            _, violations = check_or_fix_manifest(manifest, root_deps)
            all_violations.extend(violations)
            continue
        if not is_workspace_member(manifest, workspace_members):
            continue
        _, violations = check_or_fix_manifest(manifest, root_deps)
        all_violations.extend(violations)

    if MODE == "check" and all_violations:
        for violation in all_violations:
            print(violation, file=sys.stderr)
        print("\nCargo versioning check failed.", file=sys.stderr)
        return 1

    if MODE == "check":
        print("Cargo versioning check passed: root workspace remains the source of truth.")

    if MODE == "fix":
        print("Cargo versioning auto-fix complete.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())