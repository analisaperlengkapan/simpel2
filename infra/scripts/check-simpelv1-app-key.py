#!/usr/bin/env python3
"""Fail when a simpelv1 `APP_KEY` literal is not a usable AES-256 key.

Laravel's `Encrypter` accepts only 16- or 32-byte keys (AES-128/256). An
`APP_KEY` of any other length makes EVERY request 500 with

    RuntimeException - Unsupported cipher or incorrect key length.

and it fails at container boot, before any application code runs -- so it
looks like a broken app rather than a bad env var.

That is exactly what happened: the shared e2e key used by CI, `run-e2e.sh` and
the agent setup script decoded to **30** bytes ("testkey32bytesssssssssssssssss"
-- 30 characters, despite the name). Every e2e job that booted simpelv1 got
500s from nginx+php-fpm, and the cross-app SSO job never passed. It survived
because the PHPUnit job uses `phpunit.xml`'s own key, which IS 32 bytes, so the
one job that exercised v1 code never touched the broken literal.

Why a guard rather than a fix alone: the value is a hand-written base64 literal
copy-pasted in 20+ places (ci.yml env blocks, the compose harness, setup.sh).
Nothing decoded it, and a literal that "looks like a key" is invisible to every
other check -- YAML, actionlint, `compose config` and Laravel's own config cache
all accept it.

WHAT THIS CHECKS: every `APP_KEY: base64:...` / `SIMPELV1_APP_KEY=base64:...`
literal in the files that boot simpelv1 decodes to 16 or 32 bytes.

WHAT IT DOES NOT CHECK: whether the key is SECRET, or whether production values
in Secreton are valid -- those are not in the repo. It only reads literals that
are.
"""

from __future__ import annotations

import base64
import binascii
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# Files that carry a simpelv1 APP_KEY literal. Kept explicit rather than
# walking the tree: a glob would also pick up docs/ADR prose and the
# `REPLACE_ME` placeholders, and a guard that cries wolf gets disabled.
FILES = [
    ".github/workflows/ci.yml",
    "infra/scripts/run-e2e.sh",
    ".openhands/setup.sh",
]

# `APP_KEY: base64:XXXX` (YAML) and `SIMPELV1_APP_KEY=base64:XXXX` (shell/env).
# The value stops at a quote, whitespace, or end of line.
PATTERN = re.compile(
    r"""(?:SIMPELV1_)?APP_KEY\s*[:=]\s*["']?base64:([A-Za-z0-9+/=]+)"""
)

VALID_LENGTHS = (16, 32)


def main() -> int:
    problems: list[str] = []
    checked = 0

    for rel in FILES:
        path = REPO / rel
        if not path.exists():
            problems.append(f"{rel}: file missing (guard is stale)")
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for lineno, line in enumerate(text.splitlines(), start=1):
            for match in PATTERN.finditer(line):
                value = match.group(1)
                checked += 1
                try:
                    raw = base64.b64decode(value, validate=True)
                except (binascii.Error, ValueError):
                    problems.append(
                        f"{rel}:{lineno}: APP_KEY is not valid base64 "
                        f"(base64:{value})"
                    )
                    continue
                if len(raw) not in VALID_LENGTHS:
                    problems.append(
                        f"{rel}:{lineno}: APP_KEY decodes to {len(raw)} bytes, "
                        f"but Laravel needs {VALID_LENGTHS[0]} or "
                        f"{VALID_LENGTHS[1]} -- every request would 500 with "
                        f"'Unsupported cipher or incorrect key length'"
                    )

    if checked == 0:
        # No literal found at all means the extraction broke (renamed file,
        # reformatted YAML). Silence would read as success.
        problems.append(
            "no simpelv1 APP_KEY literal found in "
            + ", ".join(FILES)
            + " -- guard is not looking at what it thinks it is"
        )

    if problems:
        print("simpelv1 APP_KEY check FAILED:\n")
        for problem in problems:
            print(f"  - {problem}")
        print(
            "\nGenerate a valid key with: php artisan key:generate --show"
            "\n(or, for a throwaway test key: "
            "python3 -c \"import base64;print('base64:'+"
            "base64.b64encode(b'x'*32).decode())\")"
        )
        return 1

    print(f"simpelv1 APP_KEY check OK ({checked} literal(s), all valid lengths)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
