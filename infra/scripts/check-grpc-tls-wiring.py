#!/usr/bin/env python3
"""With backendConfig.grpcTls.enabled, every gRPC server and every gRPC client must
actually be wired for mTLS — derived from the RENDERED chart, not from a list.

WHY THIS EXISTS
---------------
mTLS here is spread over five places that must agree: the PKI (a Secret per
identity), the server's GRPC_TLS_* env, the client's GRPC_CLIENT_* env, the
https:// scheme of every *_GRPC_URL, and the Service port's appProtocol. Getting
any one wrong does not fail `helm lint`, `helm template` or kube-linter, and the
failure it produces is the worst kind:

  * a server without material      -> authenc/secreton refuse to start (production)
                                      or, elsewhere, quietly serve PLAINTEXT;
  * a client that dials https://   -> without credentials: every call fails;
  * a client left on http://       -> talks to a TLS port: every call fails, with a
                                      transport error that names neither cause;
  * appProtocol still `grpc`       -> Envoy parses TLS bytes as HTTP/2 and cuts the
                                      connection.

A future workload that starts dialling a gRPC service (the next `*_GRPC_URL`) is
caught here the day it is added, because the rules are read off the render:

 A. every `*_GRPC_URL` in backend-config is https://;
 B. a container with GRPC_TLS_CERT_PATH (a server) has KEY and CA too;
 C. a container with any GRPC_CLIENT_*_PATH has all three;
 D. a container that sets a `*_GRPC_URL` env itself (a dialler) has GRPC_CLIENT_*;
 E. every such path lies under a volumeMount backed by an EXISTING `grpc-tls-*`
    Secret, and no container mounts key material it does not use;
 F. no Service port is left `appProtocol: grpc`, and every `tls` gRPC port belongs
    to a workload that serves with GRPC_TLS_*;
 G. every `grpc-tls-*` Secret is consumed by someone (no orphaned private keys).

Input: JSON array of the rendered objects (the `yq -o=json eval-all '[.]'` the other
render guards use) rendered WITH `--set backendConfig.grpcTls.enabled=true`.
STDLIB ONLY: the ARC runner image ships neither PyYAML nor yq.
"""
from __future__ import annotations

import json
import posixpath
import sys

SERVE = ("GRPC_TLS_CERT_PATH", "GRPC_TLS_KEY_PATH", "GRPC_TLS_CA_PATH")
DIAL = ("GRPC_CLIENT_CERT_PATH", "GRPC_CLIENT_KEY_PATH", "GRPC_CLIENT_CA_PATH")


def pods(objs):
    for o in objs:
        k = o.get("kind")
        if k in ("Deployment", "StatefulSet"):
            yield o, o["spec"]["template"]


def main() -> int:
    label = sys.argv[1] if len(sys.argv) > 1 else "?"
    objs = [o for o in json.load(sys.stdin) if isinstance(o, dict)]
    errs: list[str] = []

    secrets = {o["metadata"]["name"] for o in objs if o.get("kind") == "Secret"}
    if "grpc-pki-ca" not in secrets:
        print(f"[{label}] FAIL: no grpc-pki-ca Secret in the render — it was rendered "
              "WITHOUT --set backendConfig.grpcTls.enabled=true, so nothing was checked.")
        return 1
    tls_secrets = {n for n in secrets if n.startswith("grpc-tls-")}
    consumed: set[str] = set()

    # A
    for o in objs:
        if o.get("kind") == "ConfigMap" and o["metadata"]["name"] == "backend-config":
            urls = {k: v for k, v in o.get("data", {}).items() if k.endswith("_GRPC_URL")}
            if not urls:
                errs.append("backend-config has no *_GRPC_URL — guard is measuring nothing")
            for k, v in urls.items():
                if not str(v).startswith("https://"):
                    errs.append(f"A. backend-config {k}={v} is not https://")

    servers: set[str] = set()   # app.kubernetes.io/name of workloads that serve gRPC TLS
    for o, tpl in pods(objs):
        wname = o["metadata"]["name"]
        spec = tpl["spec"]
        vols = {v["name"]: v for v in spec.get("volumes", [])}
        for c in spec.get("containers", []):
            env = {e["name"]: e for e in c.get("env", [])}
            mounts = {m["name"]: m for m in c.get("volumeMounts", [])}
            who = f"{wname}/{c['name']}"
            serves = any(n in env for n in SERVE)
            dials_env = any(n in env for n in DIAL)
            dialler = any(n.endswith("_GRPC_URL") for n in env)

            if serves and not all(n in env for n in SERVE):
                errs.append(f"B. {who} sets some GRPC_TLS_* but not all three")
            if dials_env and not all(n in env for n in DIAL):
                errs.append(f"C. {who} sets some GRPC_CLIENT_* but not all three")
            if dialler and not all(n in env for n in DIAL):
                errs.append(f"D. {who} dials gRPC ({', '.join(sorted(n for n in env if n.endswith('_GRPC_URL')))}) "
                            "but has no GRPC_CLIENT_{CERT,KEY,CA}_PATH — every call would fail")
            if serves:
                servers.add(o["metadata"]["labels"].get("app.kubernetes.io/name", wname))

            # E: paths resolve to a mounted, existing grpc-tls Secret
            used = [env[n].get("value", "") for n in SERVE + DIAL if n in env]
            for path in used:
                d = posixpath.dirname(path)
                hit = [m for m in mounts.values() if m["mountPath"] == d]
                if not hit:
                    errs.append(f"E. {who}: {path} is not under any volumeMount")
                    continue
                vol = vols.get(hit[0]["name"], {})
                sec = vol.get("secret", {}).get("secretName", "")
                if sec not in tls_secrets:
                    errs.append(f"E. {who}: {path} is mounted from '{sec or vol}', not an existing grpc-tls-* Secret")
                else:
                    consumed.add(sec)
            for mname, m in mounts.items():
                sec = vols.get(mname, {}).get("secret", {}).get("secretName", "")
                if sec.startswith("grpc-tls-") and not (serves or dials_env):
                    errs.append(f"E. {who} mounts {sec} but uses no GRPC_* path — private key distributed for nothing")

    # F
    for o in objs:
        if o.get("kind") != "Service":
            continue
        for p in o["spec"].get("ports", []):
            proto = p.get("appProtocol")
            if proto == "grpc":
                errs.append(f"F. Service {o['metadata']['name']}:{p['port']} is still appProtocol 'grpc' — Envoy would parse TLS as HTTP/2")
            if proto == "tls":
                sel = o["spec"].get("selector", {}).get("app.kubernetes.io/name")
                if sel not in servers:
                    errs.append(f"F. Service {o['metadata']['name']}:{p['port']} is appProtocol 'tls' but its workload '{sel}' sets no GRPC_TLS_*")

    # G
    for s in sorted(tls_secrets - consumed):
        errs.append(f"G. Secret {s} is not consumed by any workload")

    if errs:
        print(f"[{label}] FAIL: mTLS gRPC is not wired consistently:")
        for e in errs:
            print("  - " + e)
        return 1
    print(f"[{label}] OK: {len(servers)} gRPC server workload(s), {len(consumed)} grpc-tls Secret(s) consumed, "
          "all *_GRPC_URL https://, no appProtocol 'grpc' left.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
