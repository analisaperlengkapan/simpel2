
# 🔐 Vault for SIMPelv2

Repositori ini berisi konfigurasi dan skrip untuk mengatur HashiCorp Vault di lingkungan Kubernetes SIMPelv2. Semua fitur disesuaikan agar dapat digunakan secara **gratis**, **open source**, dan **sesuai standar ISO**.

---

## 🚀 Fitur Unggulan yang Didukung

| Fitur | Status | Keterangan |
|-------|--------|------------|
| Secret Storage (KV v2) | ✅ | Menyimpan rahasia terenkripsi |
| Dynamic Secrets | ✅ | Otomatisasi secrets untuk PostgreSQL dan lainnya |
| TTL & Rotation | ✅ | Waktu kedaluwarsa secrets otomatis |
| Audit Logging | ✅ | Semua operasi dicatat |
| RBAC & Policy | ✅ | Kontrol akses berdasarkan role dan path |
| Auto-Unseal | ✅ | Unseal otomatis via K8s secret |
| Vault Agent | ✅ | Inject secret ke Pod secara otomatis |
| Templating | ✅ | Gunakan `vault-agent` + template |
| Versioning | ✅ | Support via KV v2 |
| Kubernetes Auth | ✅ | Integrasi native dengan ServiceAccount |
| API & SDK | ✅ | Konsumsi via HTTP API dan CLI |
| Observability | ✅ | Monitoring via Prometheus dan log |
| Transit Encryption | ✅ | Enkripsi data via Vault API |
| Policy as Code | ✅ | HCL policy di-repo version control |
| Compliance Logging | ✅ | Audit file siap untuk forensik |
| Plugin Support | ✅ | Dukung plugin otentikasi tambahan |
| Workspace Grouping (Custom) | 🛠️ | Simulasi via struktur path `/<divisi>/<app>/...` |
| Role Delegation (Custom) | 🛠️ | Admin dapat memberi role via policy generator |
| Replication (Custom) | 🛠️ | Snapshot sync antar node via script |
| HSM Integration (Custom) | 🛠️ | Simulasi auto-unseal dengan K8s secret |

---

## 🧱 Struktur Folder

vault/
├── config/
│   ├── vault.hcl              # Konfigurasi utama Vault (listener, storage, UI)
│   └── policies/              # Folder semua policy HCL
├── scripts/
│   ├── setup_vault.py         # Inisialisasi dan konfigurasi Vault
│   ├── auto_unseal.py         # Script auto-unseal
│   ├── sync_snapshot.py       # Sinkronisasi antar node (replication custom)
│   └── delegator.py           # Role delegation via REST/API
├── k8s/
│   ├── deployment.yaml        # Deployment Vault di MicroK8s
│   ├── service.yaml           # Service internal Vault
│   ├── secret-init.yaml       # Secret K8s untuk auto-unseal
│   └── cronjob-backup.yaml    # Cron untuk snapshot harian
└── README.md


---

## ⚙️ Instalasi

```bash
make setup-vault

Atau manual:

kubectl apply -f vault/k8s/
python3 vault/scripts/setup_vault.py

🔐 Auto-Unseal (Kustom)

Auto-unseal dilakukan via secret K8s:

apiVersion: v1
kind: Secret
metadata:
  name: vault-init
type: Opaque
stringData:
  init.json: |
    {
      "unseal_keys_b64": ["..."],
      "root_token": "s.xxxx"
    }

Script auto_unseal.py akan otomatis menjalankan vault operator unseal di startup.

🔁 Custom Replication (Snapshot Sync)

python3 vault/scripts/sync_snapshot.py

Mengambil snapshot dari cluster aktif

Mengirim ke remote node

Merestore otomatis di target Vault

🧑‍🤝‍🧑 Role Delegation (Policy as Code)

Admin menjalankan:

python3 vault/scripts/delegator.py --target team-a --capabilities read,write

Policy tersimpan otomatis di vault/config/policies/team-a.hcl

Di-apply via Vault CLI atau API

🗂️ Workspace Grouping (Virtual Namespace)

Setiap rahasia disimpan berdasarkan struktur:

/<divisi>/<aplikasi>/<env>/...

Contoh:

/keuangan/sipeta/prod/db_password

/it/simpelv2/dev/api_key

Gunakan path-based policy untuk pembatasan akses:

path "keuangan/*" {
  capabilities = ["read", "list"]
}

📜 Contoh Policy

# path: policies/simpelv2-dev.hcl

path "it/simpelv2/dev/*" {
  capabilities = ["read", "list"]
}

📊 Monitoring & Audit

Semua operasi disimpan ke audit.log

Dapat di-forward ke Loki, Elasticsearch, atau Splunk

Integrasi Prometheus/Grafana via exporter

💬 API & CLI

vault kv get secret/it/simpelv2/dev/api_key
vault write auth/kubernetes/config ...
vault policy write simpelv2-dev policies/simpelv2-dev.hcl

📦 License

Open Source — mengikuti lisensi SIMPelv2 (MIT / Apache 2.0)

🧠 Catatan Penting

Semua konfigurasi tidak menyimpan token root di Git

Backup dilakukan via vault operator snapshot

Secrets dikelola hanya via CLI/API, tidak via Web UI