#!/usr/bin/env python3
"""Tests for ci-scope.py: the derivation that decides which CI jobs a change needs.

Two kinds of test, on purpose:

* Synthetic workspaces in a temp dir, so each rule (dependency closure, unit-wide
  changes, `no_effect`, unclassified paths, truncated diffs) is proven on its own
  and cannot be broken by a legitimate change to the real crate graph.
* A few assertions against the REAL repository, for the properties a person would
  be astonished to lose: docs-only runs nothing, a lockfile change runs everything,
  a portal-only change does not drag the backends in.

Run: python3 infra/scripts/test_ci_scope.py
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import textwrap
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

spec = importlib.util.spec_from_file_location("ci_scope", HERE / "ci-scope.py")
ci_scope = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci_scope)  # type: ignore[union-attr]


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(text), encoding="utf-8")


WORKFLOW = """\
jobs:
  detect:
    runs-on: x
  e2e-build-images:
    steps:
      - run: for img in simpel2-a simpel2-b; do echo; done
  e2e-uses-a:
    steps:
      - run: |
          for img in simpel2-a; do docker pull "$img"; done
  e2e-uses-b:
    steps:
      - run: |
          for img in simpel2-a simpel2-b; do docker pull "$img"; done
  e2e-unreadable:
    steps:
      - run: echo nothing to see here
  ci-summary:
    runs-on: x
"""


def make_repo(tmp: Path, members: dict[str, tuple[str, list[tuple[str, str]]]], cfg: dict) -> None:
    """members: {dir: (package name, [(dependency name, kind)])}.

    kind is `dependencies`, `dev-dependencies`, or `ws` (a `workspace = true`
    dependency, resolved through [workspace.dependencies]).
    """
    write(
        tmp / "Cargo.toml",
        "[workspace]\nmembers = [" + ", ".join(f'"{m}"' for m in members) + "]\n"
        "[workspace.dependencies]\n"
        + "".join(f'{name} = {{ path = "{m}" }}\n' for m, (name, _) in members.items()),
    )
    for member, (name, deps) in members.items():
        body = f'[package]\nname = "{name}"\nversion = "0.1.0"\n'
        for dep, kind in deps:
            if kind == "ws":
                body += f"[dependencies]\n{dep} = {{ workspace = true }}\n"
            else:
                body += f'[{kind}]\n{dep} = {{ path = "../{dep}" }}\n'
        write(tmp / member / "Cargo.toml", body)
    write(tmp / ".github/ci-scope.json", json.dumps(cfg))
    write(tmp / ".github/workflows/ci.yml", WORKFLOW)


def base_cfg() -> dict:
    return {
        "units": {
            "lib": {"root": "lib/base", "packages": ["base"]},
            "svc-a": {"root": "svc/a", "packages": ["a"]},
            "svc-b": {"root": "svc/b", "packages": ["b", "b-extra"]},
            "tool": {"root": "tool/t", "packages": ["t"]},
        },
        "matrices": {
            "clippy": {"units": {u: {"scope": "--all-targets"} for u in ("lib", "svc-a", "svc-b", "tool")}},
            "test-services": {"units": {"svc-a": {"test_args": ""}, "svc-b": {"test_args": "-- -q"}}},
            "build-backend": {"units": {"svc-a": {"package": "a"}}},
        },
        "images": {
            "simpel2-a": {"unit": "svc-a", "service": "a"},
            "simpel2-b": {"unit": "svc-b", "service": "b"},
        },
        "paths": {
            "everything": ["Cargo.toml", "Cargo.lock", ".github/ci-scope.json"],
            "e2e_only": ["docker-compose.yml", "tests/e2e/**"],
            "no_effect": ["**/*.md", "docs/**", "infra/helm/**"],
        },
    }


# base <- a <- b ; base <- b-extra (a dev-dependency); tool stands alone.
MEMBERS = {
    "lib/base": ("base", []),
    "svc/a": ("a", [("base", "ws")]),  # through [workspace.dependencies]
    "svc/b/core": ("b", [("a", "dependencies")]),
    "svc/b/extra": ("b-extra", [("base", "dev-dependencies")]),
    "tool/t": ("t", []),
}


class Synthetic(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.cfg = base_cfg()
        make_repo(self.root, MEMBERS, self.cfg)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def units(self, *changed: str) -> list[str]:
        return ci_scope.compute(list(changed), self.root)["units"]

    def test_leaf_change_affects_only_itself(self):
        self.assertEqual(self.units("tool/t/src/main.rs"), ["tool"])

    def test_dependents_are_followed_transitively(self):
        # base <- a <- b ; base <- b-extra (dev-dependency). Everything but `tool`.
        self.assertEqual(self.units("lib/base/src/lib.rs"), ["lib", "svc-a", "svc-b"])

    def test_middle_of_the_chain(self):
        self.assertEqual(self.units("svc/a/src/lib.rs"), ["svc-a", "svc-b"])

    def test_a_file_in_the_unit_but_not_in_a_crate_is_unit_wide(self):
        # svc/b/Dockerfile is under the unit root but inside no member directory
        # (the crates are svc/b/core and svc/b/extra).
        r = ci_scope.compute(["svc/b/Dockerfile"], self.root)
        self.assertEqual(r["units"], ["svc-b"])
        self.assertIn("whole unit", r["why"]["svc-b"])

    def test_crate_inside_a_unit_is_precise(self):
        # b-extra only depends on base; changing it must not drag svc-a in.
        self.assertEqual(self.units("svc/b/extra/src/lib.rs"), ["svc-b"])
        self.assertEqual(self.units("svc/b/core/src/lib.rs"), ["svc-b"])

    def test_no_effect_paths_skip_everything(self):
        r = ci_scope.compute(["README.md", "docs/x/y.txt", "infra/helm/simpel/values.yaml", "svc/a/NOTES.md"], self.root)
        self.assertEqual(r["units"], [])
        self.assertEqual(r["e2e-jobs"], [])
        self.assertEqual(r["ignored"], 4)

    def test_no_effect_is_matched_before_ownership(self):
        self.assertEqual(self.units("svc/a/README.md"), [])

    def test_unclassified_path_runs_everything(self):
        r = ci_scope.compute(["some/new/top-level/thing.txt"], self.root)
        self.assertEqual(r["scope"], "full")
        self.assertEqual(r["units"], ["lib", "svc-a", "svc-b", "tool"])

    def test_lockfile_runs_everything(self):
        self.assertEqual(len(self.units("Cargo.lock")), 4)

    def test_truncated_diff_runs_everything(self):
        files = [f"docs/{i}.txt" for i in range(ci_scope.FILE_LIST_LIMIT)]
        self.assertEqual(ci_scope.compute(files, self.root)["scope"], "full")

    def test_full_mode(self):
        r = ci_scope.compute(None, self.root, reason_if_full="schedule")
        self.assertEqual((r["scope"], len(r["units"])), ("full", 4))
        self.assertEqual(sorted(r["e2e-jobs"]), ["e2e-unreadable", "e2e-uses-a", "e2e-uses-b"])

    def test_e2e_follows_the_images_a_job_pulls(self):
        r = ci_scope.compute(["tool/t/src/main.rs"], self.root)
        # `tool` ships no image; only the job whose pull loop we cannot read is kept.
        self.assertEqual(r["e2e-jobs"], [])
        r = ci_scope.compute(["svc/b/core/src/lib.rs"], self.root)
        self.assertEqual(sorted(r["e2e-jobs"]), ["e2e-unreadable", "e2e-uses-b"])
        self.assertEqual(r["e2e-images"], ["simpel2-a", "simpel2-b"])
        self.assertEqual(r["e2e-services"], ["a", "b"])

    def test_e2e_only_paths_run_every_suite_but_no_unit(self):
        r = ci_scope.compute(["tests/e2e/x.spec.ts"], self.root)
        self.assertEqual(r["units"], [])
        self.assertEqual(sorted(r["e2e-jobs"]), ["e2e-unreadable", "e2e-uses-a", "e2e-uses-b"])

    def test_the_build_job_is_never_a_suite(self):
        self.assertNotIn("e2e-build-images", ci_scope.e2e_images_by_job(WORKFLOW))

    def test_matrices_only_hold_affected_units_with_their_extras(self):
        r = ci_scope.compute(["svc/b/core/src/lib.rs"], self.root)
        self.assertEqual([e["name"] for e in r["clippy-matrix"]["include"]], ["svc-b"])
        self.assertEqual(
            r["test-matrix"]["include"],
            [{"tier": "svc-b", "packages": "-p b -p b-extra", "test_args": "-- -q"}],
        )
        self.assertFalse(r["build-backend-any"])  # svc-b has no build-backend entry
        r = ci_scope.compute(["svc/a/src/lib.rs"], self.root)
        self.assertEqual(r["build-backend-matrix"]["include"], [{"name": "svc-a", "package": "a"}])

    def test_empty_diff_runs_nothing(self):
        r = ci_scope.compute([], self.root)
        self.assertEqual((r["units"], r["scope"], r["clippy-any"]), ([], "scoped", False))

    def test_config_that_contradicts_the_workspace_fails_loudly(self):
        cfg = base_cfg()
        cfg["units"]["tool"]["packages"] = ["t", "a"]  # `a` is already svc-a's
        write(self.root / ".github/ci-scope.json", json.dumps(cfg))
        with self.assertRaises(ci_scope.ScopeError):
            ci_scope.compute(["tool/t/src/main.rs"], self.root)

    def test_a_crate_no_unit_owns_is_an_error(self):
        cfg = base_cfg()
        cfg["units"]["tool"]["packages"] = []
        write(self.root / ".github/ci-scope.json", json.dumps(cfg))
        with self.assertRaises(ci_scope.ScopeError):
            ci_scope.compute([], self.root)


class Globs(unittest.TestCase):
    def test_double_star_crosses_directories_and_single_star_does_not(self):
        m = ci_scope.matches_any
        self.assertTrue(m("a/b/c.md", ["**/*.md"]))
        self.assertTrue(m("c.md", ["**/*.md"]))
        self.assertTrue(m("antarmuka/portal/tests/e2e/x.ts", ["antarmuka/*/tests/e2e/**"]))
        self.assertFalse(m("antarmuka/portal/src/tests/e2e/x.ts", ["antarmuka/*/tests/e2e/**"]))
        self.assertTrue(m(".github/workflows/release.yml", [".github/workflows/*.yml"]))
        self.assertFalse(m(".github/workflows/sub/x.yml", [".github/workflows/*.yml"]))
        self.assertFalse(m("Cargo.toml.bak", ["Cargo.toml"]))


class RealRepository(unittest.TestCase):
    """What the shipped configuration does to the shipped workspace."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = ci_scope.load_workspace(ROOT)
        cls.cfg = ci_scope.load_config(ROOT)

    def run_scope(self, *changed: str) -> dict:
        return ci_scope.compute(list(changed), ROOT, self.cfg, self.graph)

    def test_config_and_workspace_agree(self):
        self.assertEqual(ci_scope.validate(self.cfg, self.graph), [])

    def test_every_e2e_suite_pull_loop_is_readable(self):
        text = (ROOT / ci_scope.WORKFLOW_REL).read_text(encoding="utf-8")
        for job, images in ci_scope.e2e_images_by_job(text).items():
            self.assertTrue(images, f"{job}: no `for img in simpel2-...` loop found; it would always run")
            for image in images:
                self.assertIn(image, self.cfg["images"], f"{job} pulls {image}, which ci-scope.json does not know")

    def test_docs_only_runs_nothing(self):
        r = self.run_scope("README.md", "docs/E2E_TEST_GUIDE.md", "infra/helm/simpel/values-staging.yaml")
        self.assertEqual((r["units"], r["e2e-jobs"]), ([], []))

    def test_lockfile_runs_everything(self):
        r = self.run_scope("Cargo.lock")
        self.assertEqual(r["units"], list(self.cfg["units"]))
        self.assertEqual(sorted(r["e2e-jobs"]), sorted(r["e2e_all"]))

    def test_portal_only_does_not_drag_the_backends_in(self):
        r = self.run_scope("antarmuka/portal/src/lib.rs")
        self.assertEqual(r["units"], ["antarmuka-portal"])
        self.assertNotIn("e2e-secreton-bootstrap", r["e2e-jobs"])
        # exactly the suites whose own pull loop names the portal image
        text = (ROOT / ci_scope.WORKFLOW_REL).read_text(encoding="utf-8")
        wants_portal = {j for j, imgs in ci_scope.e2e_images_by_job(text).items() if "simpel2-portal" in imgs}
        self.assertEqual(set(r["e2e-jobs"]), wants_portal)

    def test_a_shared_lib_reaches_what_builds_on_it(self):
        r = self.run_scope("lib/core/src/lib.rs")
        self.assertIn("layanan-integrasi", r["units"])
        self.assertIn("antarmuka-portal", r["units"])

    def test_every_workspace_crate_change_scopes_in_its_own_unit(self):
        for pkg, info in self.graph.items():
            r = self.run_scope(f"{info['dir']}/src/lib.rs")
            unit = next(u for u, s in self.cfg["units"].items() if pkg in s["packages"])
            self.assertIn(unit, r["units"], f"a change in {pkg} must run its own unit {unit}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
