#!/usr/bin/env python3
"""Setiap pod yang berbicara ke postgres harus dicakup NetworkPolicy yang membuka postgres.

`default-deny-ingress` menutup SETIAP pod; postgres hanya terbuka untuk pod yang labelnya
dicakup salah satu peer `from` pada `allow-backend-to-postgres`. Pod klien yang labelnya
terlewat tidak gagal dengan galat — `pg_isready`/`connect()` hanya MENUNGGU, lalu Job habis
waktu atau berhenti dalam hitungan detik. Dua kejadian nyata dari satu sebab (2026-11-09):

  - Job hook `db-create` (component=database): `helm install` produksi gagal di hook
    post-install setelah 600 detik (`DeadlineExceeded`). Di staging hook itu MATI
    (`dbCreateJob.enabled: false`), jadi jalur ini tak pernah dijalankan siapa pun.
  - CronJob `layanan-integrasi-{siman,mysimkari,monsakti,snapshot-refresh}`
    (component=scheduler): gagal dalam ~10 detik di staging, `lastSuccessfulTime` kosong
    — penarikan data dan penyegaran snapshot tak pernah berhasil, tanpa satu pun sinyal merah.

`helm lint`, `helm template`, kube-linter, dan guard port (`check-netpol-service-ports.py`)
lolos semuanya: manifesnya sah. Guard port bertanya "apakah port itu terbuka untuk pod MANA
PUN"; guard ini bertanya per KLIEN: pod yang benar-benar memakai postgres, apakah labelnya
dicakup salah satu peer `from`.

Cakupan DITURUNKAN dari manifes yang dirender, bukan daftar tulis-tangan. "Klien postgres" =
pod-template (Deployment, StatefulSet, Job, CronJob) yang salah satu kontainernya (init
termasuk) memakai `DATABASE_URL`/`PG*`/`POSTGRES_*`, merujuk Secret `postgres-credentials`,
atau menjalankan `pg_isready`/`psql`. Klien berikutnya ikut tercakup pada hari ia ditambahkan.

Pakai: yq -o=json eval-all '[.]' < rendered.yaml | check-netpol-postgres-reach.py <label>
"""
import json
import re
import sys

NAME = "app.kubernetes.io/name"
PG_ENV = re.compile(r"^(DATABASE_URL|PG[A-Z]+|POSTGRES_[A-Z_]+)$")
PG_CMD = re.compile(r"\b(pg_isready|psql)\b|postgres:5432")
PG_SECRET = "postgres-credentials"
KINDS = ("Deployment", "StatefulSet", "Job", "CronJob")


def docs(blob):
    for d in blob:
        if isinstance(d, dict) and d.get("kind"):
            yield d


def template_of(d):
    if d["kind"] == "CronJob":
        return d["spec"]["jobTemplate"]["spec"]["template"]
    return d["spec"]["template"]


def uses_postgres(tpl):
    spec = tpl.get("spec", {})
    for c in (spec.get("containers") or []) + (spec.get("initContainers") or []):
        for e in c.get("env") or []:
            if PG_ENV.match(e.get("name", "")):
                return True
            ref = (e.get("valueFrom") or {}).get("secretKeyRef") or {}
            if ref.get("name") == PG_SECRET:
                return True
        for ef in c.get("envFrom") or []:
            if (ef.get("secretRef") or {}).get("name") == PG_SECRET:
                return True
        if PG_CMD.search(" ".join((c.get("command") or []) + (c.get("args") or []))):
            return True
    return False


def selects(selector, labels):
    """matchLabels saja. matchExpressions = tak didukung -> dianggap TIDAK cocok (gagal keras)."""
    if selector is None:
        return True
    if selector.get("matchExpressions"):
        return False
    return all(labels.get(k) == v for k, v in (selector.get("matchLabels") or {}).items())


def postgres_policies(ds):
    """NetworkPolicy yang menyeleksi pod postgres (name=postgres) dan membuka 5432."""
    out = []
    for d in ds:
        if d["kind"] != "NetworkPolicy":
            continue
        sel = d["spec"].get("podSelector") or {}
        if (sel.get("matchLabels") or {}).get(NAME) != "postgres":
            continue
        if "Ingress" not in (d["spec"].get("policyTypes") or ["Ingress"]):
            continue
        out.append(d)
    return out


def check(objs):
    ds = list(docs(objs))
    policies = postgres_policies(ds)
    if not policies:
        return [], 0, "tak ada NetworkPolicy yang menyeleksi postgres — dilewati"
    peers = []
    for p in policies:
        for rule in p["spec"].get("ingress") or []:
            ports = [x.get("port") for x in rule.get("ports") or []]
            if ports and 5432 not in ports and "5432" not in map(str, ports):
                continue
            for f in rule.get("from") or []:
                if "podSelector" in f and "namespaceSelector" not in f:
                    peers.append(f["podSelector"])
    bad, clients = [], 0
    for d in ds:
        if d["kind"] not in KINDS:
            continue
        tpl = template_of(d)
        labels = (tpl.get("metadata") or {}).get("labels") or {}
        if labels.get(NAME) == "postgres":
            continue  # target kebijakan itu sendiri
        if not uses_postgres(tpl):
            continue
        clients += 1
        if not any(selects(s, labels) for s in peers):
            bad.append(
                f"{d['kind']}/{d['metadata']['name']} memakai postgres tetapi labelnya "
                f"(name={labels.get(NAME)}, component={labels.get('app.kubernetes.io/component')}) "
                "tak dicakup peer `from` mana pun pada policy postgres"
            )
    return bad, clients, ""


def _obj(kind, name, labels, env=None, cmd=None, secret_env=None, cron=False):
    c = {"name": "c", "env": env or [], "command": cmd or []}
    if secret_env:
        c["env"] = c["env"] + [{"name": "X", "valueFrom": {"secretKeyRef": {"name": secret_env, "key": "k"}}}]
    tpl = {"metadata": {"labels": labels}, "spec": {"containers": [c]}}
    spec = {"jobTemplate": {"spec": {"template": tpl}}} if cron else {"template": tpl}
    return {"kind": kind, "metadata": {"name": name}, "spec": spec}


def _policy(*peers):
    return {"kind": "NetworkPolicy", "metadata": {"name": "allow-backend-to-postgres"},
            "spec": {"podSelector": {"matchLabels": {NAME: "postgres"}}, "policyTypes": ["Ingress"],
                     "ingress": [{"from": [{"podSelector": {"matchLabels": p}} for p in peers],
                                  "ports": [{"port": 5432}]}]}}


def self_test():
    comp = "app.kubernetes.io/component"
    ok_pol = _policy({comp: "backend"}, {NAME: "authenc"})
    pg = {NAME: "postgres", comp: "database"}
    cases = [
        ("klien tercakup lewat component", [ok_pol, _obj("Deployment", "a", {NAME: "a", comp: "backend"}, env=[{"name": "DATABASE_URL"}])], 0),
        ("hook db-create tak tercakup (kasus nyata)", [ok_pol, _obj("Job", "db-create", {NAME: "db-create", comp: "database"}, env=[{"name": "PGHOST"}])], 1),
        ("CronJob scheduler tak tercakup (kasus nyata)", [ok_pol, _obj("CronJob", "cron", {NAME: "cron", comp: "scheduler"}, env=[{"name": "DATABASE_URL"}], cron=True)], 1),
        ("klien lewat Secret postgres-credentials", [ok_pol, _obj("Job", "j", {NAME: "j", comp: "x"}, secret_env="postgres-credentials")], 1),
        ("klien lewat perintah pg_isready", [ok_pol, _obj("Job", "j", {NAME: "j", comp: "x"}, cmd=["sh", "-c", "until pg_isready; do :; done"])], 1),
        ("pod tanpa sinyal postgres diabaikan", [ok_pol, _obj("Deployment", "web", {NAME: "web", comp: "frontend"}, env=[{"name": "PORT"}])], 0),
        ("target (postgres) tak dihitung klien", [ok_pol, _obj("StatefulSet", "postgres", pg, env=[{"name": "POSTGRES_DB"}])], 0),
        ("peer ditambah -> lulus", [_policy({comp: "backend"}, {NAME: "db-create"}), _obj("Job", "db-create", {NAME: "db-create", comp: "database"}, env=[{"name": "PGHOST"}])], 0),
        ("tanpa policy postgres -> dilewati", [_obj("Job", "j", {NAME: "j"}, env=[{"name": "PGHOST"}])], 0),
    ]
    fail = 0
    for title, objs, want in cases:
        bad, _, _ = check(objs)
        if len(bad) != want:
            print(f"  GAGAL: {title}: dapat {len(bad)} temuan, seharusnya {want}", file=sys.stderr)
            fail += 1
    # kanari: guard HARUS menolak render yang dirusak, kalau tidak ia tak menjaga apa-apa
    bad, clients, _ = check([ok_pol, _obj("Job", "x", {NAME: "x"}, env=[{"name": "PGHOST"}])])
    if not bad or clients != 1:
        print("  GAGAL: kanari lolos", file=sys.stderr)
        fail += 1
    if fail:
        print("self-test check-netpol-postgres-reach GAGAL", file=sys.stderr)
        sys.exit(2)


def main():
    self_test()
    label = sys.argv[1] if len(sys.argv) > 1 else "chart"
    blob = json.load(sys.stdin)
    if blob and isinstance(blob[0], list):
        blob = blob[0]
    bad, clients, note = check(blob)
    if note:
        print(f"[{label}] {note}")
        return 0
    if bad:
        print(f"[{label}] {len(bad)} klien postgres TIDAK diizinkan NetworkPolicy:", file=sys.stderr)
        for b in bad:
            print(f"  - {b}", file=sys.stderr)
        print("  Tambahkan peer `from` yang mencakup labelnya di allow-backend-to-postgres "
              "(templates/network/policies.yaml).", file=sys.stderr)
        return 1
    print(f"[{label}] OK — {clients} klien postgres, semuanya dicakup NetworkPolicy.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
