#!/usr/bin/env python3
"""secreton must receive ONE shared JWT signing secret, from a Secret that survives upgrades.

WHY THIS EXISTS
---------------
`ApiConfig::from_bootstrap_and_application` used to map only the listeners and
storage; `auth.jwt` stayed at `JwtConfig::default()` — the literal string
`change-this-secret-in-production`, which is published in the source tree.
`AuthService::validate_token` accepts `token_type:"root"` WITHOUT consulting the
policy store, so anyone who could reach the HTTP port could mint a root token
and read or overwrite every secret. Nothing failed:

  * `helm lint`, `helm template` and kube-linter are all green;
  * the token still validates, so no suite goes red;
  * `SECRETON_JWT_SECRET` was documented but never read by any code.

Rules, read off the RENDER (so a workload added tomorrow is covered the day it exists).
The guard takes a MODE, because the two renders must differ:

 A. steady (`secrets.bootstrap=false`, the routine upgrade): Secret `secreton-jwt`
    must be ABSENT. `helm install/upgrade` writes every `data` it renders into the
    release Secret (`sh.helm.release.v1.*`), whose read-access is wider than a plain
    Secret's; the key signs root tokens, so a routine release must not carry it. The
    pod still references the Secret — the operator provisions it out-of-band (Sealed
    Secrets, SecretSync, a hand-created Secret; ADR-0001), exactly like `ghcr-pull`
    and `postgres-credentials` after bootstrap.
 A'. bootstrap (`secrets.bootstrap=true`, first install): Secret `secreton-jwt` is
    rendered, key `JWT_SECRET` is 64 lowercase hex (32 bytes), and it carries
    `helm.sh/resource-policy: keep` — the next upgrade drops it from the manifest and
    `keep` stops Helm deleting it, so the key survives and sessions are not cut.
    Its value must come from the operator: the chart must NOT originate it (`fail`
    otherwise), or the chart becomes the source of the root key.
 B. every container of the `secreton` workload sets `SECRETON_JWT_SECRET` from exactly
    that Secret/key, never as a literal `value` (a literal lands in the manifest, in
    Helm's release record and in `kubectl describe`);
 C. no other workload reads that Secret (the key signs every token);
 D. no workload other than secreton sets `SECRETON_JWT_SECRET` at all.

Input: JSON array of rendered objects (`yq -o=json eval-all '[.]'`, as the other render
guards use), plus the mode as the second argument (`steady` | `bootstrap`). `--self-test`
runs the rules against hand-built good and bad renders in both modes.
STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq.
"""
from __future__ import annotations

import base64
import binascii
import json
import re
import sys

SECRET = "secreton-jwt"
KEY = "SECRETON_JWT_SECRET"
OWN_KEY = {"secreton"}


def workloads(objs):
    for o in objs:
        if o.get("kind") in ("Deployment", "StatefulSet"):
            yield o["metadata"]["name"], o["spec"]["template"]["spec"]


def check(objs: list[dict], mode: str = "steady") -> list[str]:
    errs: list[str] = []

    secrets = [o for o in objs if o.get("kind") == "Secret" and o["metadata"]["name"] == SECRET]
    if mode == "steady":
        # The key signs root tokens; a routine release must not carry it into the
        # Helm release Secret. It is provisioned out-of-band (ADR-0001).
        if secrets:
            errs.append(
                f"A. Secret `{SECRET}` is rendered in a steady render — the root signing key "
                "must not enter the Helm release; gate it behind secrets.bootstrap and let the "
                "operator provision it out-of-band"
            )
    else:
        if not secrets:
            errs.append(f"A'. no Secret `{SECRET}` in the bootstrap render")
        for s in secrets:
            raw = (s.get("data") or {}).get("JWT_SECRET")
            if raw is None and "JWT_SECRET" in (s.get("stringData") or {}):
                raw = base64.b64encode(s["stringData"]["JWT_SECRET"].encode()).decode()
            try:
                val = base64.b64decode(raw or "", validate=True).decode()
            except (binascii.Error, UnicodeDecodeError, ValueError):
                val = ""
            if not re.fullmatch(r"[0-9a-f]{64}", val):
                errs.append(f"A'. Secret `{SECRET}` key `JWT_SECRET` is not 64 lowercase hex characters")
            ann = s["metadata"].get("annotations") or {}
            if ann.get("helm.sh/resource-policy") != "keep":
                errs.append(f"A'. Secret `{SECRET}` lacks `helm.sh/resource-policy: keep`")

    secreton_seen = False
    for name, pod in workloads(objs):
        for c in pod.get("containers", []):
            env = {e["name"]: e for e in c.get("env", []) if "name" in e}
            ref = ((env.get(KEY) or {}).get("valueFrom") or {}).get("secretKeyRef") or {}
            uses_secret = (
                ref.get("name") == SECRET
                or any(
                    (e.get("valueFrom") or {}).get("secretKeyRef", {}).get("name") == SECRET
                    for e in c.get("env", [])
                )
                or any(
                    (ef.get("secretRef") or {}).get("name") == SECRET
                    for ef in c.get("envFrom", [])
                )
            )
            if name == "secreton" and c.get("name") == "secreton":
                secreton_seen = True
                if KEY not in env:
                    errs.append(f"B. secreton container has no `{KEY}` env")
                elif "value" in env[KEY]:
                    errs.append(f"B. secreton sets `{KEY}` as a literal value, not from Secret `{SECRET}`")
                elif ref.get("name") != SECRET or ref.get("key") != "JWT_SECRET":
                    errs.append(f"B. secreton `{KEY}` comes from {ref}, expected {SECRET}/JWT_SECRET")
            else:
                if uses_secret:
                    errs.append(f"C. {name}/{c.get('name')} reads Secret `{SECRET}`")
                if KEY in env and name not in OWN_KEY:
                    errs.append(f"D. {name}/{c.get('name')} sets its own `{KEY}`")
    if not secreton_seen:
        errs.append("B. no `secreton` container in the render — guard is measuring nothing")
    return errs


def _workload() -> dict:
    return {"kind": "StatefulSet", "metadata": {"name": "secreton"}, "spec": {"template": {"spec": {"containers": [
        {"name": "secreton", "env": [{"name": KEY, "valueFrom": {"secretKeyRef": {"name": SECRET, "key": "JWT_SECRET"}}}]}]}}}}


def _secret() -> dict:
    key = base64.b64encode(("ab" * 32).encode()).decode()
    return {"kind": "Secret", "metadata": {"name": SECRET, "annotations": {"helm.sh/resource-policy": "keep"}},
            "data": {"JWT_SECRET": key}}


def _good(mode: str) -> list[dict]:
    # steady: the Secret is absent (operator-provisioned); bootstrap: it is rendered.
    return [_workload()] if mode == "steady" else [_secret(), _workload()]


def self_test() -> int:
    import copy

    def mutate(mode, fn):
        o = copy.deepcopy(_good(mode))
        fn(o)
        return o

    def env_of(o):
        return o[-1]["spec"]["template"]["spec"]["containers"][0]

    cases = {
        # steady: the key must not be in the release manifest
        ("steady", "good steady render passes"): (_good("steady"), False),
        ("steady", "Secret leaked into the steady render"): (mutate("steady", lambda o: o.insert(0, _secret())), True),
        ("steady", "secreton without SECRETON_JWT_SECRET"): (mutate("steady", lambda o: env_of(o).update(env=[])), True),
        ("steady", "secreton literal secret"): (mutate("steady", lambda o: env_of(o).update(
            env=[{"name": KEY, "value": "ab" * 32}])), True),
        ("steady", "secreton from the wrong Secret"): (mutate("steady", lambda o: env_of(o).update(
            env=[{"name": KEY, "valueFrom": {"secretKeyRef": {"name": "other", "key": "JWT_SECRET"}}}])), True),
        ("steady", "another workload reads the Secret"): (mutate("steady", lambda o: o.append(
            {"kind": "Deployment", "metadata": {"name": "portal"}, "spec": {"template": {"spec": {"containers": [
                {"name": "portal", "envFrom": [{"secretRef": {"name": SECRET}}]}]}}}})), True),
        ("steady", "another workload signs with its own key"): (mutate("steady", lambda o: o.append(
            {"kind": "Deployment", "metadata": {"name": "portal"}, "spec": {"template": {"spec": {"containers": [
                {"name": "portal", "env": [{"name": KEY, "value": "x"}]}]}}}})), True),
        ("steady", "no secreton at all"): (mutate("steady", lambda o: o.pop()), True),
        # bootstrap: the Secret is rendered, operator-supplied, and kept
        ("bootstrap", "good bootstrap render passes"): (_good("bootstrap"), False),
        ("bootstrap", "Secret missing"): (mutate("bootstrap", lambda o: o.pop(0)), True),
        ("bootstrap", "Secret not hex"): (mutate("bootstrap", lambda o: o[0].update(data={"JWT_SECRET": base64.b64encode(b"hunter2").decode()})), True),
        ("bootstrap", "Secret not hex (uppercase)"): (mutate("bootstrap", lambda o: o[0].update(data={"JWT_SECRET": base64.b64encode(("AB" * 32).encode()).decode()})), True),
        ("bootstrap", "Secret without keep"): (mutate("bootstrap", lambda o: o[0]["metadata"].update(annotations={})), True),
        ("bootstrap", "secreton without SECRETON_JWT_SECRET"): (mutate("bootstrap", lambda o: env_of(o).update(env=[])), True),
    }
    bad = 0
    for (mode, label), (objs, should_fail) in cases.items():
        failed = bool(check(objs, mode))
        ok = failed == should_fail
        bad += not ok
        print(f"{'ok  ' if ok else 'FAIL'} self-test [{mode}]: {label} -> {'rejected' if failed else 'accepted'}")
    return 1 if bad else 0


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        return self_test()
    label = sys.argv[1] if len(sys.argv) > 1 else "?"
    mode = sys.argv[2] if len(sys.argv) > 2 else "steady"
    if mode not in ("steady", "bootstrap"):
        print(f"[{label}] FAIL: unknown mode '{mode}' (expected steady | bootstrap)")
        return 1
    objs = [o for o in json.load(sys.stdin) if isinstance(o, dict)]
    errs = check(objs, mode)
    for e in errs:
        print(f"[{label}] FAIL: {e}")
    if not errs:
        detail = ("absent from the release manifest (operator-provisioned)"
                  if mode == "steady" else "present, 64 hex, kept across upgrades")
        print(f"[{label}] ok [{mode}]: secreton takes SECRETON_JWT_SECRET from Secret `{SECRET}` ({detail})")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
