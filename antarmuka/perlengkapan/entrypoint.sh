#!/bin/sh
set -e

# Substitute the nginx upstream host:port placeholders from env. This matters
# ONLY under docker-compose, where the image ships its own nginx.conf (chowned
# to uid 101) and the placeholders are still in it.
#
# Under Helm the chart mounts its own nginx.conf from a ConfigMap over this
# path. A ConfigMap mount is read-only NO MATTER WHO OWNS THE FILE, and that
# rendered config carries no placeholders at all — there is nothing to
# substitute. The previous version ran `sed -i` unconditionally under `set -e`,
# so in Kubernetes it died with
#   sed: can't create temp file '/etc/nginx/nginx.confXXXXXX': Read-only file system
# and the container CrashLooped. (`sed -i` writes its temp file into the
# target's DIRECTORY, so mounting the single file writable would not help
# either.) It passed CI because docker-compose never mounts over the file.
#
# So: only substitute when there is something to substitute and we can write.
PERLENGKAPAN_API_UPSTREAM="${PERLENGKAPAN_API_UPSTREAM:-layanan-perlengkapan:8093}"
AUTHENC_API_UPSTREAM="${AUTHENC_API_UPSTREAM:-authenc:8091}"
if grep -q '__PERLENGKAPAN_API_UPSTREAM__\|__AUTHENC_API_UPSTREAM__' /etc/nginx/nginx.conf 2>/dev/null; then
  if sed -i \
      -e "s|__PERLENGKAPAN_API_UPSTREAM__|${PERLENGKAPAN_API_UPSTREAM}|g" \
      -e "s|__AUTHENC_API_UPSTREAM__|${AUTHENC_API_UPSTREAM}|g" \
      /etc/nginx/nginx.conf 2>/dev/null; then
    echo "entrypoint: nginx.conf upstream placeholders substituted"
  else
    # Placeholders present but unwritable: that IS a misconfiguration — nginx
    # would proxy to a literal `__..__` host. Fail loudly rather than serve a
    # config that cannot work.
    echo "entrypoint: FATAL - nginx.conf has upstream placeholders but is not writable" >&2
    exit 1
  fi
else
  echo "entrypoint: no upstream placeholders in nginx.conf (Helm ConfigMap already rendered) - skipping substitution"
fi

# Generate config.json di lokasi canonical microfrontend
# (/perlengkapan/simpel/v2/) — same path dengan WASM `<base href>`.
#
# Di bawah Helm ini TIDAK PERNAH berhasil: chart menyetel
# `readOnlyRootFilesystem: true`, jadi web root tak bisa ditulis. Itu bukan
# regresi rc25 — pod rc24 pun tak punya config.json (diperiksa langsung di
# staging 2026-08-25), dan portal persis sama. FE jalan memakai default
# build-time, jadi kegagalan ini tak fatal; yang fatal adalah `set -e` yang
# baru ditambahkan, karena redirect yang gagal akan mematikan container.
#
# Ditangani eksplisit supaya keadaannya TERBACA, bukan tersembunyi: berhasil
# dicatat, gagal dicatat sebagai peringatan. Perbaikan sebenarnya = sajikan
# config.json dari ConfigMap chart (seperti nginx.conf) supaya tak ada tulisan
# runtime sama sekali; itu butuh verifikasi FE tersendiri.
CONFIG_JSON=/usr/share/nginx/html/perlengkapan/simpel/v2/config.json
if cat <<EOT > "$CONFIG_JSON" 2>/dev/null
{
  "api_url": "${API_URL:-http://10.1.7.121/api/v1/perlengkapan}",
  "authenc_url": "${AUTHENC_URL:-http://10.1.7.121/api/auth}"
}
EOT
then
  echo "entrypoint: wrote $CONFIG_JSON"
else
  echo "entrypoint: WARNING - could not write $CONFIG_JSON (read-only web root); FE falls back to build-time defaults" >&2
fi

# ── CSP script-src: 'unsafe-inline' → per-script hashes (opt-in) ──────────────
#
# `script-src 'unsafe-inline'` lets any injected <script> run. Trunk needs it only
# for the inline WASM boot, so the image ships the SHA-256 of each inline script
# (computed at build time by infra/scripts/csp-script-hashes.py, the only place
# the built index.html is known) and nginx swaps `'unsafe-inline'` for them when
# CSP_HASH_MODE=true. OFF by default: the switch is thrown per environment, on
# staging first, after checking the app boots with the stricter policy.
#
# nginx reads the hashes from an `include` glob; when the file is absent the CSP
# falls back to 'unsafe-inline' (see the `$csp_script_inline` map), so the default
# path — and an image without the hash file — behaves exactly as before.
CSP_HASH_FILE=/tmp/csp-script-hashes.conf
rm -f "$CSP_HASH_FILE" 2>/dev/null || true
if [ "${CSP_HASH_MODE:-false}" = "true" ]; then
  CSP_HASHES="$(cat /usr/share/nginx/csp-script-hashes.txt 2>/dev/null || true)"
  if [ -z "$CSP_HASHES" ]; then
    echo "entrypoint: FATAL - CSP_HASH_MODE=true but /usr/share/nginx/csp-script-hashes.txt is missing/empty" >&2
    exit 1
  fi
  if printf 'default "%s";\n' "$CSP_HASHES" > "$CSP_HASH_FILE" 2>/dev/null; then
    echo "entrypoint: CSP script-src pinned to inline-script hashes ($CSP_HASH_FILE)"
  else
    echo "entrypoint: FATAL - CSP_HASH_MODE=true but $CSP_HASH_FILE is not writable" >&2
    exit 1
  fi
else
  echo "entrypoint: CSP script-src keeps 'unsafe-inline' (CSP_HASH_MODE is not true)"
fi

# Execute the default CMD (nginx)
exec "$@"
