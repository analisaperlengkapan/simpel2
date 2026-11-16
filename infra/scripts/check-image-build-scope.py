#!/usr/bin/env python3
"""Keep two promises about the image builds that nothing else enforces.

1. A CHANGE THAT CAN BREAK AN IMAGE BUILD MUST REACH AN IMAGE BUILD.
   Pull requests build the six images once, in `e2e-build-images`, which runs when
   the `rust-frontend`, `rust-backend` or `e2e` path filter fires. An image reads
   more than its own directory: the portal and perlengkapan Dockerfiles `COPY . .`
   and then run `infra/scripts/*` inside the build, and the root `.dockerignore`
   decides what every context holds. A change to one of those files breaks the
   image without touching any Rust, and with no filter listing it, CI is green and
   main breaks.

   The list of such files is DERIVED here, from the Dockerfiles themselves: the
   Dockerfile, `.dockerignore` (for a root context), every `COPY` source from the
   build context, and every `/workspace/<path>` a RUN/WORKDIR line reads. Each must
   match a pattern of the filters that gate `e2e-build-images`. A hand-written list
   would be the same mistake again: correct on the day it was written, silent on
   the day a Dockerfile starts running one more script.

2. A RELEASE IMAGE IS BUILT FROM SOURCE AND NOTHING ELSE.
   The BuildKit cache lives in the in-cluster registry, which speaks plain HTTP and
   has no authentication. Whoever can reach it can replace a cached layer, and a
   build that imports the cache would put that layer into the image that gets
   promoted and deployed. ci.yml may use the cache (its images are never
   deployed); release.yml must not.

STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq, so the workflow and
the filter file are scanned as text, as in the sibling guards.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CI = ROOT / ".github" / "workflows" / "ci.yml"
RELEASE = ROOT / ".github" / "workflows" / "release.yml"
FILTERS = ROOT / ".github" / "actions" / "detect-changes" / "action.yml"

# The filters whose union decides whether `e2e-build-images` runs on a PR
# (see its `if:` in ci.yml). If that gate changes, change this and re-read it.
GATING_FILTERS = ("rust-frontend", "rust-backend", "e2e")

# `- { image: antarmuka-portal, dockerfile: antarmuka/portal/Dockerfile, context: . }`
MATRIX_RE = re.compile(
    r"-\s*\{\s*image:\s*([\w-]+)\s*,\s*dockerfile:\s*([\w./-]+)\s*,\s*context:\s*([\w./-]+)\s*\}"
)
WORKSPACE_REF_RE = re.compile(r"/workspace/([A-Za-z0-9_./-]+)")

# Anything that makes a release build read a shared, unauthenticated store.
RELEASE_FORBIDDEN = (
    (re.compile(r"type=registry"), "a registry cache (`type=registry`)"),
    (re.compile(r"buildkitd-config"), "a BuildKit config that trusts the plain-HTTP registry"),
    (re.compile(r"172\.15\.10\.252"), "the in-cluster registry's address"),
    (re.compile(r"cache-registry"), "the `cache-registry` input of the docker-build action"),
)


def code_lines(text: str) -> list[str]:
    """Lines with comments dropped: a comment may name what it forbids."""
    return [ln for ln in text.splitlines() if not ln.lstrip().startswith("#")]


def glob_to_regex(pattern: str) -> re.Pattern[str]:
    out, i = "", 0
    while i < len(pattern):
        if pattern.startswith("**/", i):
            out += "(?:.*/)?"
            i += 3
        elif pattern.startswith("**", i):
            out += ".*"
            i += 2
        elif pattern[i] == "*":
            out += "[^/]*"
            i += 1
        else:
            out += re.escape(pattern[i])
            i += 1
    return re.compile(f"^{out}$")


def filter_patterns(filters_text: str, names: tuple[str, ...]) -> dict[str, list[str]]:
    """`name:` followed by `- 'glob'` lines, inside the paths-filter block."""
    found: dict[str, list[str]] = {}
    current: str | None = None
    for line in filters_text.splitlines():
        m = re.match(r"^ {10}([A-Za-z0-9_-]+):\s*$", line)
        if m:
            current = m.group(1) if m.group(1) in names else None
            if current:
                found[current] = []
            continue
        m = re.match(r"^ {12}-\s*'([^']+)'\s*$", line)
        if m and current:
            found[current].append(m.group(1))
    return found


def build_inputs(dockerfile: str, text: str, context: str) -> set[str]:
    """Repo paths an image build reads, derived from its Dockerfile."""
    inputs = {dockerfile}
    if context == ".":
        inputs.add(".dockerignore")
    for line in code_lines(text):
        stripped = line.strip()
        if stripped.upper().startswith(("COPY ", "ADD ")):
            if "--from=" in stripped:
                continue  # copies a build stage's output, not a repo file
            parts = [p for p in stripped.split()[1:] if not p.startswith("--")]
            for src in parts[:-1]:  # the last word is the destination
                src = src.removeprefix("./")
                if src not in {".", ""}:
                    inputs.add(src)
            continue
        for ref in WORKSPACE_REF_RE.findall(line):
            inputs.add(ref.rstrip("/"))
    return inputs


def check(ci_text: str, filters_text: str, release_text: str, dockerfiles: dict[str, str]) -> list[str]:
    problems: list[str] = []
    matrix = MATRIX_RE.findall(ci_text)
    if not matrix:
        return ["::error file=.github/workflows/ci.yml::no build-docker matrix entries found: this guard measures nothing"]

    patterns = filter_patterns(filters_text, GATING_FILTERS)
    for name in GATING_FILTERS:
        if not patterns.get(name):
            problems.append(f"::error file=.github/actions/detect-changes/action.yml::filter `{name}` not found or empty")
    compiled = [glob_to_regex(p) for name in GATING_FILTERS for p in patterns.get(name, [])]

    for image, dockerfile, context in matrix:
        text = dockerfiles.get(dockerfile)
        if text is None:
            problems.append(f"::error file=.github/workflows/ci.yml::`{image}` builds {dockerfile}, which does not exist")
            continue
        for path in sorted(build_inputs(dockerfile, text, context)):
            # a directory input is covered when anything under it is
            if not any(rx.match(path) or rx.match(path + "/x") for rx in compiled):
                problems.append(
                    f"::error file=.github/actions/detect-changes/action.yml::image `{image}` reads `{path}` "
                    f"({dockerfile}), but no filter that gates e2e-build-images "
                    f"({', '.join(GATING_FILTERS)}) matches it: a change there reaches no image build. "
                    "Add it to the `e2e` filter."
                )

    for rx, what in RELEASE_FORBIDDEN:
        for line in code_lines(release_text):
            if rx.search(line):
                problems.append(
                    f"::error file=.github/workflows/release.yml::release.yml uses {what}: "
                    "the cache registry is unauthenticated plain HTTP, so a release image would "
                    f"be built from layers anyone on the network could have replaced ({line.strip()[:80]})"
                )
                break
    return problems


def self_test(ci_text: str, filters_text: str, release_text: str, dockerfiles: dict[str, str]) -> int:
    """Each mutation below is a mistake this guard exists to catch."""
    failures = 0

    def expect(description: str, needle: str, *, ci: str | None = None, filters: str | None = None,
               release: str | None = None, docker: dict[str, str] | None = None) -> None:
        nonlocal failures
        found = check(ci if ci is not None else ci_text, filters if filters is not None else filters_text,
                      release if release is not None else release_text,
                      docker if docker is not None else dockerfiles)
        if any(needle in p for p in found):
            print(f"  ok   detects: {description}")
        else:
            print(f"::error::self-test FAILED — guard did not detect: {description}")
            failures += 1

    for path in (".dockerignore", "infra/scripts/csp-script-hashes.py"):
        line = f"            - '{path}'\n"
        if line in filters_text:
            expect(f"`{path}` drops out of the e2e filter", path, filters=filters_text.replace(line, "", 1))
        else:
            print(f"::error::self-test is stale: `{path}` is not in the e2e filter in the expected form")
            failures += 1

    some = next(iter(dockerfiles))
    mutated = dict(dockerfiles)
    mutated[some] = dockerfiles[some] + "\nRUN /workspace/infra/scripts/brand-new-step.sh\n"
    expect("a Dockerfile starts running a script no filter lists", "brand-new-step.sh", docker=mutated)

    expect("release.yml starts importing the registry cache", "type=registry",
           release=release_text + "\n          cache-from: type=registry,ref=host/cache/x:main\n")
    expect("release.yml trusts the plain-HTTP registry", "buildkitd-config",
           release=release_text + "\n          buildkitd-config: .github/buildkitd.toml\n")
    expect("release.yml names the cache registry", "172.15.10.252",
           release=release_text + "\n  CACHE_REGISTRY: 172.15.10.252:32000\n")
    return failures


def main() -> int:
    ci_text = CI.read_text(encoding="utf-8")
    filters_text = FILTERS.read_text(encoding="utf-8")
    release_text = RELEASE.read_text(encoding="utf-8")
    dockerfiles = {
        dockerfile: (ROOT / dockerfile).read_text(encoding="utf-8")
        for _, dockerfile, _ in MATRIX_RE.findall(ci_text)
        if (ROOT / dockerfile).is_file()
    }
    if self_test(ci_text, filters_text, release_text, dockerfiles):
        return 1
    problems = check(ci_text, filters_text, release_text, dockerfiles)
    for problem in problems:
        print(problem)
    if problems:
        print(f"\n{len(problems)} image-build scoping problem(s) found.", file=sys.stderr)
        return 1
    inputs = sum(len(build_inputs(d, t, ".")) for d, t in dockerfiles.items())
    print(f"Image builds are reachable: {len(dockerfiles)} Dockerfiles, {inputs} derived inputs all match a "
          f"gating filter; release.yml reads no shared build cache.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
