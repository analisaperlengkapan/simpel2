#!/usr/bin/env python3
"""Fail when SIMPEL v1's own CSP refuses a CDN its pages load from.

v1's views load DataTables, select2, SweetAlert, pdfmake, jszip, CKEditor 4, Google Maps
and reCAPTCHA from public CDNs. The containerisation (e41b5b95, 2026-04-30) gave
monolith/simpelv1/nginx.conf a CSP whose script-src is `'self'` only. Every one of those
scripts was refused. On the dashboard alone that is 19 refusals, and DataTables backs
144 views.

A refused script leaves no trace on the server and no message on screen. The table
simply never becomes a DataTable. The cross-app SSO e2e looked at server-rendered HTML,
so it passed.

WHAT THIS CHECKS:
  * every host a Blade view names in `<script src="https://...">` is in script-src,
    and every host in a stylesheet `<link href="https://...">` is in style-src;
  * the same for `<script src='https://...'>` tags that v1's own scripts
    (public/assets/js/*.js) write at run time;
  * hosts those libraries pull in themselves (TRANSITIVE below);
  * both copies of the CSP in nginx.conf (server block and static-asset location; nginx
    does not inherit add_header) are identical.
Blade and HTML comments are removed first.

WHAT IT DOES NOT CHECK: whether the CDN is reachable from users' networks, or
connect-src/img-src (both already allow https:).

Usage: check-simpelv1-csp-allows-cdn.py [--self-test]
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
V1 = "monolith/simpelv1"
NGINX = f"{V1}/nginx.conf"

# Hosts a library loads from on its own, which no view names.
TRANSITIVE: dict[str, list[tuple[str, str, str]]] = {
    "www.google.com": [
        ("script-src", "www.gstatic.com", "reCAPTCHA's api.js/enterprise.js load the rest from gstatic"),
        ("frame-src", "www.google.com", "reCAPTCHA renders its challenge in an iframe"),
    ],
    "cdn.ckeditor.com": [
        ("style-src", "cdn.ckeditor.com", "CKEditor 4 loads its skin CSS from the host it came from"),
    ],
}

BLADE_COMMENT = re.compile(r"\{\{--.*?--\}\}", re.S)
HTML_COMMENT = re.compile(r"<!--.*?-->", re.S)
SCRIPT_TAG = re.compile(r"""<script\b[^>]*?\bsrc\s*=\s*['"]https?://([^/'"\s?#]+)""", re.I)
LINK_TAG = re.compile(r"""<link\b[^>]*>""", re.I)
LINK_HREF = re.compile(r"""\bhref\s*=\s*['"]https?://([^/'"\s?#]+)([^'"]*)""", re.I)
CSP_LINE = re.compile(r"""add_header\s+Content-Security-Policy\s+"([^"]+)\"""")


def strip_comments(text: str) -> str:
    return HTML_COMMENT.sub("", BLADE_COMMENT.sub("", text))


def hosts_in(text: str) -> dict[str, set[str]]:
    """{directive: {host}} for the external scripts and stylesheets a text loads."""
    text = strip_comments(text)
    out: dict[str, set[str]] = {"script-src": set(), "style-src": set()}
    out["script-src"].update(m.group(1).lower() for m in SCRIPT_TAG.finditer(text))
    for tag in LINK_TAG.finditer(text):
        t = tag.group(0)
        href = LINK_HREF.search(t)
        if not href:
            continue
        is_style = re.search(r"""rel\s*=\s*['"]?stylesheet""", t, re.I) or href.group(2).split("?")[0].endswith(".css")
        if is_style:
            out["style-src"].add(href.group(1).lower())
    return out


def parse_csp(policy: str) -> dict[str, set[str]]:
    out: dict[str, set[str]] = {}
    for part in policy.split(";"):
        words = part.split()
        if words:
            out[words[0]] = set(words[1:])
    return out


def allows(csp: dict[str, set[str]], directive: str, host: str) -> bool:
    sources = csp.get(directive)
    if sources is None and directive != "default-src":
        sources = csp.get("default-src", set())
    return f"https://{host}" in (sources or set()) or "https:" in (sources or set())


def findings(needed: dict[tuple[str, str], set[str]], policies: list[str]) -> list[str]:
    """`needed` maps (directive, host) to the files that need it."""
    out = []
    if not policies:
        return [f"no `add_header Content-Security-Policy` found in {NGINX}"]
    if len(set(policies)) != 1:
        out.append(f"{NGINX} carries {len(set(policies))} different CSPs; every copy must be identical")
    csp = parse_csp(policies[0])
    for (directive, host), where in sorted(needed.items()):
        if not allows(csp, directive, host):
            out.append(f"{directive} does not allow https://{host}, needed by {', '.join(sorted(where))}")
    return out


def git_files(*paths: str) -> list[str]:
    res = subprocess.run(
        ["git", "-C", str(REPO), "ls-files", "-z", "--", *paths], check=True, capture_output=True
    )
    return [p for p in res.stdout.decode().split("\0") if p]


def collect() -> tuple[dict[tuple[str, str], set[str]], list[str]]:
    needed: dict[tuple[str, str], set[str]] = {}

    def add(found: dict[str, set[str]], source: str) -> None:
        for directive, hosts in found.items():
            for h in hosts:
                needed.setdefault((directive, h), set()).add(source)
                for d2, h2, why in TRANSITIVE.get(h, []):
                    needed.setdefault((d2, h2), set()).add(f"{source} (via {h}: {why})")

    for f in git_files(f"{V1}/resources/views"):
        if f.endswith(".blade.php"):
            add(hosts_in((REPO / f).read_text(encoding="utf-8", errors="replace")), f)
    base = f"{V1}/public/assets/js/"
    for f in git_files(base):
        if f.endswith(".js") and "/" not in f[len(base):]:
            add(hosts_in((REPO / f).read_text(encoding="utf-8", errors="replace")), f)
    policies = CSP_LINE.findall((REPO / NGINX).read_text(encoding="utf-8"))
    return needed, policies


def self_test() -> int:
    view = """
      <script src="https://cdn.datatables.net/1.11.5/js/jquery.dataTables.min.js"></script>
      <script async defer src="https://maps.googleapis.com/maps/api/js?key={{ $k }}"></script>
      <link href="https://cdn.jsdelivr.net/npm/select2/dist/css/select2.min.css" rel="stylesheet" />
      <link rel="preconnect" href="https://fonts.gstatic.com">
      {{-- <script src="https://commented.example/x.js"></script> --}}
      <!-- <link rel="stylesheet" href="https://html-commented.example/x.css"> -->
    """
    got = hosts_in(view)
    assert got["script-src"] == {"cdn.datatables.net", "maps.googleapis.com"}, got
    assert got["style-src"] == {"cdn.jsdelivr.net"}, got
    js = "document.writeln(\"<script type='text/javascript' src='https://cdn.jsdelivr.net/npm/toastify-js'></script>\")"
    assert hosts_in(js)["script-src"] == {"cdn.jsdelivr.net"}, hosts_in(js)

    pol = "default-src 'self'; script-src 'self' https://cdn.datatables.net; style-src 'self'"
    need = {("script-src", "cdn.datatables.net"): {"v"}}
    assert findings(need, [pol, pol]) == [], "allowed"
    bad = findings({("style-src", "cdn.jsdelivr.net"): {"v"}}, [pol])
    assert len(bad) == 1 and "style-src does not allow https://cdn.jsdelivr.net" in bad[0], bad
    # A directive that is absent falls back to default-src.
    assert findings({("frame-src", "www.google.com"): {"v"}}, [pol]) != [], "frame-src falls back"
    assert any("different CSPs" in f for f in findings(need, [pol, pol + " https://x"])), "copies differ"
    print("self-test ok")
    return 0


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    needed, policies = collect()
    if not needed:
        print(f"::error::no external script or stylesheet found under {V1}: the scan itself is broken")
        return 1
    problems = findings(needed, policies)
    for p in problems:
        print(f"::error::{p}")
    if problems:
        print(f"{len(problems)} problem(s). A refused script fails silently in the browser.")
        return 1
    hosts = sorted({h for (_, h) in needed})
    print(f"ok: v1's CSP allows all {len(needed)} (directive, host) pair(s) its pages need: {', '.join(hosts)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
