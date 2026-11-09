#!/usr/bin/env python3
"""authenc must receive ONE shared JWT signing key, from a Secret that survives upgrades.

WHY THIS EXISTS
---------------
authenc signs and verifies tokens with a single key. Without `JWT_SECRET` each process
draws its own random key, so with two replicas a token minted by one is rejected by the
other, and every restart ends every session. The chart never set the variable; authenc
only logged "not for production" and carried on, and nothing failed:

  * `helm lint`, `helm template` and kube-linter are all green;
  * the staging suite passed for months because the HPA stayed at ONE replica;
  * rc40's HPA went to two mid-run and 20 of 218 e2e tests failed with
    `Token validation failed` on tokens minted seconds earlier.

Rules, read off the RENDER (so a workload added tomorrow is covered the day it exists):

 A. Secret `authenc-jwt` is rendered, key `JWT_SECRET` is 64 lowercase hex (32 bytes),
    and it carries `helm.sh/resource-policy: keep` (a rollback or uninstall that deleted
    it would end every session and make the next pod draw a fresh key);
 B. every container of the `authenc` workload sets `JWT_SECRET` from exactly that
    Secret/key, never as a literal `value` (a literal lands in the manifest, in Helm's
    release record and in `kubectl describe`);
 C. no other workload reads that Secret (the key signs every user's session);
 D. no workload other than authenc and simpelv1 sets `JWT_SECRET` at all — a second
    signer with its own key is how the next version of this bug would arrive.

Input: JSON array of rendered objects (`yq -o=json eval-all '[.]'`, as the other render
guards use). `--self-test` runs the rules against hand-built good and bad renders.
STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq.
"""
from __future__ import annotations

import base64
import binascii
import json
import re
import sys

SECRET = "authenc-jwt"
KEY = "JWT_SECRET"
OWN_KEY = {"authenc", "simpelv1"}  # simpelv1 (Laravel) signs its own `api` guard with its own key


def workloads(objs):
    for o in objs:
        if o.get("kind") in ("Deployment", "StatefulSet"):
            yield o["metadata"]["name"], o["spec"]["template"]["spec"]


def check(objs: list[dict]) -> list[str]:
    errs: list[str] = []

    secrets = [o for o in objs if o.get("kind") == "Secret" and o["metadata"]["name"] == SECRET]
    if not secrets:
        errs.append(f"A. no Secret `{SECRET}` in the render")
    for s in secrets:
        raw = (s.get("data") or {}).get(KEY)
        if raw is None and KEY in (s.get("stringData") or {}):
            raw = base64.b64encode(s["stringData"][KEY].encode()).decode()
        try:
            val = base64.b64decode(raw or "", validate=True).decode()
        except (binascii.Error, UnicodeDecodeError, ValueError):
            val = ""
        if not re.fullmatch(r"[0-9a-f]{64}", val):
            errs.append(f"A. Secret `{SECRET}` key `{KEY}` is not 64 lowercase hex characters")
        ann = s["metadata"].get("annotations") or {}
        if ann.get("helm.sh/resource-policy") != "keep":
            errs.append(f"A. Secret `{SECRET}` lacks `helm.sh/resource-policy: keep`")

    authenc_seen = False
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
            if name == "authenc" and c.get("name") == "authenc":
                authenc_seen = True
                if KEY not in env:
                    errs.append(f"B. authenc container has no `{KEY}` env")
                elif "value" in env[KEY]:
                    errs.append(f"B. authenc sets `{KEY}` as a literal value, not from Secret `{SECRET}`")
                elif ref.get("name") != SECRET or ref.get("key") != KEY:
                    errs.append(f"B. authenc `{KEY}` comes from {ref}, expected {SECRET}/{KEY}")
            else:
                if uses_secret:
                    errs.append(f"C. {name}/{c.get('name')} reads Secret `{SECRET}`")
                if KEY in env and name not in OWN_KEY:
                    errs.append(f"D. {name}/{c.get('name')} sets its own `{KEY}`")
    if not authenc_seen:
        errs.append("B. no `authenc` container in the render — guard is measuring nothing")
    return errs


def _good() -> list[dict]:
    key = base64.b64encode(("ab" * 32).encode()).decode()
    return [
        {"kind": "Secret", "metadata": {"name": SECRET, "annotations": {"helm.sh/resource-policy": "keep"}},
         "data": {KEY: key}},
        {"kind": "Deployment", "metadata": {"name": "authenc"}, "spec": {"template": {"spec": {"containers": [
            {"name": "authenc", "env": [{"name": KEY, "valueFrom": {"secretKeyRef": {"name": SECRET, "key": KEY}}}]}]}}}},
        {"kind": "Deployment", "metadata": {"name": "simpelv1"}, "spec": {"template": {"spec": {"containers": [
            {"name": "simpelv1", "env": [{"name": KEY, "valueFrom": {"secretKeyRef": {"name": "simpelv1-secrets", "key": KEY}}}]}]}}}},
    ]


def self_test() -> int:
    import copy

    def mutate(fn):
        o = copy.deepcopy(_good())
        fn(o)
        return o

    cases = {
        "good render passes": (_good(), False),
        "authenc without JWT_SECRET": (mutate(lambda o: o[1]["spec"]["template"]["spec"]["containers"][0].update(env=[])), True),
        "authenc literal JWT_SECRET": (mutate(lambda o: o[1]["spec"]["template"]["spec"]["containers"][0].update(
            env=[{"name": KEY, "value": "ab" * 32}])), True),
        "authenc from the wrong Secret": (mutate(lambda o: o[1]["spec"]["template"]["spec"]["containers"][0].update(
            env=[{"name": KEY, "valueFrom": {"secretKeyRef": {"name": "other", "key": KEY}}}])), True),
        "Secret missing": (mutate(lambda o: o.pop(0)), True),
        "Secret not hex": (mutate(lambda o: o[0].update(data={KEY: base64.b64encode(b"hunter2").decode()})), True),
        "Secret not hex (uppercase)": (mutate(lambda o: o[0].update(data={KEY: base64.b64encode(("AB" * 32).encode()).decode()})), True),
        "Secret without keep": (mutate(lambda o: o[0]["metadata"].update(annotations={})), True),
        "another workload reads the Secret": (mutate(lambda o: o.append(
            {"kind": "Deployment", "metadata": {"name": "portal"}, "spec": {"template": {"spec": {"containers": [
                {"name": "portal", "envFrom": [{"secretRef": {"name": SECRET}}]}]}}}})), True),
        "another workload signs with its own key": (mutate(lambda o: o.append(
            {"kind": "Deployment", "metadata": {"name": "portal"}, "spec": {"template": {"spec": {"containers": [
                {"name": "portal", "env": [{"name": KEY, "value": "x"}]}]}}}})), True),
        "no authenc at all": (mutate(lambda o: o.pop(1)), True),
    }
    bad = 0
    for label, (objs, should_fail) in cases.items():
        failed = bool(check(objs))
        ok = failed == should_fail
        bad += not ok
        print(f"{'ok  ' if ok else 'FAIL'} self-test: {label} -> {'rejected' if failed else 'accepted'}")
    return 1 if bad else 0


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        return self_test()
    label = sys.argv[1] if len(sys.argv) > 1 else "?"
    objs = [o for o in json.load(sys.stdin) if isinstance(o, dict)]
    errs = check(objs)
    for e in errs:
        print(f"[{label}] FAIL: {e}")
    if not errs:
        print(f"[{label}] ok: authenc takes JWT_SECRET from Secret `{SECRET}` (64 hex, kept across rollbacks)")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())
