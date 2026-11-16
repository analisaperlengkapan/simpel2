#!/usr/bin/env python3
"""Setiap container yang dirender harus muat di LimitRange namespace-nya.

LimitRange (`limitRange.enabled`, aktif di production) menolak POD, bukan Deployment: bila satu
container meminta/membatasi di bawah `min` atau di atas `max`, ReplicaSet gagal dengan
`pods "..." is forbidden: minimum memory usage per Container is 32Mi, but request is 16Mi`,
Deployment-nya tetap ADA dan melaporkan 0/N tanpa satu pun pod. `helm lint`, `helm template`,
kube-linter, dan semua tes staging tetap hijau, karena staging tidak punya LimitRange.

Kejadian nyata (rehearsal produksi, 2026-11-10): SIMPEL v1 dinyalakan di production. Init
container `nginx-dirs` meminta 16Mi; LimitRange production mensyaratkan 32Mi. Deployment `simpelv1`
berumur dua menit dengan 0/3 pod dan tanpa galat apa pun selain event FailedCreate pada
ReplicaSet. v1 tak pernah berjalan dalam profil production sebelum itu, jadi tak ada yang tahu.

Aturan: untuk tiap LimitRange bertipe Container, setiap container (init maupun biasa) pada
Deployment/StatefulSet/Job/CronJob/DaemonSet yang `resources.requests`/`resources.limits`-nya
menyebut cpu/memory harus berada dalam [min, max]. Container tanpa angka memakai default
LimitRange, yang sudah dijaga `default`/`defaultRequest`.

Pakai: yq -o=json eval-all '[.]' < rendered.yaml | check-limitrange-bounds.py <label>
"""
import json
import re
import sys

KINDS = ("Deployment", "StatefulSet", "Job", "CronJob", "DaemonSet")
_Q = re.compile(r"^([0-9]*\.?[0-9]+)([A-Za-z]*)$")
_MULT = {
    "": 1, "m": 1e-3, "k": 1e3, "K": 1e3, "M": 1e6, "G": 1e9, "T": 1e12, "P": 1e15,
    "Ki": 2**10, "Mi": 2**20, "Gi": 2**30, "Ti": 2**40, "Pi": 2**50,
}


def qty(v):
    """Kuantitas Kubernetes -> float (core untuk cpu, byte untuk memory)."""
    m = _Q.match(str(v).strip())
    if not m or m.group(2) not in _MULT:
        raise ValueError(f"kuantitas tak dikenal: {v!r}")
    return float(m.group(1)) * _MULT[m.group(2)]


def docs(blob):
    for d in blob:
        if isinstance(d, dict) and d.get("kind"):
            yield d


def pod_spec(d):
    spec = d.get("spec") or {}
    if d["kind"] == "CronJob":
        spec = ((spec.get("jobTemplate") or {}).get("spec") or {})
    return ((spec.get("template") or {}).get("spec")) or {}


def container_bounds(objs):
    """{namespace: [(min, max) per resource]} dari LimitRange bertipe Container."""
    out = {}
    for d in objs:
        if d["kind"] != "LimitRange":
            continue
        ns = (d.get("metadata") or {}).get("namespace", "")
        for lim in (d.get("spec") or {}).get("limits", []):
            if lim.get("type") == "Container":
                out.setdefault(ns, []).append(lim)
    return out


def check(objs):
    objs = list(docs(objs))
    bounds = container_bounds(objs)
    bad, checked = [], 0
    for d in objs:
        if d["kind"] not in KINDS:
            continue
        ns = (d.get("metadata") or {}).get("namespace", "")
        limits = bounds.get(ns)
        if not limits:
            continue
        ps = pod_spec(d)
        for group in ("initContainers", "containers"):
            for c in ps.get(group) or []:
                res = c.get("resources") or {}
                for lim in limits:
                    for kind_ in ("requests", "limits"):
                        for rname, val in (res.get(kind_) or {}).items():
                            if rname not in ("cpu", "memory"):
                                continue
                            checked += 1
                            x = qty(val)
                            lo = qty(lim["min"][rname]) if rname in (lim.get("min") or {}) else None
                            hi = qty(lim["max"][rname]) if rname in (lim.get("max") or {}) else None
                            who = f"{d['kind']}/{d['metadata']['name']} {group[:-1 if group == 'containers' else -10]} `{c.get('name')}`"
                            if lo is not None and x < lo:
                                bad.append(f"{who}: {kind_}.{rname} {val} < LimitRange min {lim['min'][rname]}")
                            if hi is not None and x > hi:
                                bad.append(f"{who}: {kind_}.{rname} {val} > LimitRange max {lim['max'][rname]}")
    return bad, checked, sum(len(v) for v in bounds.values())


def self_test():
    def lr(mn, mx=None, ns="n"):
        lim = {"type": "Container", "min": mn}
        if mx:
            lim["max"] = mx
        return {"kind": "LimitRange", "metadata": {"name": "l", "namespace": ns}, "spec": {"limits": [lim]}}

    def dep(requests=None, limits=None, init=False, ns="n"):
        c = {"name": "c", "resources": {"requests": requests or {}, "limits": limits or {}}}
        ps = {"initContainers": [c]} if init else {"containers": [c]}
        return {"kind": "Deployment", "metadata": {"name": "d", "namespace": ns},
                "spec": {"template": {"spec": ps}}}

    mn = {"cpu": "10m", "memory": "32Mi"}
    mx = {"cpu": "4", "memory": "8Gi"}
    cases = [
        ("init container 16Mi < 32Mi (kasus nyata)", [lr(mn), dep({"memory": "16Mi"}, init=True)], 1),
        ("container biasa 16Mi < 32Mi", [lr(mn), dep({"memory": "16Mi"})], 1),
        ("limit di bawah min", [lr(mn), dep(limits={"cpu": "5m"})], 1),
        ("tepat di min lulus", [lr(mn), dep({"memory": "32Mi", "cpu": "10m"})], 0),
        ("di atas max", [lr(mn, mx), dep(limits={"memory": "9Gi"})], 1),
        ("tepat di max lulus", [lr(mn, mx), dep(limits={"memory": "8Gi", "cpu": "4"})], 0),
        ("satuan campur: 1Gi >= 32Mi lulus", [lr(mn), dep({"memory": "1Gi", "cpu": "500m"})], 0),
        ("tanpa LimitRange (staging) lulus", [dep({"memory": "1Mi"})], 0),
        ("namespace lain tak terkena", [lr(mn, ns="x"), dep({"memory": "1Mi"}, ns="n")], 0),
        ("tanpa angka -> memakai default, lulus", [lr(mn), dep()], 0),
    ]
    fail = 0
    for title, objs, want in cases:
        got = len(check(objs)[0])
        if got != want:
            print(f"  GAGAL: {title}: dapat {got}, seharusnya {want}", file=sys.stderr)
            fail += 1
    if fail:
        print("self-test check-limitrange-bounds GAGAL", file=sys.stderr)
        sys.exit(2)


def main():
    self_test()
    label = sys.argv[1] if len(sys.argv) > 1 else "chart"
    blob = json.load(sys.stdin)
    if blob and isinstance(blob[0], list):
        blob = blob[0]
    bad, checked, nlim = check(blob)
    if not nlim:
        print(f"[{label}] OK — tak ada LimitRange Container di render ini, tak ada yang dicek.")
        return 0
    if bad:
        print(f"[{label}] {len(bad)} container melanggar LimitRange (pod DITOLAK, Deployment tetap 0/N):",
              file=sys.stderr)
        for b in bad:
            print(f"  - {b}", file=sys.stderr)
        print("  Sesuaikan resources workload itu, atau ubah limitRange di values.", file=sys.stderr)
        return 1
    print(f"[{label}] OK — {checked} angka resources, semuanya dalam batas LimitRange.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
