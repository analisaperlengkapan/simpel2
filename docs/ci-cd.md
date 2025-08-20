# 📦 Dokumentasi CI/CD - SIMPelv2

Dokumen ini menjelaskan alur otomatisasi *build → test → seal → deploy → release* untuk proyek SIMPelv2 berbasis GitLab CI/CD.

---

## 🚀 Tahapan Pipeline

| Tahap | Deskripsi | Tool |
|-------|-----------|------|
| `lint:yaml` | Lint seluruh file YAML K8s | `yamllint` |
| `validate:env` | Validasi `.env` terhadap `.env.example` | `validate_env.py` |
| `generate:k8s` | Generate YAML K8s dari `docker-compose.secure.yml` | `generate_k8s.py` |
| `seal:secrets` | Menyegel `.env` jadi SealedSecrets | `kubeseal`, `make seal-secret` |
| `test:simulate` | Simulasi run image dengan `docker run` | `make simulate-pod-run` |
| `deploy:dev` | Deploy otomatis ke MicroK8s dev | `make deploy-dev` |
| `release:tag` | Auto-tag versi jika commit ke `main` | `git tag` |

---

## 🛠️ Perintah Makefile yang Terlibat

Contoh perintah yang dipanggil oleh GitLab CI:

```bash
make validate-env
make lint-yaml
make generate-k8s
make seal-secret NAME=db FILE=.env
make simulate-pod-run
make deploy-dev
