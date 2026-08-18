# Custom ARC runner image untuk simpel2.
# Image actions-runner resmi minimal (tanpa C toolchain) → build Rust gagal
# `linker cc not found`. Tambahkan toolchain build yang dibutuhkan workspace
# (tonic/prost→protoc, aws-lc-sys→cmake/clang, openssl-sys→libssl-dev/pkg-config).
#
# ⚠️ VERSI BASE INI WAJIB DIJAGA TETAP BARU — ia bukan sekadar preferensi.
# GitHub men-deprecate rilis runner lama. Runner yang sudah lewat batas keluar
# dengan **exit code 7**, dan ARC menerjemahkan itu jadi kaskade yang MEMATIKAN
# SELURUH CI, bukan sekadar satu job gagal:
#
#   exit 7 → EphemeralRunner.phase=Outdated → EphemeralRunnerSet.phase=Outdated
#          → AutoscalingRunnerSet.phase=Outdated
#
# Fase terakhir itu **terkunci**: di controller 0.14.2, `outdated` dibaca dari
# status di awal Reconcile, lalu blok `if outdated` merobohkan listener + ERS +
# runner scale set dan `return` sebelum baris yang mengembalikannya ke `Running`.
# Satu-satunya jalan keluar adalah perubahan spec ARS (hash berubah → phase
# direset ke `Pending`) — jadi bump versi di sini sekaligus obat kaskadenya.
#
# Gejalanya menipu: job hanya menggantung `queued`, TIDAK merah, dan GitHub
# membatalkannya diam-diam di jam ke-24 sehingga terbaca seperti dibatalkan
# manusia. INSIDEN NYATA: 2.334.0 di-deprecate 2026-08-10 → CI mati total 4 hari
# tanpa satu pun indikator merah. Lihat `README.md` di direktori ini.
FROM ghcr.io/actions/actions-runner:2.336.0

USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      pkg-config \
      protobuf-compiler \
      libssl-dev \
      cmake \
      clang \
      libclang-dev \
      perl \
      curl wget git unzip xz-utils jq file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Docker Compose v2 sebagai CLI plugin (#48).
#
# Sebelum ini, TUJUH job di ci.yml masing-masing mengunduh biner yang sama ke
# `$HOME/.docker/cli-plugins` sebelum bisa memakainya. Artinya tujuh unduhan
# GitHub Releases per run, tujuh peluang gagal karena jaringan, dan tujuh tempat
# yang harus diingat saat versinya naik — sementara image runner-nya sendiri,
# yang HANYA dibangun ulang saat berubah, adalah tempat yang tepat untuk itu.
#
# Dipasang ke path plugin SISTEM (bukan $HOME) supaya berlaku untuk user mana
# pun yang menjalankan job. Versi dipertahankan persis seperti yang selama ini
# diunduh ci.yml (v2.32.4) agar perubahan ini murni "pindah tempat", bukan
# sekaligus naik versi; menaikkannya kini cukup satu baris di sini.
#
# Checksum diverifikasi: unduhan tak-terverifikasi di dalam image CI = jalur
# supply-chain yang diam-diam dipercaya oleh setiap job.
ARG COMPOSE_VERSION=v2.32.4
ARG COMPOSE_SHA256=ed1917fb54db184192ea9d0717bcd59e3662ea79db48bff36d3475516c480a6b
RUN set -eux; \
    mkdir -p /usr/local/lib/docker/cli-plugins; \
    curl -fsSL \
      "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-linux-x86_64" \
      -o /usr/local/lib/docker/cli-plugins/docker-compose; \
    echo "${COMPOSE_SHA256}  /usr/local/lib/docker/cli-plugins/docker-compose" | sha256sum -c -; \
    chmod 0755 /usr/local/lib/docker/cli-plugins/docker-compose; \
    /usr/local/lib/docker/cli-plugins/docker-compose version

USER runner
