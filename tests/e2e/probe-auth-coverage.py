#!/usr/bin/env python3
"""Prove no perlengkapan route answers an unauthenticated caller.

Walks every mounted path from routes.rs, substitutes a UUID for each `{param}`,
and probes it with a bare GET/POST. The gate here is the `Claims` extractor, and
a route that lost it is reachable straight from the internet via the Istio
gateway (`hosts: ["*"]`) — not merely from inside the cluster. So the only
acceptable answers are 401/403 (or 404 when the probe's path param does not
resolve before auth runs, and 405 for a method mismatch).

Run against a live stack: python3 tests/e2e/probe-auth-coverage.py
"""
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BASE = "http://127.0.0.1:3020"
PREFIX = "/api/v1/perlengkapan"
ROUTES_RS = REPO / "layanan/perlengkapan/src/routes.rs"

routes = re.findall(r'\.route\(\s*"([^"]+)"', ROUTES_RS.read_text())
UUID = "00000000-0000-4000-8000-000000000001"

def concrete(path: str) -> str:
    return re.sub(r"\{[^}]+\}", UUID, path)

public = []
for r in sorted(set(routes)):
    url = BASE + PREFIX + concrete(r)
    for method in ("GET", "POST"):
        req = urllib.request.Request(url, method=method)
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                code = resp.status
        except urllib.error.HTTPError as e:
            code = e.code
        except Exception as e:  # noqa: BLE001 - network/protocol noise is expected
            code = f"ERR:{type(e).__name__}"
        if code == 200 or code == 201:
            public.append((r, method, code))
            print(f"  PUBLIC  {method:5s} {r} -> {code}")

print(f"\nprobed {len(set(routes))} routes")
if public:
    print(f"UNAUTHENTICATED 2xx on {len(public)} route(s):")
    for r, m, c in public:
        print(f"  {m} {r} -> {c}")
    sys.exit(1)
print("ok: no route answered an unauthenticated request with 2xx")
