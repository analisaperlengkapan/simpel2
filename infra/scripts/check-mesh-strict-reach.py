#!/usr/bin/env python3
"""Di namespace STRICT, klien di luar mesh tak bisa memanggil server di dalam mesh.

PeerAuthentication STRICT membuat Envoy sebuah server menolak koneksi yang bukan mTLS Istio.
Klien TANPA sidecar (`sidecar.istio.io/inject: "false"`) tidak bisa melakukannya: handshake-nya
dijawab dengan sertifikat Envoy (CA Istio), dan klien gRPC melihat
`invalid peer certificate: UnknownIssuer` — galat yang menyalahkan PKI kita padahal PKI-nya
benar (setiap sertifikat leaf terverifikasi terhadap CA, SAN-nya cocok).

Kejadian nyata (rehearsal produksi, 2026-11-09): authenc berjalan tanpa sidecar dan memanggil
layanan-integrasi:50051 (provisioning otomatis di login pertama, `/auth/me`); integrasi ada di
dalam mesh. Staging PERMISSIVE, jadi tak pernah melihatnya; produksi STRICT. Akibatnya login
pertama SETIAP pegawai gagal. Semua hal lain hijau: `helm lint`, kube-linter, guard TLS, e2e.

Aturan: bila PeerAuthentication tingkat-namespace STRICT, setiap pasangan klien-di-luar-mesh ->
Service-di-dalam-mesh harus punya PeerAuthentication ber-`selector` pada workload server itu
dengan `portLevelMtls[<targetPort>]` PERMISSIVE/DISABLE. Port itu tetap dijaga mTLS aplikasi.

Cakupan DITURUNKAN dari render: keanggotaan mesh dari anotasi inject tiap pod-template, server
dari objek Service, klien dari URL in-cluster yang benar-benar dipakai pod (`configMapKeyRef`
atau env literal). Hop baru ikut tercakup pada hari ia ditambahkan.

ATURAN B (semua lingkungan): pod Job/CronJob harus menyatakan `sidecar.istio.io/inject`
secara EKSPLISIT. Tanpa anotasi mereka ikut injeksi namespace: percobaan pertama bertabrakan
dengan Envoy yang belum mendengar (keluar kode 1 dalam 3 ms: "error connecting to server"),
restart OnFailure berikutnya berhasil tetapi `backoffLimit: 1` sudah habis sehingga Job
tercatat GAGAL walau pekerjaannya selesai. Itu yang terjadi pada empat CronJob integrasi di
staging (`lastSuccessfulTime` kosong sejak awal) dan terukur di rehearsal produksi.

Pakai: yq -o=json eval-all '[.]' < rendered.yaml | check-mesh-strict-reach.py <label>
"""
import json
import re
import sys

NAME = "app.kubernetes.io/name"
URL_RE = re.compile(r"(?:^|//)([a-z0-9][a-z0-9-]*)(?:\.[a-z0-9-]+)*?(?:\.svc\.cluster\.local)?:(\d{2,5})\b")
KINDS = ("Deployment", "StatefulSet", "Job", "CronJob")


def docs(blob):
    for d in blob:
        if isinstance(d, dict) and d.get("kind"):
            yield d


def template_of(d):
    if d["kind"] == "CronJob":
        return d["spec"]["jobTemplate"]["spec"]["template"]
    return d["spec"]["template"]


def in_mesh(tpl):
    """Tanpa anotasi = ikut default namespace (injeksi aktif) = di dalam mesh."""
    ann = (tpl.get("metadata") or {}).get("annotations") or {}
    return str(ann.get("sidecar.istio.io/inject", "true")).lower() != "false"


def cm_values(ds):
    out = {}
    for d in ds:
        if d["kind"] == "ConfigMap":
            for k, v in (d.get("data") or {}).items():
                out[(d["metadata"]["name"], k)] = v
    return out


def consumers_of(tpl, cms):
    """{(service, port)} yang dipakai pod ini lewat URL in-cluster."""
    hits = set()
    spec = tpl.get("spec", {})
    for c in (spec.get("containers") or []) + (spec.get("initContainers") or []):
        for e in c.get("env") or []:
            val = e.get("value")
            ref = (e.get("valueFrom") or {}).get("configMapKeyRef")
            if val is None and ref:
                val = cms.get((ref.get("name"), ref.get("key")))
            for host, port in URL_RE.findall(val or ""):
                hits.add((host, int(port)))
    return hits


def check(objs):
    ds = list(docs(objs))
    ns_mode = None
    for d in ds:
        if d["kind"] == "PeerAuthentication" and not (d["spec"].get("selector") or {}).get("matchLabels"):
            ns_mode = (d["spec"].get("mtls") or {}).get("mode")
    if ns_mode != "STRICT":
        return [], 0, f"PeerAuthentication namespace = {ns_mode or 'tak ada'}, bukan STRICT — dilewati"

    cms = cm_values(ds)
    svc = {}  # nama Service -> (label name, {port: targetPort})
    for d in ds:
        if d["kind"] == "Service":
            sel = (d["spec"].get("selector") or {})
            ports = {}
            for p in d["spec"].get("ports") or []:
                tp = p.get("targetPort", p["port"])
                ports[int(p["port"])] = int(tp) if isinstance(tp, int) or str(tp).isdigit() else None
            svc[d["metadata"]["name"]] = (sel.get(NAME), ports)

    mesh = {}  # label name -> di dalam mesh?
    pods = []
    for d in ds:
        if d["kind"] in KINDS:
            tpl = template_of(d)
            labels = (tpl.get("metadata") or {}).get("labels") or {}
            if labels.get(NAME):
                mesh.setdefault(labels[NAME], in_mesh(tpl))
            pods.append((d, tpl, labels))

    exceptions = set()  # (label name, port)
    for d in ds:
        if d["kind"] != "PeerAuthentication":
            continue
        want = (d["spec"].get("selector") or {}).get("matchLabels") or {}
        for port, cfg in (d["spec"].get("portLevelMtls") or {}).items():
            if (cfg or {}).get("mode") in ("PERMISSIVE", "DISABLE") and want.get(NAME):
                exceptions.add((want[NAME], int(port)))

    bad, pairs = [], 0
    for d, tpl, labels in pods:
        if in_mesh(tpl):
            continue  # klien di dalam mesh: Envoy-nya membawa identitas mesh
        client = d["metadata"]["name"]
        for host, port in sorted(consumers_of(tpl, cms)):
            if host not in svc:
                continue
            target, ports = svc[host]
            if not target or not mesh.get(target, False):
                continue  # server di luar mesh: tak ada Envoy yang menuntut mTLS Istio
            tport = ports.get(port)
            pairs += 1
            if (target, tport) not in exceptions:
                bad.append(
                    f"{client} (di luar mesh) memanggil {host}:{port} (container port {tport}), "
                    f"server `{target}` ada di dalam mesh dan namespace STRICT, tetapi tak ada "
                    f"PeerAuthentication ber-selector {target} dengan portLevelMtls[{tport}] PERMISSIVE"
                )
    return bad, pairs, ""


def check_jobs(objs):
    """Aturan B: Job/CronJob wajib menyatakan anotasi inject secara eksplisit."""
    bad = []
    for d in docs(objs):
        if d["kind"] not in ("Job", "CronJob"):
            continue
        ann = (template_of(d).get("metadata") or {}).get("annotations") or {}
        if "sidecar.istio.io/inject" not in ann:
            bad.append(
                f"{d['kind']}/{d['metadata']['name']} tak menyatakan sidecar.istio.io/inject — "
                "ikut injeksi namespace, percobaan pertamanya bertabrakan dengan Envoy yang belum siap"
            )
    return bad


def _dep(name, inject, env=None):
    ann = {} if inject is None else {"sidecar.istio.io/inject": inject}
    return {"kind": "Deployment", "metadata": {"name": name},
            "spec": {"template": {"metadata": {"labels": {NAME: name}, "annotations": ann},
                                  "spec": {"containers": [{"name": "c", "env": env or []}]}}}}


def _svc(name, port=50051):
    return {"kind": "Service", "metadata": {"name": name},
            "spec": {"selector": {NAME: name}, "ports": [{"port": port, "targetPort": port}]}}


def _pa(mode, selector=None, ports=None):
    spec = {"mtls": {"mode": mode}}
    if selector:
        spec["selector"] = {"matchLabels": {NAME: selector}}
    if ports:
        spec["portLevelMtls"] = {str(p): {"mode": m} for p, m in ports.items()}
    return {"kind": "PeerAuthentication", "metadata": {"name": f"pa-{selector or 'ns'}"}, "spec": spec}


def self_test():
    url = [{"name": "X_URL", "value": "https://srv.ns.svc.cluster.local:50051"}]
    base = [_svc("srv"), _dep("srv", "true")]
    cases = [
        ("STRICT + klien luar mesh -> server dalam mesh tanpa pengecualian (kasus nyata)",
         base + [_pa("STRICT"), _dep("cli", "false", url)], 1),
        ("pengecualian port PERMISSIVE pada server -> lulus",
         base + [_pa("STRICT"), _pa("STRICT", "srv", {50051: "PERMISSIVE"}), _dep("cli", "false", url)], 0),
        ("pengecualian di port yang salah -> tetap gagal",
         base + [_pa("STRICT"), _pa("STRICT", "srv", {9999: "PERMISSIVE"}), _dep("cli", "false", url)], 1),
        ("namespace PERMISSIVE (staging) -> dilewati",
         base + [_pa("PERMISSIVE"), _dep("cli", "false", url)], 0),
        ("klien di dalam mesh -> lulus",
         base + [_pa("STRICT"), _dep("cli", "true", url)], 0),
        ("klien tanpa anotasi = di dalam mesh -> lulus",
         base + [_pa("STRICT"), _dep("cli", None, url)], 0),
        ("server di luar mesh -> lulus",
         [_svc("srv"), _dep("srv", "false"), _pa("STRICT"), _dep("cli", "false", url)], 0),
    ]
    fail = 0
    for title, objs, want in cases:
        bad, _, _ = check(objs)
        if len(bad) != want:
            print(f"  GAGAL: {title}: dapat {len(bad)}, seharusnya {want}", file=sys.stderr)
            fail += 1

    def job(kind, ann):
        tpl = {"metadata": {"labels": {NAME: "j"}, "annotations": ann}, "spec": {"containers": [{"name": "c"}]}}
        spec = {"jobTemplate": {"spec": {"template": tpl}}} if kind == "CronJob" else {"template": tpl}
        return {"kind": kind, "metadata": {"name": "j"}, "spec": spec}

    for title, objs, want in [
        ("aturan B: CronJob tanpa anotasi inject (kasus nyata)", [job("CronJob", {})], 1),
        ("aturan B: Job tanpa anotasi inject", [job("Job", {})], 1),
        ("aturan B: inject=false eksplisit lulus", [job("CronJob", {"sidecar.istio.io/inject": "false"})], 0),
        ("aturan B: inject=true eksplisit lulus", [job("Job", {"sidecar.istio.io/inject": "true"})], 0),
    ]:
        got = len(check_jobs(objs))
        if got != want:
            print(f"  GAGAL: {title}: dapat {got}, seharusnya {want}", file=sys.stderr)
            fail += 1
    if fail:
        print("self-test check-mesh-strict-reach GAGAL", file=sys.stderr)
        sys.exit(2)


def main():
    self_test()
    label = sys.argv[1] if len(sys.argv) > 1 else "chart"
    blob = json.load(sys.stdin)
    if blob and isinstance(blob[0], list):
        blob = blob[0]
    rc = 0
    jobs_bad = check_jobs(blob)
    if jobs_bad:
        print(f"[{label}] {len(jobs_bad)} Job/CronJob tanpa anotasi inject eksplisit:", file=sys.stderr)
        for b in jobs_bad:
            print(f"  - {b}", file=sys.stderr)
        print("  Set `istioInjection: false` (atau true bila memang perlu) pada values workload itu.", file=sys.stderr)
        rc = 1
    bad, pairs, note = check(blob)
    if note:
        print(f"[{label}] {note}")
    elif bad:
        print(f"[{label}] {len(bad)} hop di-blok mTLS STRICT mesh:", file=sys.stderr)
        for b in bad:
            print(f"  - {b}", file=sys.stderr)
        print("  Tambahkan `mtls.portExceptions` ({ workload, port }) di values-production.yaml.", file=sys.stderr)
        rc = 1
    else:
        print(f"[{label}] OK — {pairs} hop klien-luar-mesh -> server-dalam-mesh, semuanya berpengecualian port.")
    if not jobs_bad:
        print(f"[{label}] OK — setiap Job/CronJob menyatakan sidecar.istio.io/inject.")
    return rc


if __name__ == "__main__":
    sys.exit(main())
