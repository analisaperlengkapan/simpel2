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
USER runner
