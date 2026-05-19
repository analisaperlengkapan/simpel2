# Bidirectional Sync Setup Guide

## Sinkronisasi Bidirectional GitHub ↔ GitLab

Panduan ini menjelaskan cara mengkonfigurasi sinkronisasi bidirectional antara GitHub dan GitLab untuk repository SIMPEL.

## Arsitektur Sinkronisasi

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BIDIRECTIONAL SYNC FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────┐                              ┌──────────────┐            │
│   │    GitHub    │                              │    GitLab    │            │
│   │     main     │                              │     main     │            │
│   └──────┬───────┘                              └───────┬──────┘            │
│          │                                              │                   │
│          │  PR merged                                   │  MR merged        │
│          ▼                                              ▼                   │
│   ┌──────────────┐                              ┌──────────────┐            │
│   │   GitHub     │                              │   GitLab     │            │
│   │   Actions    │                              │   CI/CD      │            │
│   └──────┬───────┘                              └───────┬──────┘            │
│          │                                              │                   │
│          │ 1. Check sync markers                        │ 1. Check markers  │
│          │ 2. Compare branches                          │ 2. Compare        │
│          │ 3. Create sync branch                        │ 3. Create branch  │
│          │ 4. Create MR on GitLab                       │ 4. Create PR      │
│          ▼                                              ▼                   │
│   ┌──────────────┐                              ┌──────────────┐            │
│   │  GitLab MR   │◀────────────────────────────▶│  GitHub PR  │            │
│   │ (review)     │        Safe sync via         │  (review)    │            │
│   └──────────────┘        MR/PR process         └──────────────┘            │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                     LOOP PREVENTION MECHANISMS                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  1. Commit message markers: [github-sync], [gitlab-sync], [skip sync]       │
│  2. Author email detection: *-sync-bot@kejaksaan.go.id                      │
│  3. Branch naming: sync/* branches are ignored                              │
│  4. Label checking: PRs/MRs with sync labels trigger skip                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Konfigurasi yang Diperlukan

### 1. GitHub Secrets

Konfigurasi secrets di **GitHub** (Settings → Secrets and variables → Actions):

| Secret Name         | Status        | Deskripsi                   | Cara Mendapatkan                                                               |
| ------------------- | ------------- | --------------------------- | ------------------------------------------------------------------------------ |
| `GITLAB_MIRROR_URL` | ✅ Configured | URL GitLab dengan token     | Sudah tersimpan, berisi URL dengan Project Access Token                        |
| `GITLAB_API_TOKEN`  | ⚠️ Required   | GitLab Project Access Token | GitLab → Settings → Access Tokens → Create (scopes: `api`, `write_repository`) |
| `GITLAB_PROJECT_ID` | ⚙️ Optional   | ID proyek GitLab            | GitLab → Settings → General → Project ID                                       |

> **Note:** `GITLAB_MIRROR_URL` sudah dikonfigurasi dan berhasil digunakan pada GitHub Action sebelumnya.
> Token yang ada di dalam URL tersebut bisa digunakan juga untuk `GITLAB_API_TOKEN` (copy bagian token saja).

### 2. GitHub Variables (Opsional)

Tambahkan variables berikut di **GitHub** (Settings → Secrets and variables → Actions → Variables):

| Variable Name     | Deskripsi                         | Value Options       |
| ----------------- | --------------------------------- | ------------------- |
| `AUTO_MERGE_SYNC` | Aktifkan auto-merge untuk sync MR | `true` atau `false` |

> **AUTO_MERGE_SYNC:**
>
> - `false` (default): MR yang dibuat perlu di-review dan merge manual
> - `true`: MR akan otomatis di-merge jika pipeline sukses (hanya untuk sync, bukan development MR)

### 3. GitLab CI/CD Variables

Tambahkan variables berikut di **GitLab** (Settings → CI/CD → Variables):

| Variable Name  | Deskripsi                       | Value                         | Protected | Masked |
| -------------- | ------------------------------- | ----------------------------- | --------- | ------ |
| `GITHUB_TOKEN` | GitHub Fine-grained PAT         | `github_pat_xxxx...`          | ✅ Yes    | ✅ Yes |
| `GITHUB_REPO`  | Repository format: `owner/repo` | `analisaperlengkapan/simpel2` | ❌ No     | ❌ No  |

#### Cara Membuat GITHUB_TOKEN

1. Buka https://github.com/settings/tokens?type=beta (Fine-grained tokens)
2. Klik **"Generate new token"**
3. Isi konfigurasi:
   - **Token name:** `gitlab-sync-token`
   - **Expiration:** 90 days atau custom (max 1 year)
   - **Repository access:** Only select repositories → pilih `analisaperlengkapan/simpel2`
   - **Permissions:**
     - `Contents`: Read and write
     - `Pull requests`: Read and write
     - `Metadata`: Read-only (otomatis)
4. Klik **"Generate token"**
5. **Copy token** (hanya ditampilkan sekali!)
6. Simpan di GitLab CI/CD Variables sebagai `GITHUB_TOKEN`

#### Nilai GITHUB_REPO

```
analisaperlengkapan/simpel2
```

Ini adalah format `owner/repository-name` dari URL GitHub: `https://github.com/analisaperlengkapan/simpel2`

## Alur Kerja Sinkronisasi

### Skenario 1: Perubahan dari GitHub

1. Developer membuat PR di GitHub
2. PR di-merge ke `main`
3. **GitHub Actions** (`sync-to-gitlab.yml`) triggered:
   - Cek apakah commit dari sync sebelumnya (skip jika ya)
   - Fetch GitLab remote
   - Bandingkan commit SHA
   - Jika GitHub ahead → buat sync branch
   - Push branch ke GitLab
   - Buat Merge Request via GitLab API
4. Reviewer di GitLab mereview dan merge MR
5. GitLab CI akan skip karena mendeteksi `[github-sync]` marker

### Skenario 2: Perubahan dari GitLab

1. Developer membuat MR di GitLab
2. MR di-merge ke `main`
3. **GitLab CI** (`sync-to-github.yml`) triggered:
   - Cek apakah commit dari sync sebelumnya (skip jika ya)
   - Fetch GitHub remote
   - Bandingkan commit SHA
   - Jika GitLab ahead → buat sync branch
   - Push branch ke GitHub
   - Buat Pull Request via GitHub API
4. Reviewer di GitHub mereview dan merge PR
5. GitHub Actions akan skip karena mendeteksi `[gitlab-sync]` marker

### Skenario 3: Branches Diverged

Jika kedua repository memiliki perubahan unik yang tidak di-sync:

1. Workflow akan mendeteksi divergence
2. Issue dibuat otomatis di GitHub dengan label `sync-conflict`
3. Intervensi manual diperlukan untuk resolve

## Mencegah Infinite Loop

Sistem menggunakan 4 mekanisme untuk mencegah loop:

### 1. Commit Message Markers

```
[github-sync] - Commit berasal dari GitHub sync
[gitlab-sync] - Commit berasal dari GitLab sync
[skip sync]   - Manual skip synchronization
[ci skip:sync] - Alternative skip directive
```

### 2. Author Email

```
github-sync-bot@kejaksaan.go.id - GitHub sync bot
gitlab-sync-bot@kejaksaan.go.id - GitLab sync bot
```

### 3. Branch Naming

Branches dengan prefix `sync/` akan di-ignore.

### 4. Labels

PR/MR dengan label `github-sync` atau `gitlab-sync` dikenali sebagai sync.

## Testing Konfigurasi

### Test GitHub → GitLab

```bash
# 1. Buat perubahan kecil
echo "Test sync $(date)" >> test-sync.txt
git add test-sync.txt
git commit -m "test: GitHub to GitLab sync test"
git push origin main

# 2. Cek GitHub Actions logs
# 3. Cek GitLab untuk MR baru
```

### Test GitLab → GitHub

```bash
# 1. Di GitLab, buat perubahan via WebIDE atau CLI
git remote add gitlab <gitlab-url>
git checkout -b test-gitlab-sync
echo "GitLab sync test $(date)" >> gitlab-test.txt
git add gitlab-test.txt
git commit -m "test: GitLab to GitHub sync test"
git push gitlab test-gitlab-sync

# 2. Buat MR di GitLab dan merge
# 3. Cek GitHub untuk PR baru
```

### Verify Token Validity

**GitHub Token (untuk GitLab CI):**

```bash
curl -H "Authorization: token <GITHUB_TOKEN>" \
  https://api.github.com/repos/analisaperlengkapan/simpel2
```

**GitLab Token (untuk GitHub Actions):**

```bash
curl -H "PRIVATE-TOKEN: <GITLAB_API_TOKEN>" \
  "https://gitlab.kejaksaan.go.id/api/v4/projects/<PROJECT_ID>"
```

## Troubleshooting

### MR/PR tidak dibuat

1. Cek workflow/pipeline logs
2. Pastikan token memiliki permission yang benar
3. Pastikan GITLAB_PROJECT_ID atau GITHUB_REPO dikonfigurasi dengan benar

### Sync loop terdeteksi

1. Cek commit message - pastikan tidak mengandung sync marker
2. Cek commit author - pastikan bukan sync bot
3. Jika stuck, tambahkan `[skip sync]` pada commit message

### Branches diverged

1. Baca issue yang dibuat otomatis
2. Manual fetch dan compare:

   ```bash
   git fetch origin main
   git fetch gitlab main
   git log origin/main..gitlab/main --oneline
   git log gitlab/main..origin/main --oneline
   ```

3. Pilih strategi merge/rebase
4. Push dengan `[skip sync]` marker

### Rate Limiting

Jika terlalu banyak sync attempts:

1. GitHub API: 5000 requests/hour (authenticated)
2. GitLab API: Varies by instance
3. Scheduled check berjalan setiap 6 jam

## File yang Relevan

| File            | Lokasi                                   | Fungsi                             |
| --------------- | ---------------------------------------- | ---------------------------------- |
| GitHub → GitLab | `.github/workflows/sync-to-gitlab.yml`   | Sync dari GitHub ke GitLab         |
| GitLab → GitHub | `.github/workflows/sync-from-gitlab.yml` | Periodic check & sync dari GitLab  |
| GitLab CI Sync  | `.gitlab/ci/sync-to-github.yml`          | GitLab CI job untuk sync ke GitHub |
| Main GitLab CI  | `.gitlab-ci.yml`                         | Includes sync configuration        |

## Security Considerations

1. **Token Storage:** Semua token disimpan sebagai secrets/protected variables
2. **Token Scope:** Gunakan minimum permissions yang diperlukan
3. **Review Process:** Sync melalui MR/PR, bukan direct push
4. **Audit Trail:** Semua sync tercatat di workflow/pipeline logs

## Rollback

Jika sync bermasalah:

1. **Disable sementara:**

   - GitHub: Disable workflow di Actions tab
   - GitLab: Set variable `SKIP_SYNC=true`

2. **Revert changes:**

   ```bash
   git revert <sync-commit-sha>
   git push origin main -m "[skip sync] Revert sync commit"
   ```

3. **Force sync ulang:**
   - Gunakan workflow_dispatch trigger dengan `force_sync: true`
