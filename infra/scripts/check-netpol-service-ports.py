#!/usr/bin/env python3
"""Setiap port di NetworkPolicy harus port Service yang nyata, dan setiap
URL in-cluster yang dikonfigurasi chart harus benar-benar diizinkan lewat.

Dua arah, karena integrasi putus lewat KEDUANYA sekaligus (2026-08-28):

  A. `allow-incluster-egress` mengizinkan egress ke `layanan-integrasi` pada
     port **50053**. Tak ada yang mendengarkan 50053 — service-nya 50051/8080.
     Karena policy itu memakai `podSelector: {}`, SETIAP pod jadi
     egress-restricted, jadi satu digit salah memutus integrasi untuk seluruh
     namespace.

  B. Tak pernah ada policy Ingress untuk `layanan-integrasi`. authenc,
     postgres, secreton, dan redis masing-masing punya; integrasi terlewat,
     jadi `default-deny-ingress` menutupnya rapat.

Keduanya lolos `helm lint`, `helm template`, dan kube-linter: manifesnya sah,
hanya salah. Yang membuatnya mahal adalah gagalnya SENYAP — `is_integrasi_sync_risky`
memperlakukan kegagalan probe sebagai "tidak memblokir", jadi tiap keputusan
Validator Pusat diam-diam menunggu 25s + 30s sampai time out lalu lanjut. Klien
menerima 504 dari gateway (batas 30 detik) walau transisinya berhasil.

Cakupan DITURUNKAN dari manifes yang dirender, bukan daftar tulis tangan:
service + port-nya dari objek Service, pasangan yang dibutuhkan dari URL yang
benar-benar dikonfigurasi. Service baru ikut tercakup pada hari ia ditambahkan.

Pakai: yq -o=json eval-all '[.]' < rendered.yaml | check-netpol-service-ports.py <label>
"""
import json
import re
import sys

NAME = "app.kubernetes.io/name"
COMPONENT = "app.kubernetes.io/component"
# host:port di URL in-cluster — `svc`, `svc.ns`, atau `svc.ns.svc.cluster.local`
URL_RE = re.compile(r"(?:^|//)([a-z0-9][a-z0-9-]*)(?:\.[a-z0-9-]+)*?(?:\.svc\.cluster\.local)?:(\d{2,5})\b")


def docs(blob):
    for d in blob:
        if isinstance(d, dict) and d.get("kind"):
            yield d


def collect_services(ds):
    """name-label -> {ports}, dan nama Service -> name-label."""
    ports, alias = {}, {}
    for d in ds:
        if d.get("kind") != "Service":
            continue
        sel = (d.get("spec") or {}).get("selector") or {}
        label = sel.get(NAME)
        if not label:
            continue
        svc_name = (d.get("metadata") or {}).get("name", "")
        got = set()
        for p in (d.get("spec") or {}).get("ports") or []:
            if isinstance(p.get("port"), int):
                got.add(p["port"])
        ports.setdefault(label, set()).update(got)
        if svc_name:
            alias[svc_name] = label
    return ports, alias


def netpol_rules(ds):
    """(direction, target-name-label, port) yang policy sebutkan."""
    out = []
    for d in ds:
        if d.get("kind") != "NetworkPolicy":
            continue
        spec = d.get("spec") or {}
        meta = (d.get("metadata") or {}).get("name", "?")
        own = ((spec.get("podSelector") or {}).get("matchLabels") or {}).get(NAME)
        for direction, key in (("Ingress", "ingress"), ("Egress", "egress")):
            for rule in spec.get(direction.lower()) or spec.get(key) or []:
                pts = [p["port"] for p in rule.get("ports") or [] if isinstance(p.get("port"), int)]
                if direction == "Ingress":
                    # ingress: yang ditembak = pemilik policy
                    for port in pts:
                        out.append((meta, "Ingress", own, port, rule))
                else:
                    for peer in rule.get("to") or []:
                        tgt = ((peer.get("podSelector") or {}).get("matchLabels") or {}).get(NAME)
                        if tgt:
                            for port in pts:
                                out.append((meta, "Egress", tgt, port, rule))
    return out


def required_pairs(ds, alias):
    """(name-label, port) yang benar-benar dipakai konsumen, dari URL terkonfigurasi."""
    need = {}
    def scan(text, origin):
        for host, port in URL_RE.findall(str(text)):
            label = alias.get(host)
            if label:
                need.setdefault((label, int(port)), set()).add(origin)
    for d in ds:
        kind = d.get("kind")
        meta = (d.get("metadata") or {}).get("name", "?")
        if kind == "ConfigMap":
            for k, v in (d.get("data") or {}).items():
                scan(v, f"ConfigMap/{meta}:{k}")
        elif kind in ("Deployment", "StatefulSet", "Job", "DaemonSet"):
            pod = ((d.get("spec") or {}).get("template") or {}).get("spec") or {}
            for c in (pod.get("containers") or []) + (pod.get("initContainers") or []):
                for e in c.get("env") or []:
                    if isinstance(e.get("value"), str):
                        scan(e["value"], f"{kind}/{meta}:{e.get('name')}")
    return need


def main():
    label = sys.argv[1] if len(sys.argv) > 1 else "chart"
    ds = list(docs(json.load(sys.stdin)))
    svc_ports, alias = collect_services(ds)
    rules = netpol_rules(ds)
    problems = []

    # ── A. tiap port yang disebut policy harus port Service yang nyata ──────
    for pol, direction, target, port, _rule in rules:
        if target is None or target not in svc_ports:
            continue  # menargetkan sesuatu yang bukan Service ber-label; bukan urusan cek ini
        if port not in svc_ports[target]:
            problems.append(
                f"NetworkPolicy/{pol} ({direction}) membuka port {port} untuk "
                f"`{target}`, tetapi Service-nya hanya mengekspos "
                f"{sorted(svc_ports[target])}. Tak ada yang mendengarkan {port}."
            )

    # ── B. tiap URL in-cluster terkonfigurasi harus benar-benar diizinkan ───
    # Egress hanya membatasi bila ada policy Egress ber-podSelector {} (yang
    # menyeret SEMUA pod ke rezim egress-restricted).
    egress_is_restricting = any(
        d.get("kind") == "NetworkPolicy"
        and "Egress" in ((d.get("spec") or {}).get("policyTypes") or [])
        and not (((d.get("spec") or {}).get("podSelector") or {}).get("matchLabels"))
        for d in ds
    )
    deny_ingress = any(
        d.get("kind") == "NetworkPolicy"
        and (d.get("metadata") or {}).get("name") == "default-deny-ingress"
        for d in ds
    )

    def workload_peer(rule):
        """True bila rule membuka jalan untuk pod di dalam namespace (bukan hanya gateway)."""
        for peer in rule.get("from") or []:
            ml = (peer.get("podSelector") or {}).get("matchLabels") or {}
            if ml.get(COMPONENT) in ("backend", "frontend") or ml.get(NAME):
                if not peer.get("namespaceSelector"):
                    return True
        return False

    for (target, port), origins in sorted(required_pairs(ds, alias).items()):
        why = ", ".join(sorted(origins))
        if deny_ingress and not any(
            d == "Ingress" and t == target and p == port and workload_peer(r)
            for _pol, d, t, p, r in rules
        ):
            problems.append(
                f"`{target}:{port}` dipakai oleh {why}, tetapi tak ada NetworkPolicy "
                f"Ingress yang membuka port itu dari pod di dalam namespace. "
                f"`default-deny-ingress` akan menelan koneksinya."
            )
        if egress_is_restricting and not any(
            d == "Egress" and t == target and p == port for _pol, d, t, p, _r in rules
        ):
            problems.append(
                f"`{target}:{port}` dipakai oleh {why}, tetapi tak ada aturan "
                f"Egress ke port itu. Policy Egress ber-podSelector {{}} membuat "
                f"SETIAP pod egress-restricted, jadi koneksinya time out."
            )

    if problems:
        print(f"[{label}] NetworkPolicy tidak cocok dengan Service/URL chart:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        sys.exit(1)
    print(f"[{label}] NetworkPolicy cocok dengan port Service dan URL terkonfigurasi.")


if __name__ == "__main__":
    main()
