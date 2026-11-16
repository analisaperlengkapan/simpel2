#!/usr/bin/env python3
"""Decide which CI jobs a change actually needs (the Rust units and the e2e suites).

WHY THIS EXISTS
---------------
Every pull request used to run every Rust job and every e2e suite, whatever it
touched: a Markdown typo cost ~1,200 runner-minutes on a 20-runner pool and sat in
the queue for hours. The rule now is: run what the change can affect, and nothing
else.

HOW "CAN AFFECT" IS DECIDED
---------------------------
Nothing here is a hand-written list of dependencies — those rot (see
check-crate-ci-coverage.py for the same lesson). Three sources, all derived:

  1. The crate graph, read from the workspace's own Cargo.toml files (normal,
     dev, build and target-specific dependencies, including `workspace = true`
     path dependencies). A change to crate X affects X and every crate that
     depends on it, transitively. Change `lib-core` and everything that builds on
     it runs; change `antarmuka/portal` and only the portal does.
  2. `.github/ci-scope.json`: which crates form a CI *unit* (the granularity the
     clippy/test/build jobs run at) and how each path that is not inside a crate
     is classified.
  3. ci.yml itself: the images an e2e job needs are the ones its own
     `for img in simpel2-...; do docker pull` loop pulls, so an e2e job that starts
     needing another image changes its own scope automatically.

THE DIRECTION OF EVERY DOUBT IS "RUN IT"
----------------------------------------
A path that matches no rule (not in a crate, not classified) is treated as
`everything`. So a stale classification costs minutes, never coverage. Only the
`no_effect` globs — docs, Helm values, other workflows, guard scripts — are allowed
to skip work, and they are matched BEFORE ownership, so list them deliberately.

OUTPUTS (GitHub Actions `$GITHUB_OUTPUT`, one line each)
-------------------------------------------------------
  scope               "full" or "scoped"
  units               JSON array of affected unit names
  clippy-matrix / clippy-any
  test-matrix / test-any             (test-services tiers)
  build-backend-matrix / build-backend-any
  e2e-jobs            JSON array of e2e job ids that must run
  e2e-images          space-separated image names e2e-build-images must build+push
  e2e-services        space-separated compose services for those images

`full` is used for anything that is not a diff against a base: the nightly
schedule and workflow_dispatch — the safety net behind a classification mistake.

DEPENDENCIES
------------
Python standard library only (`tomllib`, `json`, `re`). The ARC runner image ships
no PyYAML, and a gate that cannot run on the runner is not a gate.
"""

from __future__ import annotations

import argparse
import json
import os
import posixpath
import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    import tomli as tomllib  # type: ignore

CONFIG_REL = ".github/ci-scope.json"
WORKFLOW_REL = ".github/workflows/ci.yml"

# `e2e-build-images` builds images; it is not a suite that consumes them.
E2E_NOT_A_SUITE = {"e2e-build-images"}

# A diff this long may have been truncated by the API that produced it; a
# truncated list can only hide files, so treat it as "everything changed".
FILE_LIST_LIMIT = 2900


class ScopeError(Exception):
    """The configuration contradicts the workspace; fail loudly, never guess."""


# ── globs ────────────────────────────────────────────────────────────────────
def glob_to_regex(pattern: str) -> re.Pattern[str]:
    """`**` crosses directories, `*` and `?` do not; everything else is literal."""
    out: list[str] = []
    i = 0
    while i < len(pattern):
        c = pattern[i]
        if pattern.startswith("**/", i):
            out.append("(?:.*/)?")
            i += 3
        elif pattern.startswith("**", i):
            out.append(".*")
            i += 2
        elif c == "*":
            out.append("[^/]*")
            i += 1
        elif c == "?":
            out.append("[^/]")
            i += 1
        else:
            out.append(re.escape(c))
            i += 1
    return re.compile("^" + "".join(out) + "$")


def matches_any(path: str, patterns: list[str]) -> bool:
    return any(glob_to_regex(p).match(path) for p in patterns)


# ── workspace graph ──────────────────────────────────────────────────────────
def _dep_tables(manifest: dict) -> list[dict]:
    tables = [
        manifest.get("dependencies", {}),
        manifest.get("dev-dependencies", {}),
        manifest.get("build-dependencies", {}),
    ]
    for target in manifest.get("target", {}).values():
        tables += [
            target.get("dependencies", {}),
            target.get("dev-dependencies", {}),
            target.get("build-dependencies", {}),
        ]
    return tables


def load_workspace(root: Path) -> dict[str, dict]:
    """{package name: {"dir": repo-relative dir, "deps": {workspace package names}}}."""
    top = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    ws = top.get("workspace", {})
    members: list[str] = ws.get("members", [])
    ws_deps: dict = ws.get("dependencies", {})

    manifests: dict[str, tuple[str, dict]] = {}
    for member in members:
        data = tomllib.loads((root / member / "Cargo.toml").read_text(encoding="utf-8"))
        manifests[data["package"]["name"]] = (member, data)

    graph: dict[str, dict] = {}
    for name, (member, data) in manifests.items():
        deps: set[str] = set()
        for table in _dep_tables(data):
            for key, spec in table.items():
                real = spec.get("package", key) if isinstance(spec, dict) else key
                if real in manifests:
                    deps.add(real)
                    continue
                # `foo = { workspace = true }` resolves through [workspace.dependencies],
                # where the path (and a possible rename) lives.
                if isinstance(spec, dict) and spec.get("workspace"):
                    wspec = ws_deps.get(key)
                    if isinstance(wspec, dict):
                        wreal = wspec.get("package", key)
                        if wreal in manifests:
                            deps.add(wreal)
        deps.discard(name)
        graph[name] = {"dir": member, "deps": deps}
    add_build_script_reads(root, graph)
    return graph


# A string literal that climbs out of the crate: "../../layanan/authenc/proto", "../proto".
_CLIMBING_LITERAL = re.compile(r'"((?:\.\./)+[^"\n]*|\.\.)"')


def add_build_script_reads(root: Path, graph: dict[str, dict]) -> None:
    """Record, per crate, the directories its build.rs reads from OUTSIDE the crate.

    Cargo's graph cannot see these. perlengkapan's build.rs compiles
    `../../layanan/authenc/proto/*.proto` into its gRPC clients without depending
    on any authenc crate, so a change to that proto reached perlengkapan through no
    edge and its tests were skipped. The reads are derived from the script itself:
    every string literal that climbs out of the crate and names a path that exists.
    A change under such a path seeds the crate that reads it (see `compute`).
    The directory is the grain, not the file the script happens to join onto it:
    the direction of doubt is "run more". The read is NOT turned into a dependency
    edge on a crate, because the directory need not belong to any one crate
    (`layanan/authenc` is not itself a workspace member).
    """
    for info in graph.values():
        info["reads"] = set()
        script = root / info["dir"] / "build.rs"
        if not script.is_file():
            continue
        code = "\n".join(
            line for line in script.read_text(encoding="utf-8").splitlines()
            if not line.lstrip().startswith("//")
        )
        own = info["dir"]
        for literal in _CLIMBING_LITERAL.findall(code):
            target = posixpath.normpath(posixpath.join(own, literal))
            if target.startswith("..") or target == "." or not (root / target).exists():
                continue
            if target == own or target.startswith(own + "/"):
                continue
            info["reads"].add(target)


def dependents_closure(graph: dict[str, dict], seeds: set[str]) -> set[str]:
    """`seeds` plus every package that (transitively) depends on one of them."""
    rdeps: dict[str, set[str]] = {n: set() for n in graph}
    for name, info in graph.items():
        for dep in info["deps"]:
            rdeps[dep].add(name)
    seen = set(seeds)
    stack = list(seeds)
    while stack:
        for parent in rdeps[stack.pop()]:
            if parent not in seen:
                seen.add(parent)
                stack.append(parent)
    return seen


# ── configuration ────────────────────────────────────────────────────────────
def load_config(root: Path) -> dict:
    return json.loads((root / CONFIG_REL).read_text(encoding="utf-8"))


def validate(cfg: dict, graph: dict[str, dict]) -> list[str]:
    """Contradictions between the config and the workspace (empty = consistent)."""
    problems: list[str] = []
    units: dict = cfg["units"]
    owner: dict[str, str] = {}
    for unit, spec in units.items():
        root = spec["root"].rstrip("/")
        for pkg in spec["packages"]:
            if pkg not in graph:
                problems.append(f"unit {unit}: package {pkg!r} is not a workspace member")
                continue
            if pkg in owner:
                problems.append(f"package {pkg!r} is in both {owner[pkg]!r} and {unit!r}")
            owner[pkg] = unit
            pdir = graph[pkg]["dir"]
            if not (pdir == root or pdir.startswith(root + "/")):
                problems.append(f"unit {unit}: {pkg!r} lives in {pdir}, outside the unit root {root}")
    for pkg in graph:
        if pkg not in owner:
            problems.append(f"workspace package {pkg!r} belongs to no unit (it would never be scoped in)")
    roots = sorted((spec["root"].rstrip("/"), unit) for unit, spec in units.items())
    for (ra, ua), (rb, ub) in zip(roots, roots[1:]):
        if rb == ra or rb.startswith(ra + "/"):
            problems.append(f"unit roots overlap: {ua} ({ra}) contains {ub} ({rb})")
    for kind, block in cfg["matrices"].items():
        for unit in block["units"]:
            if unit not in units:
                problems.append(f"matrices.{kind}: unknown unit {unit!r}")
    images = {k: v for k, v in cfg["images"].items() if not k.startswith("_")}
    for image, spec in images.items():
        if spec["unit"] not in units:
            problems.append(f"images.{image}: unknown unit {spec['unit']!r}")
    return problems


# ── classification ───────────────────────────────────────────────────────────
def classify(path: str, cfg: dict, graph: dict[str, dict]) -> tuple[str, object]:
    """('everything'|'e2e_only'|'no_effect', None) | ('unit', (unit, package|None))."""
    paths = cfg["paths"]
    for kind in ("everything", "e2e_only", "no_effect"):
        if matches_any(path, paths.get(kind, [])):
            return kind, None
    best_unit = None
    best_len = -1
    for unit, spec in cfg["units"].items():
        root = spec["root"].rstrip("/")
        if (path == root or path.startswith(root + "/")) and len(root) > best_len:
            best_unit, best_len = unit, len(root)
    if best_unit is None:
        return "everything", None  # unclassified: the direction of doubt is "run it"
    best_pkg = None
    best_dir = -1
    for pkg in cfg["units"][best_unit]["packages"]:
        pdir = graph[pkg]["dir"]
        if (path == pdir or path.startswith(pdir + "/")) and len(pdir) > best_dir:
            best_pkg, best_dir = pkg, len(pdir)
    return "unit", (best_unit, best_pkg)


def e2e_images_by_job(workflow_text: str) -> dict[str, list[str]]:
    """{e2e job id: images its `for img in ...; do docker pull` loop pulls}."""
    header = re.compile(r"^ {2}([A-Za-z0-9_-]+):\s*$", re.M)
    spans = [(m.group(1), m.start()) for m in header.finditer(workflow_text)]
    jobs: dict[str, list[str]] = {}
    for i, (job, start) in enumerate(spans):
        if not job.startswith("e2e-") or job in E2E_NOT_A_SUITE:
            continue
        end = spans[i + 1][1] if i + 1 < len(spans) else len(workflow_text)
        body = workflow_text[start:end]
        images: list[str] = []
        for loop in re.finditer(r"for img in ([^;\n]+);\s*do", body):
            images += [t for t in loop.group(1).replace("\\", " ").split() if t.startswith("simpel2-")]
        jobs[job] = sorted(set(images))
    return jobs


# ── the decision ─────────────────────────────────────────────────────────────
def compute(
    changed: list[str] | None,
    root: Path,
    cfg: dict | None = None,
    graph: dict[str, dict] | None = None,
    workflow_text: str | None = None,
    reason_if_full: str = "no diff to scope against",
) -> dict:
    """`changed=None` means "run everything" (schedule, dispatch, unknown diff)."""
    cfg = cfg or load_config(root)
    graph = graph or load_workspace(root)
    workflow_text = workflow_text if workflow_text is not None else (root / WORKFLOW_REL).read_text(encoding="utf-8")
    problems = validate(cfg, graph)
    if problems:
        raise ScopeError("ci-scope.json contradicts the workspace:\n  - " + "\n  - ".join(problems))

    all_units = list(cfg["units"])
    images_cfg = {k: v for k, v in cfg["images"].items() if not k.startswith("_")}
    why: dict[str, str] = {}
    everything = changed is None
    e2e_global = changed is None
    if changed is None:
        why["*"] = f"full run: {reason_if_full}"
    elif len(changed) >= FILE_LIST_LIMIT:
        everything = e2e_global = True
        why["*"] = f"{len(changed)} changed files: the list may be truncated, so everything runs"

    pkg_seeds: set[str] = set()
    unit_wide: set[str] = set()
    ignored = 0
    owner = {p: u for u, s in cfg["units"].items() for p in s["packages"]}
    for path in changed or []:
        # What a build script reads from another crate's directory (a shared .proto)
        # affects the reader whatever class the path is in.
        for pkg, info in graph.items():
            if any(path == r or path.startswith(r + "/") for r in info.get("reads", ())):
                pkg_seeds.add(pkg)
                why.setdefault(owner[pkg], f"{path} (read by the build script of {pkg})")
        kind, detail = classify(path, cfg, graph)
        if kind == "everything":
            everything = e2e_global = True
            why.setdefault("*", f"{path} affects every unit")
        elif kind == "e2e_only":
            e2e_global = True
            why.setdefault("e2e", f"{path} affects the e2e stack only")
        elif kind == "no_effect":
            ignored += 1
        else:
            unit, pkg = detail  # type: ignore[misc]
            if pkg is None:
                unit_wide.add(unit)
                why.setdefault(unit, f"{path} (not inside one crate: whole unit)")
            else:
                pkg_seeds.add(pkg)
                why.setdefault(unit, f"{path}")

    for unit in unit_wide:
        pkg_seeds.update(cfg["units"][unit]["packages"])

    if everything:
        affected_pkgs = set(graph)
    else:
        affected_pkgs = dependents_closure(graph, pkg_seeds)
    affected_units = [u for u in all_units if everything or any(owner[p] == u for p in affected_pkgs)]
    for unit in affected_units:
        if unit not in why and not everything:
            via = sorted(
                p for p in cfg["units"][unit]["packages"]
                if p in affected_pkgs and p not in pkg_seeds
            )
            dep = sorted(
                d for p in cfg["units"][unit]["packages"] for d in graph[p]["deps"] if d in affected_pkgs
            )
            why[unit] = "depends on " + ", ".join(sorted(set(dep) or set(via)))

    # e2e suites: a suite runs when it consumes an image whose unit is affected,
    # or when something that only the e2e stack reads has changed.
    job_images = e2e_images_by_job(workflow_text)
    all_images = sorted(images_cfg)
    e2e_jobs: list[str] = []
    for job, imgs in job_images.items():
        needs = imgs or all_images  # a loop we cannot read: assume it needs everything
        hit = e2e_global or any(images_cfg[i]["unit"] in affected_units for i in needs if i in images_cfg)
        if hit:
            e2e_jobs.append(job)
    e2e_images = sorted(
        {i for job in e2e_jobs for i in (job_images[job] or all_images) if i in images_cfg}
    )
    e2e_services = [images_cfg[i]["service"] for i in e2e_images]

    def matrix(kind: str, key: str) -> tuple[dict, bool]:
        block = cfg["matrices"][kind]["units"]
        entries = []
        for unit in all_units:
            if unit not in affected_units or unit not in block:
                continue
            extra = dict(block[unit])
            packages = " ".join(f"-p {p}" for p in cfg["units"][unit]["packages"])
            entry: dict = {key: unit}
            if kind != "build-backend":
                entry["packages"] = packages
            entry.update(extra)
            entries.append(entry)
        return {"include": entries}, bool(entries)

    clippy, clippy_any = matrix("clippy", "name")
    tests, test_any = matrix("test-services", "tier")
    builds, build_any = matrix("build-backend", "name")

    return {
        "scope": "full" if (changed is None or everything) else "scoped",
        "units": affected_units,
        "all_units": all_units,
        "why": why,
        "ignored": ignored,
        "changed": len(changed or []),
        "clippy-matrix": clippy,
        "clippy-any": clippy_any,
        "test-matrix": tests,
        "test-any": test_any,
        "build-backend-matrix": builds,
        "build-backend-any": build_any,
        "e2e-jobs": e2e_jobs,
        "e2e-images": e2e_images,
        "e2e-services": e2e_services,
        "e2e_all": sorted(job_images),
    }


# ── presentation ─────────────────────────────────────────────────────────────
def github_outputs(result: dict) -> list[str]:
    def js(v) -> str:
        return json.dumps(v, separators=(",", ":"))

    return [
        f"scope={result['scope']}",
        f"units={js(result['units'])}",
        f"clippy-matrix={js(result['clippy-matrix'])}",
        f"clippy-any={str(result['clippy-any']).lower()}",
        f"test-matrix={js(result['test-matrix'])}",
        f"test-any={str(result['test-any']).lower()}",
        f"build-backend-matrix={js(result['build-backend-matrix'])}",
        f"build-backend-any={str(result['build-backend-any']).lower()}",
        f"e2e-jobs={js(result['e2e-jobs'])}",
        f"e2e-images={' '.join(result['e2e-images'])}",
        f"e2e-services={' '.join(result['e2e-services'])}",
    ]


def summary_markdown(result: dict) -> str:
    lines = [
        "## CI scope",
        "",
        f"Scope: **{result['scope']}** — {result['changed']} changed file(s), "
        f"{result['ignored']} with no effect on any job.",
        "",
    ]
    if "*" in result["why"]:
        lines += [f"> {result['why']['*']}", ""]
    lines += ["| Unit | Runs | Why |", "|---|---|---|"]
    for unit in result["all_units"]:
        on = unit in result["units"]
        lines.append(f"| {unit} | {'yes' if on else 'skipped'} | {result['why'].get(unit, '') if on else ''} |")
    lines += ["", "| e2e suite | Runs |", "|---|---|"]
    for job in result["e2e_all"]:
        lines.append(f"| {job} | {'yes' if job in result['e2e-jobs'] else 'skipped'} |")
    if result["why"].get("e2e"):
        lines += ["", f"> e2e: {result['why']['e2e']}"]
    return "\n".join(lines) + "\n"


def read_changed(path: str) -> list[str]:
    text = Path(path).read_text(encoding="utf-8").strip()
    if not text:
        return []
    if text.startswith("["):
        return [str(p) for p in json.loads(text)]
    return [line.strip() for line in text.splitlines() if line.strip()]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--root", default=str(Path(__file__).resolve().parents[2]))
    ap.add_argument("--changed-files", help="file with changed paths: a JSON array or one per line")
    ap.add_argument("--full", action="store_true", help="run everything (schedule, dispatch)")
    ap.add_argument("--reason", default="no diff to scope against")
    ap.add_argument("--github-output", action="store_true", help="append outputs to $GITHUB_OUTPUT / $GITHUB_STEP_SUMMARY")
    ap.add_argument("--print", action="store_true", help="print the decision as a table")
    args = ap.parse_args(argv)

    root = Path(args.root)
    if args.full or not args.changed_files:
        changed = None
    else:
        changed = read_changed(args.changed_files)
    try:
        result = compute(changed, root, reason_if_full=args.reason)
    except ScopeError as exc:
        print(f"::error::{exc}", file=sys.stderr)
        return 1

    if args.github_output:
        out = os.environ.get("GITHUB_OUTPUT")
        if not out:
            print("::error::--github-output needs $GITHUB_OUTPUT", file=sys.stderr)
            return 2
        with open(out, "a", encoding="utf-8") as fh:
            fh.write("\n".join(github_outputs(result)) + "\n")
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            with open(summary, "a", encoding="utf-8") as fh:
                fh.write(summary_markdown(result))
    if args.print or not args.github_output:
        print(summary_markdown(result))
    return 0


if __name__ == "__main__":
    sys.exit(main())
