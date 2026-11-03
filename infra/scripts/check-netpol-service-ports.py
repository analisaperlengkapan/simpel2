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

  C. Policy Ingress `allow-backend-to-integrasi` mengizinkan pod berkomponen
     `backend` dan `frontend`. authenc berkomponen `identity`, jadi SETIAP
     panggilan gRPC authenc -> integrasi dibuang diam-diam sampai kehabisan
     waktu (2026-11-03): `/auth/me` memanggil integrasi dua kali berurutan
     (nama satker, lalu foto pegawai), masing-masing menunggu ~5 detik, dan
     gateway memutus di 10 detik dengan 504. Cek A dan B tak menangkapnya
     karena keduanya bertanya "apakah port itu terbuka untuk pod MANA PUN di
     namespace" dan hanya membaca URL literal; URL integrasi authenc datang dari
     ConfigMap (`configMapKeyRef`). Cek C bertanya per KONSUMEN: pod yang
     benar-benar memakai URL itu, apakah labelnya dicakup salah satu peer
     `from` pada policy yang membuka port tersebut.

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


def pod_template(d):
    """(label-map, containers) dari template pod sebuah workload; None bila bukan workload."""
    kind = d.get("kind")
    spec = d.get("spec") or {}
    if kind in ("Deployment", "StatefulSet", "Job", "DaemonSet"):
        tpl = spec.get("template") or {}
    elif kind == "CronJob":
        tpl = (((spec.get("jobTemplate") or {}).get("spec") or {}).get("template")) or {}
    else:
        return None
    pod = tpl.get("spec") or {}
    labels = (tpl.get("metadata") or {}).get("labels") or {}
    return labels, (pod.get("containers") or []) + (pod.get("initContainers") or [])


def consumer_pairs(ds, alias):
    """{workload: (pod-labels, {(name-label, port)})} — apa yang SETIAP workload benar-benar pakai.

    URL diambil dari variabel yang disebut satu per satu: `env.value` literal dan
    `env.valueFrom.configMapKeyRef` (URL integrasi milik authenc datang dari yang
    kedua, itulah mengapa cek lama tak melihatnya).
    """
    cms = {
        (d.get("metadata") or {}).get("name"): (d.get("data") or {})
        for d in ds
        if d.get("kind") == "ConfigMap"
    }
    out = {}
    for d in ds:
        tpl = pod_template(d)
        if tpl is None:
            continue
        labels, containers = tpl
        own = labels.get(NAME)
        texts = []
        for c in containers:
            for e in c.get("env") or []:
                if isinstance(e.get("value"), str):
                    texts.append(e["value"])
                ref = ((e.get("valueFrom") or {}).get("configMapKeyRef")) or {}
                val = (cms.get(ref.get("name")) or {}).get(ref.get("key"))
                if isinstance(val, str):
                    texts.append(val)
            # `envFrom` sengaja TIDAK dibaca: ia menyalin SELURUH ConfigMap ke
            # pod, jadi "pod menerima URL" tak berarti "pod memanggil URL" dan
            # setiap pod menjadi konsumen setiap service. Yang dibaca hanya
            # variabel yang DISEBUT satu per satu (literal atau configMapKeyRef):
            # di situlah seseorang memutuskan pod ini butuh service itu.
        need = set()
        for t in texts:
            for host, port in URL_RE.findall(t):
                label = alias.get(host)
                if label and label != own:
                    need.add((label, int(port)))
        if need:
            name = f"{d.get('kind')}/{(d.get('metadata') or {}).get('name', '?')}"
            out[name] = (labels, need)
    return out


def peer_admits(peer, labels):
    """True bila peer `from` (podSelector tanpa namespaceSelector) mencakup pod berlabel `labels`."""
    if peer.get("namespaceSelector") or peer.get("ipBlock"):
        return False
    sel = (peer.get("podSelector") or {}).get("matchLabels")
    if sel is None:
        return False
    # podSelector kosong ({}) memilih SEMUA pod di namespace.
    return all(labels.get(k) == v for k, v in sel.items())


def unadmitted_consumers(ds, alias, rules):
    """[(workload, target, port)] — konsumen yang tak dicakup policy Ingress mana pun."""
    bad = []
    for workload, (labels, need) in sorted(consumer_pairs(ds, alias).items()):
        for target, port in sorted(need):
            admitted = any(
                d == "Ingress" and t == target and p == port
                and any(peer_admits(peer, labels) for peer in (r.get("from") or []))
                for _pol, d, t, p, r in rules
            )
            if not admitted:
                bad.append((workload, target, port))
    return bad


def self_test():
    """Kanari dua arah. Penjaga yang hanya diuji pada kasus 'lulus' tak membuktikan apa pun."""
    def svc(name, ports):
        return {"kind": "Service", "metadata": {"name": name},
                "spec": {"selector": {NAME: name}, "ports": [{"port": p} for p in ports]}}
    def wl(name, comp, env=None, env_from=None):
        c = {"name": name, "env": env or [], "envFrom": env_from or []}
        return {"kind": "Deployment", "metadata": {"name": name},
                "spec": {"template": {"metadata": {"labels": {NAME: name, COMPONENT: comp}},
                                      "spec": {"containers": [c]}}}}
    cm = {"kind": "ConfigMap", "metadata": {"name": "cfg"}, "data": {"URL": "http://integrasi:50051"}}
    pol = lambda comps: {"kind": "NetworkPolicy", "metadata": {"name": "allow"},
                         "spec": {"podSelector": {"matchLabels": {NAME: "integrasi"}}, "policyTypes": ["Ingress"],
                                  "ingress": [{"from": [{"podSelector": {"matchLabels": {COMPONENT: c}}} for c in comps],
                                               "ports": [{"port": 50051}]}]}}
    via_key = [{"name": "U", "valueFrom": {"configMapKeyRef": {"name": "cfg", "key": "URL"}}}]
    via_from = [{"configMapRef": {"name": "cfg"}}]
    cases = [
        # (nama, dokumen, jumlah konsumen yang tak diizinkan yang HARUS ditemukan)
        ("identity lewat configMapKeyRef, policy hanya backend/frontend", [svc("integrasi", [50051]), cm, wl("authenc", "identity", env=via_key), pol(["backend", "frontend"])], 1),
        ("envFrom saja BUKAN bukti pemakaian (menyalin seluruh ConfigMap)", [svc("integrasi", [50051]), cm, wl("authenc", "identity", env_from=via_from), pol(["backend", "frontend"])], 0),
        ("URL literal pada pod identity", [svc("integrasi", [50051]), wl("authenc", "identity", env=[{"name": "U", "value": "http://integrasi:50051"}]), pol(["backend"])], 1),
        ("identity DIIZINKAN", [svc("integrasi", [50051]), cm, wl("authenc", "identity", env=via_key), pol(["backend", "frontend", "identity"])], 0),
        ("backend sudah diizinkan", [svc("integrasi", [50051]), cm, wl("perlengkapan", "backend", env=via_key), pol(["backend", "frontend"])], 0),
        ("workload tak memakai URL itu", [svc("integrasi", [50051]), cm, wl("portal", "frontend"), pol(["backend"])], 0),
        ("integrasi menunjuk dirinya sendiri", [svc("integrasi", [50051]), cm, wl("integrasi", "backend", env=via_key), pol(["frontend"])], 0),
    ]
    gagal = []
    for nama, ds, expect in cases:
        _ports, alias = collect_services(ds)
        got = len(unadmitted_consumers(ds, alias, netpol_rules(ds)))
        if got != expect:
            gagal.append(f"{nama}: ditemukan {got}, seharusnya {expect}")
    if gagal:
        print("self-test check-netpol-service-ports GAGAL:", file=sys.stderr)
        for g in gagal:
            print(f"  - {g}", file=sys.stderr)
        sys.exit(2)


def main():
    self_test()
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

    # ── C. tiap KONSUMEN URL in-cluster harus dicakup peer `from` ───────────
    if deny_ingress:
        for workload, target, port in unadmitted_consumers(ds, alias, rules):
            problems.append(
                f"{workload} memakai `{target}:{port}` (dari URL terkonfigurasi), tetapi "
                f"tak ada NetworkPolicy Ingress untuk `{target}:{port}` yang peer `from`-nya "
                f"mencakup label pod {workload}. `default-deny-ingress` akan menelan "
                f"koneksinya, dan penelepon menunggu sampai time out."
            )

    if problems:
        print(f"[{label}] NetworkPolicy tidak cocok dengan Service/URL chart:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        sys.exit(1)
    print(f"[{label}] NetworkPolicy cocok dengan port Service dan URL terkonfigurasi.")


if __name__ == "__main__":
    main()
