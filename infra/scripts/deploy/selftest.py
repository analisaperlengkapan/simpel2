"""Self-test for pipeline.py (`pipeline.py --self-test`).

The pipeline is run as a real subprocess with `helm`, `kubectl`, `gh` and `git` replaced by
shims on PATH and a local HTTP server standing in for the ingress. That tests the command
lines the pipeline really builds, and the part that matters most: what it does when a step
fails (no upgrade after a bad dump, rollback after a bad verification, a person called when
the rollback itself fails).

Every scenario asserts both what happened and what did NOT happen. The mutation list at the
bottom of the PR description names the line each check was proven to catch.
"""
from __future__ import annotations

import http.server
import json
import os
import re
import socketserver
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

CANARY = "CANARY-SECRET-0123456789"
SHIM = """#!{python}
import json, os, sys
tool = os.path.basename(sys.argv[0])
argv = " ".join(sys.argv[1:])
stdin = ""
if "-i" in sys.argv and "exec" in sys.argv:
    stdin = sys.stdin.read()
scn = json.load(open(os.environ["FAKE_SCENARIO"]))
with open(os.environ["FAKE_LOG"], "a") as fh:
    fh.write(tool + " " + argv + (" <stdin %d bytes>" % len(stdin) if stdin else "") + "\\n")
for i, rule in enumerate(scn.get(tool, [])):
    marker = os.path.join(os.environ["FAKE_DIR"], "marker-rolled")
    if rule.get("when") == "rolled" and not os.path.exists(marker):
        continue
    if rule.get("when") == "not-rolled" and os.path.exists(marker):
        continue
    if all(m in argv for m in rule["match"]) and not any(n in argv for n in rule.get("not", [])):
        seq = rule.get("seq")
        if seq is not None:
            cf = os.path.join(os.environ["FAKE_DIR"], "count-%s-%d" % (tool, i))
            n = int(open(cf).read()) if os.path.exists(cf) else 0
            open(cf, "w").write(str(n + 1))
            rule = seq[min(n, len(seq) - 1)]
        for eff in rule.get("effects", []):
            if eff == "mark-rolled":
                open(marker, "w").write("1")
            if eff == "fix-http":
                try:
                    os.unlink(os.path.join(os.environ["FAKE_DIR"], "broken"))
                except OSError:
                    pass
            if eff == "check-overlay":
                path = sys.argv[sys.argv.index("-f", sys.argv.index("-f") + 1) + 1]
                ok = oct(os.stat(path).st_mode & 0o777) == "0o600" and {canary!r} in open(path).read()
                open(os.path.join(os.environ["FAKE_DIR"], "overlay-ok"), "w").write("yes" if ok else "no")
        sys.stdout.write(rule.get("out", ""))
        sys.stderr.write(rule.get("err", ""))
        sys.exit(rule.get("rc", 0))
sys.stderr.write("UNEXPECTED CALL: " + tool + " " + argv + "\\n")
sys.exit(99)
"""

DEPLOYED = [{"revision": 6, "status": "superseded"}, {"revision": 7, "status": "deployed"}]


def deploy(name, ready, want=1, image="ghcr.io/o/simpelv2-x:v0.1.0-rc52", kind="Deployment"):
    return {
        "kind": kind,
        "metadata": {"name": name, "generation": 3},
        "spec": {"replicas": want, "template": {"spec": {"containers": [{"name": "c", "image": image}]}}},
        "status": {"observedGeneration": 3, "replicas": want, "updatedReplicas": want, "readyReplicas": ready},
    }


def pod(name, restarts=0, waiting=None, last=None, owner="ReplicaSet"):
    cs = {"name": "c", "restartCount": restarts, "state": {"waiting": {"reason": waiting}} if waiting else {"running": {}}}
    if last:
        cs["lastState"] = {"terminated": {"reason": last}}
    return {"metadata": {"name": name, "ownerReferences": [{"kind": owner}]}, "status": {"containerStatuses": [cs]}}


def items(*xs):
    return json.dumps({"items": list(xs)})


def base_scenario(**over):
    s = {
        "helm": [
            {"match": ["lint"], "rc": 0},
            {"match": ["history"], "out": json.dumps(DEPLOYED)},
            {"match": ["get", "values", "--all"], "out": json.dumps({"secretonAuth": {"enabled": False}})},
            {"match": ["get", "values"], "out": json.dumps({"global": {"imageTag": "v0.1.0-rc51"}, "secrets": {"jwt": CANARY}})},
            {"match": ["upgrade"], "effects": ["check-overlay"]},
            {"match": ["rollback"], "effects": ["fix-http", "mark-rolled"]},
        ],
        "kubectl": [
            {"match": ["exec", "postgres-0"], "out": "2549\n"},
            {"match": ["get", "deploy,sts"], "when": "rolled", "out": items(deploy("authenc", 2, 2, image="ghcr.io/o/simpelv2-x:v0.1.0-rc51"))},
            {"match": ["get", "deploy,sts"], "when": "not-rolled", "out": items(deploy("authenc", 2, 2), deploy("secreton-0", 0, 1, kind="StatefulSet"))},
            {"match": ["get", "pods"], "out": items(pod("authenc-1"), pod("authenc-2"))},
            {"match": ["logs"], "out": "INFO started\nERROR something expected 42\n"},
        ],
        "gh": [],
        "git": [],
    }
    for k, v in over.items():
        s[k] = v + s[k] if isinstance(v, list) else v
    return s


class Handler(http.server.BaseHTTPRequestHandler):
    def _github(self):
        gh = getattr(self.server, "gh", {})
        body = gh.get("statuses", []) if "/statuses" in self.path else gh.get("deployments", [])
        if gh.get("fail"):
            self.send_response(500)
            self.end_headers()
            return
        raw = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.end_headers()
        self.wfile.write(raw)

    def _answer(self):
        if self.path.startswith("/repos/"):
            return self._github()
        broken = os.path.exists(os.path.join(self.server.fake_dir, "broken"))
        code = {"/health": 200, "/perlengkapan/simpel/": 200, "/portal/": 200,
                "/api/v1/perlengkapan/pakaian-dinas/jenis": 401, "/api/v1/iam/users": 401,
                "/api/v1/auth/login": 422}.get(self.path, 404)
        if broken and self.path == "/health":
            code = 500
        self.send_response(code)
        self.end_headers()

    do_GET = do_POST = _answer

    def log_message(self, *a):  # keep the output quiet
        pass


class Env:
    def __init__(self, script: Path):
        self.script = script
        self.tmp = Path(tempfile.mkdtemp(prefix="deploy-selftest-"))
        self.bin = self.tmp / "bin"
        self.bin.mkdir()
        for tool in ("helm", "kubectl", "gh", "git"):
            f = self.bin / tool
            f.write_text(SHIM.format(python=sys.executable, canary=CANARY))
            f.chmod(0o755)
        (self.tmp / "runtime").mkdir()
        self.chart = self.tmp / "chart"
        self.chart.mkdir()
        self.set_values("v0.1.0-rc52", "v0.1.0")
        socketserver.TCPServer.allow_reuse_address = True
        self.httpd = socketserver.ThreadingTCPServer(("127.0.0.1", 0), Handler)
        self.httpd.fake_dir = str(self.tmp)
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
        self.base_url = f"http://127.0.0.1:{self.httpd.server_address[1]}"

    def set_values(self, staging, production):
        for name, tag in (("values-staging.yaml", staging), ("values-production.yaml", production)):
            (self.chart / name).write_text(f"# header\nglobal:\n  # note\n  imageTag: {tag}\n  imagePullPolicy: IfNotPresent\nother:\n  imageTag: nope\n")

    def run(self, args, scenario, *, broken=False):
        for f in self.tmp.glob("count-*"):
            f.unlink()
        for name in ("broken", "overlay-ok", "calls.log", "marker-rolled"):
            (self.tmp / name).unlink(missing_ok=True)
        if broken:
            (self.tmp / "broken").write_text("1")
        (self.tmp / "scenario.json").write_text(json.dumps(scenario))
        env = dict(os.environ, PATH=f"{self.bin}:{os.environ['PATH']}", FAKE_SCENARIO=str(self.tmp / "scenario.json"),
                   FAKE_LOG=str(self.tmp / "calls.log"), FAKE_DIR=str(self.tmp), XDG_RUNTIME_DIR=str(self.tmp / "runtime"), GITHUB_API_URL=self.base_url, GITHUB_TOKEN="t")
        env.pop("GITHUB_STEP_SUMMARY", None)
        proc = subprocess.run([sys.executable, str(self.script), *args], capture_output=True, text=True, env=env, timeout=120)
        calls = (self.tmp / "calls.log").read_text() if (self.tmp / "calls.log").exists() else ""
        return proc, calls

    def run_env(self, env_name, tag, scenario, *extra, **kw):
        args = ["run", "--env", env_name, "--tag", tag, "--base-url", self.base_url, "--chart-dir", str(self.chart),
                "--smoke-pause", "0", "--wait", "1", *extra]
        return self.run(args, scenario, **kw)

    def leftover_overlays(self):
        return list((self.tmp / "runtime").glob("simpel-carry-*"))


FAILS: list[str] = []


def check(cond: bool, what: str, detail: str = "") -> None:
    print(("  ok   " if cond else "  FAIL ") + what)
    if not cond:
        FAILS.append(what + (f"\n         {detail}" if detail else ""))


def called(calls: str, needle: str) -> bool:
    return needle in calls


def run(script: Path) -> int:
    sys.path.insert(0, str(script.parent))
    import importlib.util

    spec = importlib.util.spec_from_file_location("pipeline_under_test", script)
    pl = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(pl)
    e = Env(script)

    print("values file parser")
    t = e.tmp / "v.yaml"
    t.write_text("x:\n  imageTag: wrong\nglobal:\n  # imageTag: commented\n  imageTag: v1.2.3-rc4  # trailing\n  y: 1\n")
    check(pl.values_image_tag(t) == "v1.2.3-rc4", "reads global.imageTag, skips comments and other blocks")
    t.write_text("global:\n  y: 1\nnext:\n  imageTag: other\n")
    check(pl.values_image_tag(t) is None, "does not read an imageTag from the next top-level block")

    print("tag ordering")
    check(pl.any_tag_key("v0.1.0-rc10") > pl.any_tag_key("v0.1.0-rc9"), "rc10 sorts after rc9 (numeric, not text)")
    check(pl.any_tag_key("v0.1.0") > pl.any_tag_key("v0.1.0-rc99"), "the final tag sorts after every rc of its version")
    check(pl.tag_key("production", "v0.1.0-rc1") is None and pl.tag_key("staging", "v0.1.0") is None, "each environment accepts only its own tag shape")

    print("log scan rules")
    check(bool(pl.scan_text("thread 'main' panicked at src/x.rs:1")), "a panic is denied")
    check(bool(pl.scan_text("FATAL:  the database system is starting up")), "FATAL is denied")
    check(bool(pl.scan_text("token=eyJhbGciOiJIUzI1NiIsInR5cCI6.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI.sig")), "a JWT-shaped token is denied")
    check(bool(pl.scan_text("scheduled data fetch every 15m")), "the integrasi server scheduler line is denied")
    check(not pl.scan_text("ERROR validation failed for user 42"), "an ordinary ERROR line is not a failure")

    print("S1 staging happy path")
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(), "--reseed", "--fixtures", str(Path(__file__).parents[3] / "tests/fixtures/e2e"), "--e2e-cmd", "true")
    check(p.returncode == 0, "exit 0", p.stdout[-600:] + p.stderr[-300:])
    check(calls.index("pg_dump") < calls.index("helm upgrade"), "the dump is taken BEFORE the upgrade")
    check("--set" not in calls and "--wait" not in calls.split("helm upgrade")[1].splitlines()[0], "no --set and no --wait on helm upgrade")
    check((e.tmp / "overlay-ok").read_text() == "yes" if (e.tmp / "overlay-ok").exists() else False, "the carry overlay held the secrets, mode 0600, while helm ran")
    check(not e.leftover_overlays(), "the overlay file is gone afterwards")
    check(CANARY not in p.stdout + p.stderr + calls, "no secret value reached the output or the call log")
    check(called(calls, "psql"), "fixtures were reseeded")
    check("rollback" not in calls, "no rollback on success")

    print("S2 verification fails, previous revision is restored")
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(), broken=True)
    check(p.returncode == 1, "exit 1 (failed, rolled back)", p.stdout[-500:])
    check("helm rollback simpel 7" in calls, "rolls back to the revision preflight recorded (7)")
    check("rolled back to v0.1.0-rc51" in p.stdout, "says what it restored")

    print("S3 the rollback does not verify either")
    sc = base_scenario(helm=[{"match": ["rollback"], "effects": ["mark-rolled"]}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc, broken=True)
    check(p.returncode == 3, "exit 3 (a person is needed)", p.stdout[-500:])
    check("MANUAL INTERVENTION" in p.stdout and ".dump" in p.stdout, "names the dump to restore from")

    print("S4 refused before anything changes")
    def untouched(label, p, calls, code=2, dumped=False):
        check(p.returncode == code, f"{label}: exit {code}", p.stdout[-400:] + p.stderr[-200:])
        check("helm upgrade" not in calls and (dumped or "pg_dump" not in calls), f"{label}: no upgrade" + ("" if dumped else ", no dump"))

    p, calls = e.run_env("production", "v0.1.0-rc52", base_scenario())
    untouched("an rc tag to production", p, calls)
    p, calls = e.run_env("staging", "v0.1.0", base_scenario())
    untouched("a final tag to staging", p, calls)
    p, calls = e.run_env("staging", "v0.1.0-rc53", base_scenario())
    untouched("values file records another tag", p, calls)
    sc = base_scenario(helm=[{"match": ["history"], "rc": 1, "err": "Error: release: not found"}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    untouched("no Helm release (first install stays manual)", p, calls)
    check("only upgrades" in p.stdout, "no release: explains that first install is manual")
    sc = base_scenario(helm=[{"match": ["get", "values"], "not": ["--all"], "out": json.dumps({"global": {"imageTag": "v0.1.0-rc60"}, "secrets": {}})}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    untouched("downgrade", p, calls)
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc, "--allow-downgrade", "--dry-run")
    check(p.returncode == 0, "downgrade passes with --allow-downgrade")
    sc = base_scenario(helm=[{"match": ["lint"], "rc": 1}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    untouched("helm lint fails", p, calls)
    sc = base_scenario(helm=[{"match": ["history"], "rc": 1, "err": "connection refused"}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    untouched("cluster unreachable", p, calls)
    sc = base_scenario(helm=[{"match": ["history"], "out": json.dumps([{"revision": 7, "status": "failed"}])}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    untouched("no deployed revision", p, calls)
    e.set_values("v0.1.0-rc52", "v0.1.0")
    p, calls = e.run_env("production", "v0.1.0", base_scenario(), "--reseed")
    untouched("production with --reseed", p, calls)
    check("helm" not in calls, "production with --reseed: not even a helm call")
    p, calls = e.run_env("production", "v0.1.0", base_scenario(), "--e2e-cmd", "true")
    untouched("production with --e2e-cmd", p, calls)

    print("S5 dump problems stop the upgrade")
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=[{"match": ["exec", "postgres-0"], "rc": 1, "err": "pg_dump: error"}]))
    untouched("pg_dump fails", p, calls, dumped=True)
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=[{"match": ["exec", "postgres-0"], "out": "12\n"}]))
    untouched("a 12-entry dump is not trusted", p, calls, dumped=True)

    print("S5b reseed refuses production by itself")
    try:
        pl.reseed("production", Path(__file__).parents[3] / "tests/fixtures/e2e")
        check(False, "reseed(production) raises Refused")
    except pl.Refused:
        check(True, "reseed(production) raises Refused, whatever the CLI did before it")

    print("S6 --dry-run")
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(), "--dry-run")
    check(p.returncode == 0 and "pg_dump" not in calls and "helm upgrade" not in calls, "stops after preflight")

    print("S7 what verification looks at")
    for label, kube in {
        "a crash-looping pod": [{"match": ["get", "pods"], "out": items(pod("authenc-1", waiting="CrashLoopBackOff"))}],
        "an OOM-killed pod": [{"match": ["get", "pods"], "out": items(pod("authenc-1", last="OOMKilled"))}],
        "a pod restarted 9 times": [{"match": ["get", "pods"], "out": items(pod("authenc-1", restarts=9))}],
        "an old image still running": [{"match": ["get", "deploy,sts"], "out": items(deploy("authenc", 2, 2, image="ghcr.io/o/simpelv2-x:v0.1.0-rc51"))}],
        "a workload that is not ready": [{"match": ["get", "deploy,sts"], "out": items(deploy("authenc", 1, 2))}],
    }.items():
        p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=kube))
        check(p.returncode in (1, 3) and "helm rollback" in calls, f"{label}: fails verification and rolls back", p.stdout[-300:])
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=[{"match": ["get", "deploy,sts"], "out": items()}]))
    check(p.returncode in (1, 3) and "helm rollback" in calls, "no workloads found at all is a failure, not a pass")
    sec = [{"match": ["get", "deploy,sts"], "out": items(deploy("authenc", 2, 2), deploy("secreton-0", 0, 1, kind="StatefulSet"))}]
    p, _ = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=sec))
    check(p.returncode == 0, "sealed Secreton does not fail a deploy while its auth is off")
    sc = base_scenario(kubectl=sec, helm=[{"match": ["get", "values", "--all"], "out": json.dumps({"secretonAuth": {"enabled": True}})}])
    p, calls = e.run_env("staging", "v0.1.0-rc52", sc)
    check(p.returncode in (1, 3) and "helm rollback" in calls, "...but does once Secreton auth is on")
    pods = [{"match": ["get", "pods"], "out": items(pod("migrate-job-x", restarts=9, owner="Job"), pod("authenc-1"))}]
    p, _ = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=pods))
    check(p.returncode == 0, "Job-owned pods are not judged as long-running pods")

    print("S8 e2e and log scan")
    p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(), "--e2e-cmd", "false")
    check(p.returncode == 1 and "helm rollback" in calls, "a failing suite rolls the deploy back")
    for label, line in {"a panic": "thread 'main' panicked at src/a.rs:3", "a FATAL": "FATAL: boom",
                        "a leaked token": "auth header eyJhbGciOiJIUzI1NiIsInR5cCI6.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI.abc"}.items():
        p, calls = e.run_env("staging", "v0.1.0-rc52", base_scenario(kubectl=[{"match": ["logs"], "out": line + "\n"}]))
        check(p.returncode == 1 and "helm rollback" in calls, f"{label} in a pod log rolls the deploy back")

    print("S9 production")
    prod = [{"match": ["get", "deploy,sts"], "when": "not-rolled", "out": items(deploy("authenc", 2, 2, image="ghcr.io/o/simpelv2-x:v0.1.0"))}]
    p, calls = e.run_env("production", "v0.1.0", base_scenario(kubectl=prod, helm=[{"match": ["get", "values"], "not": ["--all"], "out": json.dumps({"global": {"imageTag": "v0.0.9"}, "secrets": {"jwt": CANARY}})}]))
    check(p.returncode == 0 and "helm upgrade" in calls and "simpelv2-production" in calls, "production upgrades in its own namespace")
    check("psql" not in calls, "production never reseeds")

    print("S10 promote-guard")
    SHA = "abc12345deadbeef"

    def guard(tag, *, tags="v0.1.0 v0.1.0-rc51\n", status="success", deployments="default", api_down=False):
        if deployments == "default":
            deployments = [{"id": 11, "sha": SHA, "ref": "v0.1.0-rc51"}]
        e.httpd.gh = {"deployments": deployments, "statuses": [{"state": status}], "fail": api_down}
        sc = {
            "git": [{"match": ["rev-list"], "out": SHA + "\n"}, {"match": ["--points-at"], "out": tags}],
            "helm": [], "kubectl": [], "gh": [],
        }
        return e.run(["promote-guard", "--tag", tag, "--repo", "o/r"], sc)

    p, calls = guard("v0.1.0")
    check(p.returncode == 0 and p.stdout.strip().splitlines()[-1] == "v0.1.0-rc51", "ok: the commit of an rc whose staging deployment is the newest and succeeded")
    check("helm" not in calls and "kubectl" not in calls and "gh " not in calls, "the guard needs neither the cluster nor the gh CLI")
    p, _ = guard("v0.1.0", deployments=[{"id": 11, "sha": SHA, "ref": "refs/tags/v0.1.0-rc51"}])
    check(p.returncode == 0, "a deployment ref spelled refs/tags/<rc> is understood")
    p, _ = guard("v0.1.0-rc51")
    check(p.returncode == 2, "an rc tag is not a final tag")
    p, _ = guard("v0.1.0", tags="v0.1.0\n")
    check(p.returncode == 2, "refused: no rc on that commit")
    # the newest staging deployment is that other rc at the same commit, so only the version match can refuse it
    p, _ = guard("v0.1.0", tags="v0.1.0 v0.2.0-rc1\n", deployments=[{"id": 11, "sha": SHA, "ref": "v0.2.0-rc1"}])
    check(p.returncode == 2, "refused: the rc belongs to another version")
    p, _ = guard("v0.1.0", status="failure")
    check(p.returncode == 2, "refused: the staging deployment did not succeed")
    p, _ = guard("v0.1.0", status="in_progress")
    check(p.returncode == 2, "refused: the staging deployment has not finished")
    p, _ = guard("v0.1.0", deployments=[])
    check(p.returncode == 2, "refused: staging has no recorded deployment")
    p, _ = guard("v0.1.0", api_down=True)
    check(p.returncode == 2 and "not guessing" in p.stdout, "refused, not guessed, when GitHub cannot be read")
    p, _ = guard("v0.1.0", deployments=[{"id": 12, "sha": "ffff0000ffff0000", "ref": "v0.1.0-rc52"}, {"id": 11, "sha": SHA, "ref": "v0.1.0-rc51"}])
    check(p.returncode == 2, "refused: a newer rc has been deployed to staging since")

    e.httpd.shutdown()
    if FAILS:
        print(f"\n{len(FAILS)} check(s) failed:")
        for f in FAILS:
            print(" -", f)
        return 1
    print("\nall deploy pipeline self-tests passed")
    return 0
