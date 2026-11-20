#!/usr/bin/env python3
"""Deploy pipeline for SIMPEL: one tag, one verified upgrade, one way back.

Replaces the hand-run sequence that proved itself from rc31 to rc51 (dump, carry
overlay, `helm upgrade`, reseed, suite, log scan) with the same steps in code, so a
release tag is deployed the same way every time and a failed deploy is undone without
anybody having to be awake.

    pipeline.py run        --env staging    --tag v0.1.0-rc52 --base-url http://10.1.7.121 \
                           --reseed --e2e-cmd '<command that runs the suite>'
    pipeline.py run        --env production --tag v0.1.0 --base-url https://simpel.kejaksaan.go.id
    pipeline.py promote-guard --tag v0.1.0 --repo owner/repo
    pipeline.py --self-test

Steps of `run` (each is also a subcommand):

    preflight  tag shape, values file records the tag, chart lints, release EXISTS (this
               pipeline only ever upgrades), no accidental downgrade, previous revision noted
    snapshot   pg_dump -Fc inside the postgres pod, kept in the pod's volume, TOC checked
    upgrade    helm upgrade with the secrets carry overlay derived from the live release
    verify     rollouts, no crash-looping pod, every container runs THIS tag, smoke over HTTP
    e2e        (optional) the command given with --e2e-cmd, staging only
    logscan    every simpel pod's log: no panic, no FATAL, no credential-shaped token
    rollback   helm rollback to the revision noted by preflight, then verify again

Exit codes: 0 deployed and verified; 1 verification failed AND the previous revision was
restored and verified; 2 refused before anything changed; 3 failed and the rollback did not
verify either (a person is needed; the dump path is printed).

Rollback is `helm rollback`, which is only safe because migrations here are additive
(expand/contract). A release that ships a destructive migration needs the dump restored by
hand; this script prints the dump name for that and does not pretend otherwise.

Standard library only (the CI runner image has no PyYAML), and secret values are never
printed: `helm get values` output is parsed in memory and only the `secrets` object is
written to a mode-0600 file that is overwritten and removed afterwards.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CHART = REPO / "infra" / "helm" / "simpel"
RELEASE = "simpel"
IMAGE_RE = re.compile(r"^ghcr\.io/[^/]+/simpelv2-")
TAG_RE = {
    "staging": re.compile(r"^v(\d+)\.(\d+)\.(\d+)-rc(\d+)$"),
    "production": re.compile(r"^v(\d+)\.(\d+)\.(\d+)()$"),
}
ENVS = {
    "staging": {"namespace": "simpelv2-staging", "values": "values-staging.yaml"},
    "production": {"namespace": "simpelv2-production", "values": "values-production.yaml"},
}
POSTGRES_POD = "postgres-0"
POSTGRES_CONTAINER = "postgres"
DUMP_DIR = "/var/lib/postgresql/data/pre-upgrade"
DATABASE = "dbsimpelv2"
KEEP_DUMPS = 3
MIN_TOC = 500

# Reachable through the ingress without a login; what each proves is in the comment.
SMOKE = [
    ("GET", "/health", {200}),  # the gateway answers
    ("GET", "/perlengkapan/simpel/", {200}),  # the SPA is served
    ("GET", "/portal/", {200}),  # the portal is served
    ("GET", "/api/v1/perlengkapan/pakaian-dinas/jenis", {401}),  # perlengkapan alive, authenticator live
    ("GET", "/api/v1/iam/users", {401}),  # authenc alive, authenticator live
    ("POST", "/api/v1/auth/login", {422}),  # the login handler validates its body
]

# A line matching one of these in ANY simpel pod log fails the deploy. Each earlier
# release candidate that taught one of them is named, because "why is this here" is the
# first thing someone asks when it fires.
LOG_DENY = [
    (re.compile(r"panicked at"), "a Rust panic (rc31: NULL kode_satker killed the backend)"),
    (re.compile(r"\bFATAL\b"), "FATAL (rc48: the postgres probe logged one eighteen times a minute)"),
    (
        re.compile(r"eyJ[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}\."),
        "a JWT-shaped token in a log line (rc47: a MonSAKTI token was logged)",
    ),
    (
        re.compile(r"scheduled data fetch"),
        "the integrasi server running its own scheduler (rc44: it hit the real API every 15 minutes)",
    ),
]
# Lines matching these are dropped BEFORE the deny list is applied. Empty on purpose: add an
# entry only with the reason, in this file, next to the incident it excuses.
LOG_ALLOW: list[tuple[re.Pattern, str]] = []

EXIT_OK, EXIT_ROLLED_BACK, EXIT_REFUSED, EXIT_MANUAL = 0, 1, 2, 3


class Refused(Exception):
    """Raised before anything has changed."""


class Failed(Exception):
    """Raised after the cluster may have changed; the caller decides about rollback."""


def log(msg: str) -> None:
    print(msg, flush=True)


def summary(md: str) -> None:
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if path:
        with open(path, "a", encoding="utf-8") as fh:
            fh.write(md + "\n")


def sh(cmd: list[str], *, check: bool = True, input_text: str | None = None, timeout: int | None = None):
    """Run a command. stdout/stderr are captured and NEVER echoed: some of what we run
    (`helm get values`) prints secrets. A failure reports the command name, its exit code
    and the last lines of stderr only."""
    proc = subprocess.run(cmd, input=input_text, capture_output=True, text=True, timeout=timeout)
    if check and proc.returncode != 0:
        tail = " | ".join((proc.stderr or "").strip().splitlines()[-3:])[:400]
        raise Failed(f"`{cmd[0]} {cmd[1] if len(cmd) > 1 else ''}` exited {proc.returncode}: {tail}")
    return proc


def kubectl_json(ns: str, *args: str) -> dict:
    return json.loads(sh(["kubectl", "-n", ns, *args, "-o", "json"]).stdout or "{}")


def helm_values(ns: str, *, all_values: bool = False) -> dict:
    cmd = ["helm", "get", "values", RELEASE, "-n", ns, "-o", "json"]
    if all_values:
        cmd.insert(3, "--all")
    return json.loads(sh(cmd).stdout or "null") or {}


def values_image_tag(path: Path) -> str | None:
    """`global.imageTag` from a values file, without a YAML parser: the first `imageTag:`
    after the top-level `global:` and before the next top-level key."""
    in_global = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if re.match(r"^global:\s*(#.*)?$", line):
            in_global = True
            continue
        if in_global and re.match(r"^\S", line) and not line.startswith("#"):
            return None
        if in_global:
            m = re.match(r"^\s+imageTag:\s*([^\s#]+)", line)
            if m:
                return m.group(1).strip("'\"")
    return None


def tag_key(env: str, tag: str):
    m = TAG_RE[env].match(tag)
    if not m:
        return None
    major, minor, patch, rc = m.groups()
    # A final tag sorts after every rc of the same version.
    return (int(major), int(minor), int(patch), int(rc) if rc != "" else 10**9)


def any_tag_key(tag: str):
    for env in ("staging", "production"):
        k = tag_key(env, tag)
        if k:
            return k
    return None


# ───────────────────────────── preflight ─────────────────────────────


def preflight(env: str, tag: str, state: dict, *, chart: Path, allow_downgrade: bool) -> None:
    cfg = ENVS[env]
    ns = cfg["namespace"]
    if tag_key(env, tag) is None:
        want = "vX.Y.Z-rcN" if env == "staging" else "vX.Y.Z (no -rc)"
        raise Refused(f"tag {tag!r} is not a {env} tag; {env} takes {want}")

    values = chart / cfg["values"]
    recorded = values_image_tag(values)
    if recorded != tag:
        raise Refused(
            f"{values.name} records global.imageTag={recorded!r}, not {tag!r}. The bump PR must land "
            f"before the tag is cut: the chart at the tag is what gets deployed."
        )

    if sh(["helm", "lint", str(chart), "-f", str(values)], check=False).returncode != 0:
        raise Refused(f"helm lint fails for {values.name}")

    hist = sh(["helm", "history", RELEASE, "-n", ns, "-o", "json", "--max", "20"], check=False)
    if hist.returncode != 0 and "not found" not in (hist.stderr or "").lower():
        raise Failed(f"cannot read the release history of {RELEASE!r} in {ns}: {(hist.stderr or '').strip()[-200:]}")
    if hist.returncode != 0:
        raise Refused(
            f"no Helm release {RELEASE!r} in {ns}. This pipeline only upgrades; the first install is the "
            f"bootstrap in infra/helm/RUNBOOK.md (§1) and stays manual"
        )
    deployed = [h for h in json.loads(hist.stdout or "[]") if h.get("status") == "deployed"]
    if not deployed:
        raise Refused(f"release {RELEASE!r} in {ns} has no revision in state 'deployed' (a half-finished upgrade?)")
    current = max(deployed, key=lambda h: h["revision"])
    state["previous_revision"] = current["revision"]

    live = (helm_values(ns).get("global") or {}).get("imageTag")
    state["previous_tag"] = live
    if live and not allow_downgrade:
        a, b = any_tag_key(live), tag_key(env, tag)
        if a and b and b < a:
            raise Refused(f"{tag} is older than what {env} runs ({live}); pass --allow-downgrade to do that on purpose")
    log(f"preflight ok: {env} runs {live or '?'} at revision {current['revision']}, deploying {tag}")


# ───────────────────────────── snapshot ─────────────────────────────


def snapshot(env: str, tag: str, state: dict) -> None:
    ns = ENVS[env]["namespace"]
    name = f"{env}-{tag}-{time.strftime('%Y%m%dT%H%M%S', time.gmtime())}.dump"
    path = f"{DUMP_DIR}/{name}"
    script = (
        f'set -e; mkdir -p {DUMP_DIR}; '
        f'PGPASSWORD="$POSTGRES_PASSWORD" pg_dump -h 127.0.0.1 -U "$POSTGRES_USER" -Fc -d {DATABASE} -f {path}; '
        f'PGPASSWORD="$POSTGRES_PASSWORD" pg_restore -l {path} | grep -c . ; '
        f'ls -1t {DUMP_DIR}/*.dump | tail -n +{KEEP_DUMPS + 1} | xargs -r rm -f'
    )
    proc = sh(["kubectl", "-n", ns, "exec", POSTGRES_POD, "-c", POSTGRES_CONTAINER, "--", "sh", "-c", script], timeout=1800)
    lines = (proc.stdout or "").split()
    toc = int(lines[-1]) if lines and lines[-1].isdigit() else 0
    if toc < MIN_TOC:
        raise Failed(f"the dump's table of contents has {toc} entries (expected at least {MIN_TOC}); not trusting it")
    state["dump"] = f"{POSTGRES_POD}:{path}"
    log(f"snapshot ok: {state['dump']} ({toc} TOC entries)")


# ───────────────────────────── upgrade ─────────────────────────────


def secure_remove(path: str) -> None:
    try:
        size = os.path.getsize(path)
        with open(path, "r+b") as fh:
            fh.write(b"\0" * size)
            fh.flush()
            os.fsync(fh.fileno())
    except OSError:
        pass
    try:
        os.unlink(path)
    except OSError:
        pass


def upgrade(env: str, tag: str, *, chart: Path, timeout: str) -> None:
    cfg = ENVS[env]
    ns = cfg["namespace"]
    carry = {"secrets": helm_values(ns).get("secrets", {})}
    base = os.environ.get("XDG_RUNTIME_DIR") or "/dev/shm"
    if not os.path.isdir(base) or not os.access(base, os.W_OK):
        base = tempfile.gettempdir()
    fd, overlay = tempfile.mkstemp(prefix="simpel-carry-", suffix=".json", dir=base)
    try:
        os.fchmod(fd, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            json.dump(carry, fh)
        # No --set: the tag lives in the values file (that is how the repo and the cluster once came to
        # describe different systems). No --wait: Secreton can legitimately sit sealed, so readiness is
        # verified per workload below, not asked of Helm.
        sh(
            [
                "helm", "upgrade", "--install", RELEASE, str(chart),
                "-f", str(chart / cfg["values"]), "-f", overlay,
                "--namespace", ns, "--timeout", timeout,
            ],
            timeout=3600,
        )
    finally:
        secure_remove(overlay)
    log(f"upgrade ok: helm applied {tag}")


# ───────────────────────────── verify ─────────────────────────────

BAD_WAITING = {"CrashLoopBackOff", "ImagePullBackOff", "ErrImagePull", "CreateContainerConfigError", "InvalidImageName"}


def workloads(ns: str) -> list[dict]:
    items = kubectl_json(ns, "get", "deploy,sts", "-l", f"app.kubernetes.io/instance={RELEASE}").get("items", [])
    return items


def required_workloads(ns: str) -> list[dict]:
    """Every Deployment and StatefulSet of the release, except Secreton while its auth is off: it
    comes up sealed after any restart and is not Ready by design."""
    auth_on = bool(((helm_values(ns, all_values=True).get("secretonAuth")) or {}).get("enabled"))
    out = []
    for w in workloads(ns):
        if w["metadata"]["name"].startswith("secreton") and not auth_on:
            continue
        out.append(w)
    return out


def rollout_problems(w: dict) -> str | None:
    st, spec, meta = w.get("status", {}), w.get("spec", {}), w["metadata"]
    want = spec.get("replicas", 1)
    if st.get("observedGeneration", 0) < meta.get("generation", 0):
        return "controller has not seen the latest spec"
    if st.get("updatedReplicas", 0) < want or st.get("readyReplicas", 0) < want or st.get("replicas", 0) > want:
        return f"{st.get('readyReplicas', 0)}/{want} ready, {st.get('updatedReplicas', 0)} updated"
    return None


def wait_rollouts(ns: str, timeout_s: int) -> list[str]:
    deadline = time.monotonic() + timeout_s
    while True:
        bad = []
        for w in required_workloads(ns):
            why = rollout_problems(w)
            if why:
                bad.append(f"{w['kind']}/{w['metadata']['name']}: {why}")
        if not bad or time.monotonic() >= deadline:
            return bad
        time.sleep(min(10, max(1, deadline - time.monotonic())))


def pod_problems(ns: str, max_restarts: int) -> list[str]:
    out = []
    for p in kubectl_json(ns, "get", "pods", "-l", f"app.kubernetes.io/instance={RELEASE}").get("items", []):
        owners = {o.get("kind") for o in p["metadata"].get("ownerReferences", [])}
        if not owners & {"ReplicaSet", "StatefulSet"}:
            continue  # Jobs and CronJob pods end by design
        name = p["metadata"]["name"]
        for cs in p.get("status", {}).get("containerStatuses", []) or []:
            waiting = (cs.get("state", {}).get("waiting") or {}).get("reason")
            last = (cs.get("lastState", {}).get("terminated") or {}).get("reason")
            if waiting in BAD_WAITING:
                out.append(f"{name}/{cs['name']}: {waiting}")
            elif last == "OOMKilled":
                out.append(f"{name}/{cs['name']}: OOMKilled")
            elif cs.get("restartCount", 0) > max_restarts:
                out.append(f"{name}/{cs['name']}: restarted {cs['restartCount']} times")
    return out


def image_problems(ns: str, tag: str) -> list[str]:
    out = []
    for w in required_workloads(ns):
        tmpl = w.get("spec", {}).get("template", {}).get("spec", {})
        for c in (tmpl.get("containers") or []) + (tmpl.get("initContainers") or []):
            img = c.get("image", "")
            if IMAGE_RE.match(img) and not img.endswith(":" + tag):
                out.append(f"{w['metadata']['name']}/{c['name']} runs {img.split('/')[-1]}, not {tag}")
    return out


def smoke(base_url: str, attempts: int = 6, pause: float = 10.0) -> list[str]:
    last: list[str] = []
    for i in range(attempts):
        last = []
        for method, path, ok in SMOKE:
            req = urllib.request.Request(
                base_url.rstrip("/") + path,
                method=method,
                data=b"{}" if method == "POST" else None,
                headers={"content-type": "application/json"} if method == "POST" else {},
            )
            try:
                with urllib.request.urlopen(req, timeout=15) as r:
                    code = r.status
            except urllib.error.HTTPError as e:
                code = e.code
            except Exception as e:  # noqa: BLE001 — any transport failure is a failed smoke check
                last.append(f"{method} {path}: {type(e).__name__}")
                continue
            if code not in ok:
                last.append(f"{method} {path}: HTTP {code}, expected {sorted(ok)}")
        if not last:
            return []
        if i + 1 < attempts:
            time.sleep(pause)
    return last


def verify(env: str, tag: str, base_url: str, *, wait: int, max_restarts: int, pause: float = 10.0) -> list[str]:
    ns = ENVS[env]["namespace"]
    problems = []
    problems += wait_rollouts(ns, wait)
    problems += pod_problems(ns, max_restarts)
    problems += image_problems(ns, tag)
    problems += smoke(base_url, pause=pause)
    return problems


# ───────────────────────────── log scan ─────────────────────────────


def scan_text(text: str) -> list[tuple[str, str]]:
    hits = []
    for line in text.splitlines():
        if any(p.search(line) for p, _ in LOG_ALLOW):
            continue
        for pat, why in LOG_DENY:
            if pat.search(line):
                hits.append((why, line[:160]))
                break
    return hits


def normalise(line: str) -> str:
    line = re.sub(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "<uuid>", line)
    line = re.sub(r"\d{4}-\d\d-\d\dT[\d:.]+Z?", "<ts>", line)
    return re.sub(r"\d+", "N", line)[:140]


def logscan(env: str, since: str) -> list[str]:
    ns = ENVS[env]["namespace"]
    problems, error_groups = [], {}
    pods = kubectl_json(ns, "get", "pods", "-l", f"app.kubernetes.io/instance={RELEASE}").get("items", [])
    for p in pods:
        owners = {o.get("kind") for o in p["metadata"].get("ownerReferences", [])}
        if not owners & {"ReplicaSet", "StatefulSet"}:
            continue
        name = p["metadata"]["name"]
        out = sh(["kubectl", "-n", ns, "logs", name, "--all-containers", f"--since={since}", "--tail=20000"], check=False).stdout or ""
        for why, line in scan_text(out):
            problems.append(f"{name}: {why}: {line}")
        for line in out.splitlines():
            if re.search(r"\bERROR\b", line):
                k = normalise(line)
                error_groups[k] = error_groups.get(k, 0) + 1
    if error_groups:
        top = sorted(error_groups.items(), key=lambda kv: -kv[1])[:12]
        summary("#### ERROR lines by message (informational)\n\n| count | message |\n|---:|---|\n" + "\n".join(f"| {n} | `{m}` |" for m, n in top))
    return problems


# ───────────────────────────── reseed / rollback / run ─────────────────────────────


def reseed(env: str, fixtures: Path) -> None:
    if env != "staging":
        raise Refused("fixtures are loaded into staging only; production data is never reseeded")
    ns = ENVS[env]["namespace"]
    seeds = sorted(fixtures.glob("seed-*.sql"))
    if not seeds:
        raise Refused(f"no seed-*.sql under {fixtures}")
    for f in seeds:
        sh(
            ["kubectl", "-n", ns, "exec", "-i", POSTGRES_POD, "-c", POSTGRES_CONTAINER, "--", "sh", "-c",
             f'PGPASSWORD="$POSTGRES_PASSWORD" psql -h 127.0.0.1 -U "$POSTGRES_USER" -d {DATABASE} -v ON_ERROR_STOP=1'],
            input_text=f.read_text(encoding="utf-8"),
            timeout=600,
        )
    log(f"reseed ok: {len(seeds)} fixture file(s)")


def rollback(env: str, state: dict, base_url: str, *, wait: int, max_restarts: int, pause: float) -> bool:
    ns = ENVS[env]["namespace"]
    rev = state.get("previous_revision")
    if not rev:
        log("rollback impossible: no previous revision was recorded")
        return False
    log(f"ROLLBACK: helm rollback {RELEASE} to revision {rev}")
    if sh(["helm", "rollback", RELEASE, str(rev), "-n", ns, "--timeout", "20m"], check=False).returncode != 0:
        return False
    prev = state.get("previous_tag") or ""
    problems = verify(env, prev, base_url, wait=wait, max_restarts=max_restarts, pause=pause) if prev else wait_rollouts(ns, wait)
    for p in problems:
        log(f"  after rollback: {p}")
    return not problems


def cmd_run(a) -> int:
    env, tag = a.env, a.tag
    chart = Path(a.chart_dir)
    state: dict = {}
    t0 = time.time()
    rows: list[tuple[str, str]] = []

    def done(step: str, text: str = "ok") -> None:
        rows.append((step, text))

    def finish(code: int, headline: str) -> int:
        body = "\n".join(f"| {s} | {t} |" for s, t in rows)
        summary(f"### {headline}\n\n| step | result |\n|---|---|\n{body}\n\nprevious: `{state.get('previous_tag')}` (revision {state.get('previous_revision')}); dump: `{state.get('dump', '-')}`")
        log(headline)
        return code

    if env == "production" and (a.reseed or a.e2e_cmd):
        log("REFUSED: --reseed and --e2e-cmd are staging only; production gets the non-destructive smoke")
        return EXIT_REFUSED
    try:
        preflight(env, tag, state, chart=chart, allow_downgrade=a.allow_downgrade)
        done("preflight")
    except Refused as e:
        done("preflight", f"REFUSED: {e}")
        log(f"REFUSED: {e}")
        return finish(EXIT_REFUSED, f"{env} {tag}: refused, nothing changed")
    except Failed as e:
        done("preflight", f"error: {e}")
        return finish(EXIT_REFUSED, f"{env} {tag}: preflight could not complete, nothing changed")

    if a.dry_run:
        done("dry-run", "stopped after preflight")
        return finish(EXIT_OK, f"{env} {tag}: dry run ok")

    try:
        snapshot(env, tag, state)
        done("snapshot", state["dump"])
    except Failed as e:
        done("snapshot", f"FAILED: {e}")
        log(f"snapshot failed, not upgrading: {e}")
        return finish(EXIT_REFUSED, f"{env} {tag}: no usable dump, nothing changed")

    failure = None
    try:
        upgrade(env, tag, chart=chart, timeout=a.helm_timeout)
        done("upgrade")
        problems = verify(env, tag, a.base_url, wait=a.wait, max_restarts=a.max_restarts, pause=a.smoke_pause)
        if problems:
            raise Failed("verification failed:\n  " + "\n  ".join(problems))
        done("verify", "rollouts, pods, images, smoke")
        if a.reseed:
            reseed(env, Path(a.fixtures))
            done("reseed")
        if a.e2e_cmd:
            if subprocess.run(a.e2e_cmd, shell=True).returncode != 0:  # noqa: S602 — the operator's own command
                raise Failed("the e2e suite failed")
            done("e2e", "passed")
        problems = logscan(env, a.log_since)
        if problems:
            raise Failed("log scan:\n  " + "\n  ".join(problems[:20]))
        done("logscan", "clean")
    except (Failed, Refused) as e:
        failure = str(e)
        done("FAILED", failure.splitlines()[0])
        log(f"FAILED: {failure}")

    if failure is None:
        return finish(EXIT_OK, f"{env} {tag}: deployed and verified")

    if a.no_rollback:
        return finish(EXIT_MANUAL, f"{env} {tag}: failed, rollback disabled by --no-rollback")
    if rollback(env, state, a.base_url, wait=a.wait, max_restarts=a.max_restarts, pause=a.smoke_pause):
        done("rollback", f"restored {state.get('previous_tag')} (revision {state['previous_revision']}) and verified")
        return finish(EXIT_ROLLED_BACK, f"{env} {tag}: FAILED and rolled back to {state.get('previous_tag')}")
    done("rollback", "FAILED: needs a person")
    log(f"MANUAL INTERVENTION: the rollback did not verify. Dump: {state.get('dump', 'none taken')}")
    return finish(EXIT_MANUAL, f"{env} {tag}: FAILED and the rollback did not verify")


def github_get(path: str):
    """GET from the GitHub REST API with the standard library: the ARC runner image has no `gh` CLI
    (release.yml and labeler.yml work around the same gap). GITHUB_API_URL lets the self-test stand in."""
    base = os.environ.get("GITHUB_API_URL", "https://api.github.com").rstrip("/")
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN") or ""
    headers = {"Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        with urllib.request.urlopen(urllib.request.Request(base + path, headers=headers), timeout=30) as r:
            return json.loads(r.read() or b"[]")
    except urllib.error.HTTPError as e:
        raise Failed(f"GitHub API {path.split('?')[0]}: HTTP {e.code}") from e
    except (urllib.error.URLError, TimeoutError) as e:
        raise Failed(f"GitHub API {path.split('?')[0]}: {e}") from e


def cmd_promote_guard(a) -> int:
    """A final tag may deploy to production only if it IS a release candidate that passed staging: the same
    commit, and the NEWEST staging deployment GitHub recorded is that commit and succeeded. The record is
    GitHub's own deployment history (a job that names `environment: staging` writes one, with the result of
    the job), so this needs no access to the cluster."""
    tag = a.tag
    if tag_key("production", tag) is None:
        log(f"REFUSED: {tag!r} is not a final tag")
        return EXIT_REFUSED
    sha = sh(["git", "rev-list", "-n", "1", tag]).stdout.strip()
    rcs = [t for t in sh(["git", "tag", "--points-at", sha]).stdout.split() if tag_key("staging", t) and t.startswith(tag + "-rc")]
    if not rcs:
        log(f"REFUSED: no {tag}-rcN tag points at {sha[:8]}; a final tag must sit on the commit that passed staging")
        return EXIT_REFUSED
    try:
        deps = github_get(f"/repos/{a.repo}/deployments?environment=staging&per_page=10")
    except Failed as e:
        log(f"REFUSED: cannot read the staging deployment history ({e}); not guessing")
        return EXIT_REFUSED
    if not deps:
        log("REFUSED: staging has no recorded deployment")
        return EXIT_REFUSED
    latest = deps[0]
    ref = str(latest.get("ref", "")).replace("refs/tags/", "")
    if latest.get("sha") != sha or ref not in rcs:
        log(f"REFUSED: the newest staging deployment is {ref or '?'} at {str(latest.get('sha'))[:8]}; {tag} sits on {sha[:8]} "
            f"({', '.join(rcs)}). Staging has moved on, or this rc never ran there")
        return EXIT_REFUSED
    try:
        sts = github_get(f"/repos/{a.repo}/deployments/{latest['id']}/statuses?per_page=1")
    except Failed as e:
        log(f"REFUSED: cannot read the deployment status ({e}); not guessing")
        return EXIT_REFUSED
    state = sts[0].get("state") if sts else None
    if state != "success":
        log(f"REFUSED: the staging deployment of {ref} ended as {state or 'unknown'}, not success")
        return EXIT_REFUSED
    log(f"promote-guard ok: {tag} = {ref} at {sha[:8]}, the newest staging deployment, verified")
    print(ref)
    return EXIT_OK


def main(argv=None) -> int:
    if argv is None:
        argv = sys.argv[1:]
    if argv[:1] == ["--self-test"]:
        sys.path.insert(0, str(Path(__file__).parent))
        import selftest  # noqa: PLC0415

        return selftest.run(Path(__file__))
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run", help="preflight, snapshot, upgrade, verify, e2e, logscan, rollback on failure")
    r.add_argument("--env", choices=ENVS, required=True)
    r.add_argument("--tag", required=True)
    r.add_argument("--base-url", required=True, help="where the smoke checks go, e.g. http://10.1.7.121")
    r.add_argument("--chart-dir", default=str(CHART))
    r.add_argument("--fixtures", default=str(REPO / "tests" / "fixtures" / "e2e"))
    r.add_argument("--reseed", action="store_true", help="staging only: reapply the e2e fixtures after the upgrade")
    r.add_argument("--e2e-cmd", help="staging only: command that runs the suite; non-zero fails the deploy")
    r.add_argument("--helm-timeout", default="20m")
    r.add_argument("--wait", type=int, default=900, help="seconds to wait for rollouts")
    r.add_argument("--max-restarts", type=int, default=3)
    r.add_argument("--smoke-pause", type=float, default=10.0)
    r.add_argument("--log-since", default="45m")
    r.add_argument("--allow-downgrade", action="store_true")
    r.add_argument("--no-rollback", action="store_true")
    r.add_argument("--dry-run", action="store_true", help="stop after preflight")
    g = sub.add_parser("promote-guard", help="may this final tag deploy to production?")
    g.add_argument("--tag", required=True)
    g.add_argument("--repo", required=True)
    a = ap.parse_args(argv)
    return cmd_run(a) if a.cmd == "run" else cmd_promote_guard(a)


if __name__ == "__main__":
    sys.exit(main())
