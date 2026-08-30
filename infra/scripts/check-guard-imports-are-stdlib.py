#!/usr/bin/env python3
"""A guard that CI cannot import is a guard that does not run.

The ARC runner image (`infra/helm/arc/runner-image/Dockerfile`) ships a bare
Python. `import yaml` therefore succeeds on every developer machine and fails on
the runner — the script exits non-zero for a reason that has nothing to do with
what it checks, and the finding it exists to report never gets made.

This has now cost time three times, each time in this same job:

  * `Workflow Syntax (actionlint)` reported all eight workflows as "Invalid
    YAML" for 40+ days. The workflows were fine; the fallback `python3 -c
    "import yaml"` was not, and its failure was read as the files' failure.
  * `bc` and `yq` produced `exit 127` in later steps of the same job.
  * `check-pr-workflows-run-on-every-pr.py` (the guard that introduced this
    file) failed its very first CI run on `import yaml`, having passed locally.

Which is why the other guards in this directory parse YAML with regexes rather
than with a library — a choice that looks arbitrary until you have watched this
happen.

The rule: a script CI invokes may import only the standard library. `sys`
supplies the module list at runtime, so it is the interpreter CI actually uses
that decides, not a list maintained here. A genuine third-party need goes in
`PERMITTED` **and** in the runner image, together.

Run: python3 infra/scripts/check-guard-imports-are-stdlib.py
     python3 infra/scripts/check-guard-imports-are-stdlib.py --self-test
"""

from __future__ import annotations

import ast
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github/workflows"
SCRIPTS = ROOT / "infra/scripts"

# module -> reason it is available on the runner. Adding one here without also
# installing it in the runner image reintroduces exactly this failure.
PERMITTED: dict[str, str] = {}

# Which scripts CI runs, derived from the workflows rather than listed here: a
# hand-kept list is what stops covering the thing it was written for.
INVOKE_RE = re.compile(r"infra/scripts/([A-Za-z0-9_.-]+\.py)")


def invoked_scripts() -> set[str]:
    names: set[str] = set()
    for wf in WORKFLOWS.glob("*.y*ml"):
        names |= set(INVOKE_RE.findall(wf.read_text()))
    return names


IMPORT_ERRORS = {"ImportError", "ModuleNotFoundError"}


def _modules(nodes) -> set[str]:
    found: set[str] = set()
    for node in nodes:
        for sub in ast.walk(node):
            if isinstance(sub, ast.Import):
                for a in sub.names:
                    found.add(a.name.split(".")[0])
            elif isinstance(sub, ast.ImportFrom):
                # `from . import x` has no module; relative = a local file.
                if sub.level == 0 and sub.module:
                    found.add(sub.module.split(".")[0])
    return found


def _catches_import_error(handler: ast.ExceptHandler) -> bool:
    t = handler.type
    names = []
    if isinstance(t, ast.Name):
        names = [t.id]
    elif isinstance(t, ast.Tuple):
        names = [e.id for e in t.elts if isinstance(e, ast.Name)]
    return bool(names) and all(n in IMPORT_ERRORS for n in names)


def imports_of(source: str) -> tuple[set[str], set[str]]:
    """(required, fallback) top-level module names.

    A *fallback* is an import inside an `except ImportError` handler whose
    `try` body imports only standard-library modules — the stdlib-first idiom

        try:
            import tomllib
        except ModuleNotFoundError:
            import tomli as tomllib

    which two guards here already use and which cannot fail on a runner whose
    Python has the stdlib module. An import inside such a handler whose try
    body reaches for something non-stdlib is NOT a fallback: that is two
    third-party dependencies wearing one coat, and it is reported.
    """
    tree = ast.parse(source)
    fallback: set[str] = set()
    skip: list[ast.AST] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Try):
            continue
        try_mods = _modules(node.body)
        if not try_mods or not try_mods <= sys.stdlib_module_names:
            continue
        for h in node.handlers:
            if _catches_import_error(h):
                fallback |= _modules(h.body)
                skip.extend(h.body)

    skipped_ids = {id(n) for parent in skip for n in ast.walk(parent)}
    required: set[str] = set()
    for node in ast.walk(tree):
        if id(node) in skipped_ids:
            continue
        if isinstance(node, ast.Import):
            for a in node.names:
                required.add(a.name.split(".")[0])
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
            required.add(node.module.split(".")[0])
    return required - fallback, fallback


def problems_in(name: str, source: str, local_modules: set[str]) -> list[str]:
    out = []
    required, _fallback = imports_of(source)
    for mod in sorted(required):
        if mod in sys.stdlib_module_names or mod in local_modules or mod in PERMITTED:
            continue
        out.append(
            f"{name} imports {mod!r}, which is not in the standard library. The "
            f"ARC runner image ships a bare Python, so this passes locally and "
            f"exits non-zero on the runner — reporting a failure that is not the "
            f"one the guard checks for. Parse it with the standard library, or "
            f"add {mod!r} to PERMITTED *and* to the runner image."
        )
    return out


def scan() -> list[str]:
    invoked = invoked_scripts()
    local = {p.stem for p in SCRIPTS.glob("*.py")}
    out: list[str] = []
    for name in sorted(invoked):
        path = SCRIPTS / name
        if not path.exists():
            out.append(f"{name} is invoked by a workflow but does not exist in infra/scripts/")
            continue
        out += problems_in(name, path.read_text(), local)
    return out


def main() -> int:
    found = scan()
    for f in found:
        print(f"::error::{f}")
    if found:
        return 1
    invoked = invoked_scripts()
    fallbacks: set[str] = set()
    for name in invoked:
        path = SCRIPTS / name
        if path.exists():
            fallbacks |= imports_of(path.read_text())[1]
    print(
        f"check-guard-imports-are-stdlib: {len(invoked)} script(s) invoked from "
        f"workflows, all importing only the standard library "
        f"({len(PERMITTED)} permitted, "
        f"{len(fallbacks)} stdlib-first fallback(s): {', '.join(sorted(fallbacks)) or 'none'})"
    )
    return 0


def _canaries() -> int:
    failures = 0
    local: set[str] = set()

    def expect(label: str, source: str, should_fail: bool) -> None:
        nonlocal failures
        ok = bool(problems_in("canary.py", source, local)) == should_fail
        print(f"  {'ok  ' if ok else 'FAIL'} {label}")
        if not ok:
            failures += 1

    expect("the invoked scripts as committed pass", "", False)
    expect("detects: the exact regression — import yaml", "import yaml\n", True)
    expect("detects: `from yaml import safe_load`", "from yaml import safe_load\n", True)
    expect("detects: an import nested inside a function", "def f():\n    import requests\n", True)
    expect(
        "tolerates: stdlib imports, including dotted ones",
        "import re, sys\nfrom pathlib import Path\nimport xml.etree.ElementTree as ET\n",
        False,
    )
    expect(
        "tolerates: the stdlib-first fallback two guards already use",
        "try:\n    import tomllib\nexcept ModuleNotFoundError:\n    import tomli as tomllib\n",
        False,
    )
    expect(
        "detects: a fallback whose try half is ALSO third-party",
        "try:\n    import ujson\nexcept ImportError:\n    import simplejson\n",
        True,
    )
    expect(
        "detects: a bare `except:` used to smuggle a third-party import",
        "try:\n    import json\nexcept:\n    import ujson\n",
        True,
    )
    # The real scan must agree with the canaries, or the canaries are theatre.
    real = scan()
    ok = not real
    print(f"  {'ok  ' if ok else 'FAIL'} the real repository scan is clean")
    if not ok:
        failures += 1
    return failures


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(_canaries())
    sys.exit(main())
