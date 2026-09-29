#!/usr/bin/env python3
"""Jalankan pemeriksaan dependency dengan retry, lalu klasifikasikan hasilnya.

## Masalah yang dijawab

Pemeriksaan dependency (`cargo audit`, `cargo deny`, `composer audit`) semuanya
mengunduh sesuatu dari internet sebelum ia bisa memeriksa apa pun:

  * `cargo audit` / `cargo deny` mengambil DB advisory RustSec
    (`github.com/rustsec/advisory-db`) — sebuah `git clone`/fetch;
  * `cargo deny` juga menarik crate registri untuk membangun graph;
  * `composer audit` mengambil DB advisory Packagist, dan container PHP-nya
    sendiri harus di-`docker pull`;
  * memasang alatnya sendiri mengunduh rilis GitHub.

Kegagalan di lapisan itu **tidak ada hubungannya dengan kesehatan dependency
proyek**. Tapi tanpa pembedaan, ketiganya muncul sebagai job merah yang sama
persis, dan yang paling sering terjadi justru yang paling tidak berarti: satu
koneksi putus membaca sebagai "ada kerentanan".

Sejarah repo ini sudah membayar ongkosnya berkali-kali — lihat `composer-install`
action (retry karena `--prefer-dist` mematikan source fallback, sehingga satu
`curl error 28` di antara 159 unduhan langsung mematikan job), `release.yml`
(plugin Blade bertag mengambang), dan #711 (actionlint lewat GitHub API). Pola
umumnya selalu sama: **jangan pernah menggantungkan build pada unduhan jaringan
tanpa pin + retry.**

## Tiga klasifikasi (bukan dua)

Modul ini mengembalikan salah satu dari tiga, dan itu disengaja:

  GREEN — alat berjalan dan **tidak menemukan apa pun**. Lulus.
  RED   — alat berjalan dan **menemukan sesuatu**, ATAU gagal karena sebab
          yang bukan jaringan (advisory tidak bisa di-parse, config rusak,
          crate hilang). Ini sinyal nyata; jangan ditelan.
  HANG  — alat tidak pernah menjawab, atau menjawab dengan kegagalan jaringan
          di SETIAP percobaan. Dihitung **hijau** (lihat di bawah).

### Kenapa HANG = hijau

Kebijakan eksplisit pemilik repo: *"kalau misal masih belum ada respon dianggap
green saja, kalau misal ada respon dan tidak ada masalah berarti green, kalau
masih ada yang masalah dependency berarti red."*

Dasarnya bukan "biar CI gampang", melainkan bahwa sebuah pemeriksaan yang tidak
bisa dijalankan karena jaringan tidak membuktikan apa pun TENTANG PROYEK. Job
merah dalam keadaan itu tidak memberi tahu siapa pun bahwa dependency-nya rusak —
ia hanya memberi tahu bahwa GitHub/Docker Hub/crates.io sedang tidak bisa
dihubungi. Sementara itu, alternatifnya lebih buruk lagi: pemeriksaan yang merah
saat jaringan putus adalah pemeriksaan yang akan diajarkan untuk diabaikan, dan
begitu ia diabaikan, ia juga berhenti menangkap temuan sungguhan.

Yang membuat klasifikasi ini aman adalah **RED tetap merah**. Kegagalan jaringan
disaring secara sempit: hanya pola transport (resolve, connect, TLS, timeout,
5xx, dan kalimat "could not download/network unreachable" milik alat masing-
masing). Apa pun yang menyerupai temuan, atau parse error, atau config error,
jatuh ke RED. Sebuah advisory tidak pernah bisa lolos lewat jalur hijau ini.

### Batas yang jujur

Tidak ada retry yang bisa membedakan "registry sedang down" dari "registry
selamanya hilang untuk kita". Setelah percobaan habis, ini memilih hijau — dan
konsekuensinya nyata: pemeriksaan yang sedang dibungkam tidak akan menangkap
temuan baru selama jaringan tetap buruk. Karena itu setiap keputusan hijau-karena-
jaringan ditulis ke `GITHUB_STEP_SUMMARY` dengan pola yang cocok, supaya "hijau"
dan "tidak pernah benar-benar berjalan" tidak terlihat sama. Jangan mengubah
perilaku itu menjadi senyap.

## Pemakaian

    python3 infra/scripts/retry-dependency-check.py \\
        --name "cargo audit" -- cargo audit

Keluaran: ringkasan ke stdout + `$GITHUB_STEP_SUMMARY` (bila ada).
Keluar 0 untuk GREEN, 1 untuk RED.

Opsi:
    --attempts N   jumlah percobaan (default 4)
    --base-delay S detik sebelum retry pertama, digandakan tiap ulang (default 5)
    --timeout S    batas waktu per percobaan (default 300) — INI yang membuat
                   HANG terdeteksi cepat alih-alih menggantung 30 menit sampai
                   timeout job
    --allow-failure  jangan pernah keluar 1 (untuk job advisory yang hanya
                     melaporkan). Klasifikasi tetap dicetak.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import time

# ─────────────────────────────────────────────────────────────────────────────
# Pola kegagalan NETWORK — sengaja sempit.
#
# Setiap pola di sini adalah pesan TRANSPORT, bukan pesan semantik. Aturannya:
# kalau sebuah string bisa muncul pada kegagalan yang berarti "dependency kita
# bermasalah", ia TIDAK boleh ada di sini.
# ─────────────────────────────────────────────────────────────────────────────
NETWORK_PATTERNS = [
    # reqwest/rustls/hyper — jalur yang dipakai cargo, cargo-deny, composer.
    r"error sending request",
    r"failed to connect",
    r"connection (refused|reset|closed|timed out|aborted)",
    r"could not connect",
    r"tcp connect error",
    r"dns error",
    r"failed to lookup address",
    r"name or service not known",
    r"temporary failure in name resolution",
    r"network is unreachable",
    r"no route to host",
    r"i/o timeout",
    # Frasa timeout TRANSPORT, bukan substring "timeout". Pola `timed? ?out`
    # sebelumnya cocok dengan NAMA CRATE (`hyper-timeout`, `tokio-io-timeout`
    # ada di pohon dependensi ini) — sehingga kegagalan deterministik apa pun
    # yang mencetak pohon itu terbaca "jaringan" dan lolos hijau.
    r"\btimed out\b",
    r"timeout was reached",  # libcurl/cargo "[28] Timeout was reached"
    r"operation was canceled",  # reqwest cancellation (koneksi putus)
    r"incomplete message",
    r"unexpected eof",
    r"connection closed before message completed",
    r"tls handshake",
    # Kegagalan VERIFIKASI SERTIFIKAT (`certificate verify failed`, `schannel: `,
    # dll.) TIDAK ada di sini: itu kegagalan integritas — bisa MITM atau CA
    # rusak — bukan ketersediaan. Lihat TLS_INTEGRITY_PATTERNS: RED.
    # HTTP 5xx dari registry / releases API.
    r"http server returned (5\d\d)",
    r"status code (5\d\d)",
    r"(50[0-9]) (internal server error|bad gateway|service unavailable|gateway timeout)",
    # Git — advisories DB di-fetch sebagai git repo.
    #
    # `failed to fetch` sengaja DISEMPITKAN dengan konteks. Tanpa itu ia
    # bertabrakan dengan pesan cargo-deny "failed to fetch crates: failed to
    # run cargo", yang muncul saat toolchain-nya HILANG — kegagalan setup yang
    # deterministik, bukan jaringan. Ditemukan justru karena diuji lawan
    # biner sungguhan; sebagai pola telanjang ia mengubah setup rusak jadi
    # hijau palsu.
    r"failed to fetch (from|into|https?://|advisory|index|repository|remote)",
    r"could not resolve host",
    # `unable to access '<url>': <sebab>` — HANYA bila sebabnya transport. Tanpa
    # syarat itu ia menelan 401/403/404 (repo hilang, izin dicabut) yang
    # deterministik.
    r"unable to access '[^']*':\s*(could not resolve host|failed to connect|"
    r"recv failure|connection (reset|timed out|refused)|operation timed out|"
    r"empty reply|http/2 stream|the requested url returned error: 5\d\d)",
    r"git protocol error",
    r"remote end hung up",
    r"early eof",
    r"the remote end hung up unexpectedly",
    # Docker (composer audit menarik image PHP). Hanya galat daemon yang sifatnya
    # transport/registry-5xx; `manifest unknown` (tag terpin hilang/salah) dan
    # `pull access denied` PERMANEN, jadi bukan di sini — pemeriksaan yang tak
    # akan pernah berjalan tidak boleh tampak lulus.
    r"error response from daemon:\s*(get|head|received unexpected http status)",
    r"failed to resolve source metadata",
    # Galat unduh yang eksplisit milik alat-alat ini.
    r"could not download",
    r"failed to download",
    r"error downloading",
    r"download failed",
    r"failed to update advisory database",
    r"failed to fetch advisory",
    r"error updating index",
    r"failed to get (crate|index|registry)",
    # Bukan `crates.io` telanjang: hostname itu tercetak di banner rutin
    # (`Updating crates.io index`) SETIAP jalan cargo, jadi kegagalan
    # deterministik (mis. gagal memilih versi) ikut terbaca "jaringan".
    r"spurious network error",
]

# ─────────────────────────────────────────────────────────────────────────────
# Kegagalan INTEGRITAS transport — RED, tanpa retry.
#
# Sertifikat TLS yang tidak lolos verifikasi bukan "server tidak menjawab":
# server menjawab dengan identitas yang tak bisa dipercaya (MITM, CA korporat
# berubah, sertifikat kedaluwarsa). Menghitungnya hijau berarti penyerang di
# jalur jaringan bisa membisukan pemeriksaan advisory cukup dengan menyodorkan
# sertifikat tak sah.
# ─────────────────────────────────────────────────────────────────────────────
TLS_INTEGRITY_PATTERNS = [
    r"certificate verify failed",
    r"unable to get local issuer certificate",
    r"self[- ]signed certificate",
    r"certificate has expired",
    r"ssl certificate problem",
    r"invalid peer certificate",
    r"unknownissuer",
    r"unknown ca",
    r"bad certificate",
    r"certificate (is )?not trusted",
    # Windows SChannel's verification failures. NOT the bare word `schannel`: it is
    # also a crate in every dependency tree that has native-tls (`schannel v0.1.x`),
    # and cargo-deny prints that whole tree. As a bare word it turned a PASSING
    # cargo-deny run red on every commit.
    r"schannel: ",
    r"sec_e_(untrusted_root|cert_expired|wrong_principal|cert_unknown)",
]
TLS_INTEGRITY_RE = re.compile("|".join(TLS_INTEGRITY_PATTERNS), re.I)

# ─────────────────────────────────────────────────────────────────────────────
# Pola SETUP/DECISIVE — diperiksa LEBIH DULU, dan menghasilkan RED tanpa retry.
#
# Dua isinya: (a) temuan sungguhan, (b) kegagalan lingkungan yang DETERMINISTIK —
# daemon mati, berkas hilang, izin ditolak. Yang (b) sengaja tidak dianggap
# hijau walau ia juga "tidak menjawab": menggagalkan retry tak akan mengubah
# apa pun, dan meloloskannya akan membuat pemeriksaan yang tak pernah berjalan
# terlihat persis seperti pemeriksaan yang lulus — persis kelas cacat yang
# dikejar repo ini di tempat lain (middleware yang tak pernah di-mount,
# kredensial cache-miss yang tak pernah teruji).
# ─────────────────────────────────────────────────────────────────────────────
DECISIVE_PATTERNS = [
    # (a) Temuan / hasil pemeriksaan yang nyata.
    r"vulnerabilit(y|ies) found",
    r"advisories? (found|detected)",
    r"error\[advisories\]",
    r"error\[bans\]",
    r"error\[licenses\]",
    r"error\[sources\]",
    r"security advisory",
    r"^error: .*(deny|banned|allowed|unmatched)",
    r"unmatched skip",
    r"advisory-not-detected",
    r"license-not-encountered",
    # (b) Kegagalan setup yang deterministik. Sengaja SPESIFIK: pola selebar
    # `no such file or directory` akan menyerap galat DNS asli
    # (getaddrinfo memunculkannya) dan mengubahnya jadi RED palsu.
    r"docker daemon",
    r"docker\.sock",
    r"cannot connect to the docker daemon",
    r"is the docker daemon running",
    r"permission denied",
    r"error: could not find `?Cargo\.toml`?",
    r"error: no such (file|manifest)",
    # Alatnya butuh biner lain yang tidak ada di PATH (cargo-deny -> cargo).
    # Deterministik, bukan jaringan.
    r"failed to run cargo",
    r"failed to start `?cargo",
    r"os error 2",
    r"no such file or directory \(os error 2\)",
    r"failed to parse",
    r"invalid (config|toml|manifest)",
    r"error: unable to parse",
    r"command not found",
]

NETWORK_RE = re.compile("|".join(NETWORK_PATTERNS), re.I)
DECISIVE_RE = re.compile("|".join(DECISIVE_PATTERNS), re.I | re.M)

# Kalimat BERSIH yang memuat kata "found"/"detected" tetapi justru berarti TIDAK
# ADA temuan. Tanpa dinetralkan, pola sinyal temuan di atas membacanya sebagai
# temuan — sehingga pemindaian yang LULUS dilaporkan sebagai kegagalan.
#
# Ini bukan hipotetis. `composer audit` yang bersih mencetak tepat
# "No security vulnerability advisories found.", dan sejak penjaga ini dipasang
# job `composer-audit (simpelv1)` merah tanpa satu pun advisory. Pola
# `advisories? (found|detected)` cocok dengan frasa itu apa adanya. Sinyal merah
# yang tak berarti apa-apa adalah sinyal yang akan diabaikan, dan begitu diabaikan
# ia juga menyembunyikan temuan yang sungguhan.
CLEAN_PATTERNS = [
    r"\bno\s+(security\s+)?(vulnerabilit(?:y|ies)|advisories?)\b[^\n]{0,40}?\b(found|detected)\b",
    r"\b0\s+(vulnerabilit(?:y|ies)|advisories?)\b[^\n]{0,40}?\b(found|detected)\b",
]
CLEAN_RE = re.compile("|".join(CLEAN_PATTERNS), re.I)

# Untuk mengekstrak baris yang menjawab "temuan apa?" saat verdict RED, supaya
# ringkasannya bisa didiagnosis tanpa membuka log mentah.
ID_RE = re.compile(r"RUSTSEC-\d{4}-\d{4}|CVE-\d{4}-\d{4,}|GHSA-[0-9a-z-]{4,}")
FIELD_RE = re.compile(r"^(Crate|Version|Title|ID|Date|Solution|Severity|Warning):")

# `cargo audit` mewarnai keluarannya secara default, jadi barisnya berbentuk
# `\x1b[1mCrate:\x1b[0m rustls` — bukan `Crate: rustls`. Tanpa dibersihkan,
# pencocokan ber-anchor `^` (FIELD_RE) tak pernah kena, sehingga ekstraksi
# "temuan apa?" diam-diam menghasilkan NOL baris di dunia nyata (terbukti pada
# run 2026-09-25: bagian sinyal tak muncul sama sekali, padahal direproduksi
# sintetis lulus karena tak ada kode ANSI di sana).
#
# Dibersihkan di sini, bukan dengan mengandalkan `--color never`, supaya
# pembungkus tetap benar untuk alat apa pun dan pemanggil apa pun.
ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")


def strip_ansi(text: str) -> str:
    return ANSI_RE.sub("", text)


CRATE_RE = re.compile(r"^Crate:\s*(\S+)")
WARNING_FIELD_RE = re.compile(r"^Warning:")
DEPENDENCY_BLOCK_RE = re.compile(r"^(Dependency tree|Tree):")


def extract_findings(output: str, max_blocks: int = 3) -> list[str]:
    """Ambil header blok temuan SUNGGUHAN dari keluaran `cargo audit`.

    Kenapa diparse per-blok, bukan dengan mengangkur ke baris `error:`:
    `run_once` menangkap stdout dan stderr **terpisah** lalu menyambungnya
    (`stdout + stderr`). `cargo audit` menulis laporan ke stdout dan baris
    ringkasan `error: N vulnerability found!` ke stderr, jadi di keluaran yang
    benar-benar diklasifikasi baris `error:` itu berada di UJUNG — ratusan
    baris sesudah blok advisory yang diterangkannya. Jendela ±baris di sekitar
    anchor karena itu tak pernah menjangkau bloknya, dan ekstraksi diam-diam
    hanya menyisakan baris `error:` itu sendiri (persis yang terjadi di CI
    2026-09-25). Sebelumnya ia tampak berhasil hanya karena log CI yang sudah
    di-merge men-interleave kedua stream sehingga `error:` kebetulan mendarat
    di tengah blok — bergantung pada urutan yang kebetulan, bukan pada isi.

    Parser ini membaca blok apa adanya: tiap `Crate:` membuka blok, `Dependency
    tree:` menutupnya (pohon dependensi bisa ratusan baris dan bukan bagian
    dari jawaban "temuan apa?"). Sebuah blok dihitung temuan bila punya
    `Severity:` atau punya `ID:` TANPA `Warning:` — di `cargo audit`,
    advisory informasional (unmaintained/yanked/unsound) selalu memuat
    `Warning:` dan tak pernah memuat `Severity:`. Dengan begitu blok peringatan
    tidak ikut membanjiri ringkasan.
    """
    out: list[str] = []
    block: list[str] = []

    def flush() -> None:
        if not block or len(out) >= max_blocks * 14:
            block.clear()
            return
        has_warning = any(WARNING_FIELD_RE.match(l) for l in block)
        has_severity = any(l.startswith("Severity:") for l in block)
        has_id = any(l.startswith("ID:") for l in block)
        if has_severity or (has_id and not has_warning):
            out.extend(block)
            out.append("")

    for raw in output.splitlines():
        stripped = raw.strip()
        if CRATE_RE.match(stripped):
            flush()
            block = [stripped]
            continue
        if DEPENDENCY_BLOCK_RE.match(stripped):
            flush()
            block = []
            continue
        if not block:
            continue
        if FIELD_RE.match(stripped) and not CLEAN_RE.search(raw):
            block.append(stripped)
        elif ID_RE.search(raw):
            block.append(stripped)
    flush()

    return out[: max_blocks * 14]



def classify(output: str, returncode: int, timed_out: bool) -> tuple[str, str]:
    """Tentukan GREEN / RED / HANG beserta alasannya.

    Urutannya penting: timeout dan kegagalan jaringan diperiksa setelah
    memastikan tak ada sinyal keras yang muncul. Kalau sebuah temuan sudah
    tercetak di output, proses yang kemudian mati karena jaringan tetap RED —
    temuan itu nyata dan kita sudah melihatnya.

    Kalimat penyangkalan ("no ... advisories found") dinetralkan lebih dulu,
    per-baris, supaya pernyataan bersih tidak dibaca sebagai temuan. Neutralisasi
    itu per-baris (`[^\\n]`), jadi "No vulnerabilities found" di satu baris tidak
    bisa menelan temuan di baris berikutnya.
    """
    scrubbed = CLEAN_RE.sub(" ", output)

    if DECISIVE_RE.search(scrubbed):
        return "RED", "output memuat sinyal temuan/konfigurasi — bukan kegagalan jaringan"

    # Sertifikat yang tak lolos verifikasi = integritas, bukan ketersediaan.
    # Diperiksa SEBELUM timeout/jaringan supaya tak bisa diselamatkan menjadi
    # HANG (hijau) oleh jalur di bawah.
    #
    # Hanya untuk proses yang GAGAL (non-nol atau timeout). Proses yang keluar 0
    # sudah memutuskan bahwa ia lulus; teks di outputnya (pohon dependensi yang
    # memuat nama crate, kutipan dokumentasi) bukan bukti kegagalan TLS. Tanpa
    # syarat ini, cargo-deny yang LULUS dilaporkan merah.
    if (timed_out or returncode != 0) and TLS_INTEGRITY_RE.search(output):
        return "RED", "verifikasi sertifikat TLS gagal — masalah integritas, bukan ketersediaan"

    if timed_out:
        return "HANG", "tidak ada jawaban dalam batas waktu"

    if returncode == 0:
        return "GREEN", "selesai tanpa temuan"

    if NETWORK_RE.search(output):
        return "HANG", "kegagalan jaringan/transport"

    # Keluar non-nol tanpa satu pun pola di atas: kita tidak tahu apa ini, jadi
    # diperlakukan sebagai sinyal nyata. Menebak "pasti cuma jaringan" di sini
    # adalah persis cara sebuah pemeriksaan berhenti berguna tanpa ada yang
    # menyadarinya.
    return "RED", "keluar non-nol tanpa pola jaringan yang dikenali"


SELF_TESTS: list[tuple[str, str, int, bool, str]] = [
    # (nama, output, returncode, timed_out, verdict yang diharapkan)
    ("sukses bersih", "", 0, False, "GREEN"),
    ("temuan cargo-audit", "error: 2 vulnerabilities found!\n", 1, False, "RED"),
    ("temuan cargo-deny", "error[advisories]: 1 advisory found\n", 1, False, "RED"),
    ("temuan composer", "Found 3 security vulnerability advisories\n", 1, False, "RED"),
    # cargo-audit mewarnai keluarannya. Pola ber-anchor `^` (mis. `^error:`) tidak
    # akan kena bila kode ANSI belum dibersihkan — dan itu membuat verdict salah
    # DIAM-DIAM. Kasus ini ada karena versi sebelumnya lulus uji sintetis tanpa
    # ANSI sementara di CI sungguhan gagal mendeteksi temuan apa pun.
    ("temuan berwarna (ANSI) tetap RED",
     "\x1b[0m\x1b[1m\x1b[31merror:\x1b[0m 1 vulnerability found!\n", 1, False, "RED"),
    ("cargo-deny berwarna (ANSI) tetap RED",
     "\x1b[1merror\x1b[0m\x1b[1m[advisories]\x1b[0m: 1 advisory found\n", 1, False, "RED"),
    # Kalimat penyangkalan: pemindaian BERSIH tak boleh dibaca sebagai temuan.
    ("composer bersih (baseline v4)",
     "-------------------------------------\n"
     "No security vulnerability advisories found.\n", 0, False, "GREEN"),
    ("cargo-audit bersih",
     "No vulnerabilities found\n", 0, False, "GREEN"),
    ("penyangkalan bersih tapi alat keluar non-nol",
     "No security vulnerability advisories found.\n", 1, False, "RED"),
    # Batas neutralisasi: per-baris. Penyangkalan di satu baris TIDAK boleh
    # menelan temuan nyata di baris berikutnya.
    ("penyangkalan + temuan di baris lain tetap RED",
     "No vulnerabilities found\n"
     "error: 2 vulnerabilities found!\n", 1, False, "RED"),
    ("composer bersih + temuan di baris lain tetap RED",
     "No security vulnerability advisories found.\n"
     "Found 1 security vulnerability advisory\n", 1, False, "RED"),
    ("temuan composer di baris yang sama tetap RED",
     "Found 3 security vulnerability advisories\n", 1, False, "RED"),
    ("jaringan: connect refused",
     "error: failed to update advisory database: error sending request: tcp connect error", 1, False, "HANG"),
    ("jaringan: DNS",
     "fatal: unable to access 'https://github.com/rustsec/advisory-db/': "
     "Could not resolve host: github.com", 128, False, "HANG"),
    # Handshake yang putus di tengah = transport (HANG). Sertifikat yang GAGAL
    # diverifikasi = integritas (RED): server menjawab, tapi identitasnya tak
    # bisa dipercaya. Menghitungnya hijau membiarkan penyerang di jalur jaringan
    # membisukan pemeriksaan advisory hanya dengan menyodorkan sertifikat palsu.
    ("jaringan: TLS handshake terputus",
     "error: tls handshake eof", 1, False, "HANG"),
    ("TLS: sertifikat gagal diverifikasi = RED",
     "error: tls handshake failure: certificate verify failed", 1, False, "RED"),
    ("TLS: sertifikat self-signed = RED",
     "fatal: unable to access 'https://github.com/x/y/': SSL certificate problem: "
     "self signed certificate in certificate chain", 128, False, "RED"),
    ("TLS: sertifikat + timeout tak bisa diselamatkan jadi hijau",
     "error: certificate verify failed", 124, True, "RED"),
    # Regresi: cargo-deny mencetak seluruh pohon dependensi, dan `schannel` adalah
    # nama crate di dalamnya. Proses yang LULUS tidak boleh dibaca sebagai kegagalan TLS.
    ("TLS: nama crate `schannel` di pohon dependensi, keluar 0 = GREEN",
     "warning[duplicate]: found 2 duplicate entries for crate 'zip'\n"
     "    └── native-tls v0.2.18\n        └── schannel v0.1.29\n"
     "    └── rustls-native-certs v0.8.1\n", 0, False, "GREEN"),
    ("TLS: `schannel` sebagai nama crate saja tidak mengubah kegagalan jaringan jadi RED",
     "error: failed to fetch index: connection reset by peer\n"
     "    └── schannel v0.1.29\n", 1, False, "HANG"),
    ("TLS: kegagalan SChannel sungguhan = RED",
     "curl: (35) schannel: SEC_E_UNTRUSTED_ROOT (0x80090325)", 35, False, "RED"),
    ("jaringan: registry 5xx",
     "Error response from daemon: received unexpected HTTP status: 503 Service Unavailable", 1, False, "HANG"),
    ("timeout", "connecting...", 124, True, "HANG"),
    ("daemon mati (RED, bukan hijau)",
     "failed to connect to the docker API at unix:///var/run/docker.sock: "
     "dial unix /var/run/docker.sock: connect: no such file or directory", 1, False, "RED"),
    ("DNS getaddrinfo TIDAK boleh jadi RED",
     "error: failed to fetch index: getaddrinfo: no such file or directory", 1, False, "HANG"),
    ("unmatched", "segfault", 139, False, "RED"),
    ("cargo-deny tanpa toolchain = RED, bukan hijau",
     "2026-09-24 [ERROR] failed to fetch crates: failed to run cargo: "
     "No such file or directory (os error 2)\n"
     "2026-09-24 [ERROR] failed to start `cargo metadata`: "
     "No such file or directory (os error 2)\n", 1, False, "RED"),
    ("cargo-deny clone advisory DB gagal = HANG",
     "error: failed to fetch from https://github.com/rustsec/advisory-db: "
     "Could not resolve host: github.com\n", 1, False, "HANG"),
    ("temuan + noise jaringan tetap RED",
     "warning: error sending request for https://crates.io\n"
     "error: 1 vulnerability found\n", 1, False, "RED"),
    ("temuan lalu mati karena timeout tetap RED",
     "error: 4 vulnerabilities found!\n", 124, True, "RED"),
    # ── Regresi FALSE-GREEN (audit PR #930) ───────────────────────────────────
    # Tiap kasus di bawah adalah kegagalan DETERMINISTIK yang dulu terbaca
    # "jaringan" dan lolos hijau. Pemeriksaan yang tak pernah berjalan tak boleh
    # tampak persis seperti pemeriksaan yang lulus.
    ("banner crates.io + gagal pilih versi = RED",
     "    Updating crates.io index\n"
     "error: failed to select a version for the requirement `foo = \"^9\"`\n",
     101, False, "RED"),
    ("tag Docker terpin hilang (manifest unknown) = RED",
     "Unable to find image 'composer:9.9.9' locally\n"
     "docker: Error response from daemon: manifest unknown: manifest unknown.\n",
     125, False, "RED"),
    ("pull access denied = RED",
     "docker: Error response from daemon: pull access denied for x/y, repository "
     "does not exist or may require 'docker login'\n", 125, False, "RED"),
    ("nama crate berisi 'timeout' + galat lain = RED",
     "hyper-timeout v0.5.2\ntokio-io-timeout v1.2.0\n"
     "error: failed to load manifest for workspace member `/w/lib/x`\n",
     101, False, "RED"),
    ("panic yang menyebut crates.io = RED",
     "thread 'main' panicked at 'index out of bounds' "
     "(see https://crates.io/crates/cargo-audit)\n", 101, False, "RED"),
    ("git 404 (repo hilang) = RED, bukan jaringan",
     "fatal: unable to access 'https://github.com/x/gone/': "
     "The requested URL returned error: 404\n", 128, False, "RED"),
    ("galat daemon non-transport (mount) = RED",
     "docker: Error response from daemon: invalid mount config for type \"bind\"\n",
     125, False, "RED"),
    # Kontrol positif: transport yang SAH tetap HANG setelah penyempitan.
    ("git 503 = HANG",
     "fatal: unable to access 'https://github.com/x/y/': "
     "The requested URL returned error: 503\n", 128, False, "HANG"),
    ("libcurl [28] Timeout was reached = HANG",
     "error: failed to download from `https://index.crates.io/config.json`\n"
     "Caused by: [28] Timeout was reached\n", 101, False, "HANG"),
    ("docker GET registry putus = HANG",
     "docker: Error response from daemon: Get \"https://registry-1.docker.io/v2/\": "
     "net/http: request canceled while waiting for connection\n", 125, False, "HANG"),
]


def selftest() -> int:
    """Buktikan klasifikasi bisa MERAH, bukan hanya bisa hijau.

    Sebuah penjaga yang tak pernah diuji lawan regresi yang ia klaim tangkap
    adalah penjaga yang akan lulus saat barisnya dihapus. Repo ini sudah
    menemukan tepat itu: penjaga cakupan CI versi pertama lulus walau `run:`-nya
    dihapus, karena nama step-nya masih memuat perintahnya. Karena itu setiap
    resiko di bawah di-assert.
    """
    failures = []
    for name, output, rc, timed_out, want in SELF_TESTS:
        got, reason = classify(output, rc, timed_out)
        mark = "ok " if got == want else "FAIL"
        if got != want:
            failures.append(f"{name}: mau {want}, dapat {got} ({reason})")
        print(f"  {mark} {name}: {got}")

    # Kasus terpenting: sebuah temuan tidak boleh pernah terserap jadi hijau.
    for name, output, rc, timed_out, want in SELF_TESTS:
        if want == "RED":
            got, _ = classify(output, rc, timed_out)
            if got == "HANG":
                failures.append(f"TEMUAN LOLOS JADI HIJAU: {name}")

    failures += selftest_capture()

    if failures:
        print("\nSELF-TEST GAGAL:", file=sys.stderr)
        for f in failures:
            print("  " + f, file=sys.stderr)
        return 1
    print(f"\nself-test lulus ({len(SELF_TESTS)} kasus klasifikasi + capture)")
    return 0


def selftest_capture() -> list[str]:
    """Uji mode `--capture` lawan probe yang menyerahkan JSON.

    Yang dipertaruhkan di sini bukan klasifikasi melainkan **kejujuran berkas
    hasil**: `security.yml` memakai ketiadaan berkas untuk memutuskan "tak bisa
    diperiksa" versus "bersih". Kalau probe yang menggantung menulis berkas
    kosong, penjaga di hilir akan mem-parse-nya, gagal, dan melaporkan
    `gagal mem-parse laporan JSON` — merah yang menyesatkan, persis cacat yang
    mode ini ada untuk menutupnya.
    """
    import tempfile

    failures: list[str] = []

    def probe(script: str, tmp: str, fail_closed: bool = False) -> int:
        args = argparse.Namespace(
            name="probe-uji", attempts=2, base_delay=0.01, timeout=10,
            capture=os.path.join(tmp, "report.json"), hang_is_red=fail_closed,
        )
        return capture(["python3", "-c", script], args)

    # 1. Laporan sungguhan ditulis apa adanya, walau exit code non-nol.
    #    (`cargo audit` keluar non-nol justru saat ia menemukan advisory.)
    with tempfile.TemporaryDirectory() as tmp:
        rc = probe("import sys; sys.stdout.write('{\"vulnerabilities\":{\"list\":[]}}'); sys.exit(1)", tmp)
        path = os.path.join(tmp, "report.json")
        got = open(path, encoding="utf-8").read() if os.path.exists(path) else None
        if rc != 0:
            failures.append(f"capture: rc={rc}, mau 0 saat laporan ada")
        if got != '{"vulnerabilities":{"list":[]}}':
            failures.append(f"capture: isi laporan salah: {got!r}")
        else:
            print("  ok  capture: laporan ditulis walau exit code non-nol")

    # 2. Tak pernah menjawab → hijau, dan berkas TIDAK ditulis (biar hilir
    #    melewatinya dengan alasan jujur alih-alih mem-parse berkas kosong).
    with tempfile.TemporaryDirectory() as tmp:
        rc = probe("import sys; sys.stderr.write('error: failed to update advisory database: "
                   "error sending request: tcp connect error'); sys.exit(1)", tmp)
        path = os.path.join(tmp, "report.json")
        if rc != 0:
            failures.append(f"capture: rc={rc}, mau 0 saat jaringan gagal (kebijakan hijau)")
        if os.path.exists(path):
            failures.append("capture: berkas ditulis padahal tak ada laporan — "
                            "hilir akan mem-parse kosong dan merah menyesatkan")
        else:
            print("  ok  capture: jaringan gagal = hijau tanpa berkas")

    # 2b. Kontrol kompensasi: di mode fail-closed (job `schedule`) jaringan yang
    #     tak pernah menjawab MERAH — kalau tidak, kebijakan "tak ada respons =
    #     hijau" bisa membisukan pemeriksaan tanpa batas waktu.
    with tempfile.TemporaryDirectory() as tmp:
        rc = probe("import sys; sys.stderr.write('error: failed to update advisory database: "
                   "error sending request: tcp connect error'); sys.exit(1)", tmp,
                   fail_closed=True)
        if rc != 1:
            failures.append(f"capture: rc={rc}, mau 1 saat jaringan gagal di mode fail-closed")
        elif os.path.exists(os.path.join(tmp, "report.json")):
            failures.append("capture: berkas ditulis di mode fail-closed")
        else:
            print("  ok  capture: mode fail-closed — jaringan gagal = merah")

    # 3. Gagal BUKAN karena jaringan → merah, jangan ditelan.
    with tempfile.TemporaryDirectory() as tmp:
        rc = probe("import sys; sys.stderr.write('error: invalid config: unable to parse manifest'); "
                   "sys.exit(2)", tmp)
        if rc != 1:
            failures.append(f"capture: rc={rc}, mau 1 saat gagal non-jaringan")
        else:
            print("  ok  capture: gagal non-jaringan tetap merah")

    failures += selftest_findings()

    return failures


def selftest_findings() -> list[str]:
    """Buktikan ekstraksi temuan berfungsi di bentuk keluaran yang NYATA.

    Versi sebelumnya lulus uji yang menaruh blok advisory dan baris `error:`
    berdekatan. `run_once` menangkap stdout dan stderr terpisah lalu
    menyambungnya, jadi di keluaran sungguhan `error:` berada di ujung, ratusan
    baris dari bloknya — dan jendela ±baris di sekitar anchor tak pernah
    mencapainya. Ekstraksinya menghasilkan NOL baris temuan, dan tak ada uji
    yang gagal. Uji di bawah menaruh keduanya persis seperti `run_once`
    melihatnya: blok advisory (stdout) di depan, ringkasan (stderr) di ekor.
    """
    failures: list[str] = []
    # Dua blok; hanya yang pertama adalah temuan sungguhan. Yang kedua meniru
    # advisory informasional `cargo audit` (selalu ber-`Warning:`, tanpa
    # `Severity:`) yang tak boleh membanjiri ringkasan.
    stream = (
        "Crate:     rustls\n"
        "Version:   0.23.43\n"
        "Title:     TLS 1.3 handshake messages incorrectly accepted\n"
        "Date:      2026-09-14\n"
        "ID:        RUSTSEC-2026-0285\n"
        "URL:       https://rustsec.org/advisories/RUSTSEC-2026-0285\n"
        "Severity:  5.3 (medium)\n"
        "Solution:  Upgrade to >=0.23.45\n"
        "Dependency tree:\n"
        "rustls 0.23.43\n"
        "└── tokio-rustls 0.26.4\n"
        "Crate:     validit\n"
        "Version:   0.2.5\n"
        "Warning:   yanked\n"
        "Dependency tree:\n"
        "validit 0.2.5\n"
        "└── openraft 0.9.25\n"
        "\n"
        # stdout berakhir; stderr-nya menyusul di ujung, jauh dari bloknya:
        "error: 1 vulnerability found!\n"
        "warning: 9 allowed warnings found\n"
    )
    got = extract_findings(strip_ansi(stream))
    joined = "\n".join(got)
    for want in (
        "Crate:     rustls",
        "Version:   0.23.43",
        "RUSTSEC-2026-0285",
        "Severity:  5.3 (medium)",
        "Solution:  Upgrade to >=0.23.45",
    ):
        if want not in joined:
            failures.append(f"findings: '{want}' tak tersorot dari keluaran nyata")
    if "Warning:" in joined:
        failures.append("findings: blok peringatan ikut terbawa ke ringkasan")
    if "tokio-rustls" in joined:
        failures.append("findings: ekor pohon dependensi bocor ke ringkasan")
    if not failures:
        print("  ok  findings: blok temuan tersorot lintas-stream")

    # Ekstraksi harus tetap jujur saat tak ada temuan sama sekali.
    if extract_findings("error: 1 vulnerability found!\n"):
        failures.append("findings: mengarang temuan dari keluaran tanpa blok")

    return failures


def run_once(cmd: list[str], timeout: int) -> tuple[int, str, str, bool]:
    """Jalankan sekali. Kembalikan (returncode, stdout, stderr, apakah timeout).

    stdout dipisahkan dari stderr karena mode `--capture` menulis stdout apa
    adanya sebagai laporan terstruktur; mencampurnya dengan progres/log ke
    stderr akan menghasilkan "JSON" yang tak bisa diparse.

    `subprocess.run` dengan `timeout` MEMBUNUH anaknya dan mengembalikan
    `TimeoutExpired`, jadi satu percobaan yang menggantung tidak menahan job
    sampai `timeout-minutes`-nya habis.
    """
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            # Jangan raise; kita yang memutuskan arti returncode.
            check=False,
        )
        return proc.returncode, proc.stdout or "", proc.stderr or "", False
    except subprocess.TimeoutExpired as e:
        def decode(stream):
            if not stream:
                return ""
            return stream.decode() if isinstance(stream, bytes) else stream

        return 124, decode(e.stdout), decode(e.stderr), True
    except FileNotFoundError as e:
        # Alatnya tidak ada di PATH: itu kesalahan setup, bukan temuan dan bukan
        # jaringan. Jangan dianggap hijau.
        return 127, "", f"error: command not found: {e}", False


def capture(cmd: list[str], args) -> int:
    """Jalankan pemeriksaan yang KELUARANNYA adalah muatannya, bukan exit code.

    Dipakai oleh probe seperti "tidak ada ignore basi" (`security.yml`), yang
    menjalankan `cargo audit --json` lalu menyerahkan JSON-nya ke skrip lain.
    Di sana exit code bukan sinyal: `cargo audit` keluar non-nol justru KETIKA
    ia menemukan advisory, dan himpunan advisory itulah yang dibutuhkan.

    Kriteria berhasilnya karena itu "stdout memuat sesuatu", dan kegagalan
    jaringan diperlakukan persis seperti di jalur utama: diulang, lalu dihitung
    hijau bila tak pernah menjawab.

    Berkas ditulis HANYA bila ada laporan sungguhan. Pemanggil membedakan tiga
    keadaan — laporan ada, laporan tak ada (jaringan), laporan ada tapi rusak
    (temuan nyata) — dan hanya yang terakhir yang boleh merah. Itu penting:
    sebelum ini probe menjalankan `cargo audit` telanjang, satu koneksi putus
    menghasilkan stdout kosong, dan penjaga di hilir melaporkannya sebagai
    "gagal mem-parse laporan JSON" — job merah yang tak ada hubungannya dengan
    ignore basi. Persis kelas yang seharusnya ditutup pembungkus ini.
    """
    for attempt in range(1, args.attempts + 1):
        started = time.monotonic()
        rc, stdout, stderr, timed_out = run_once(cmd, args.timeout)
        elapsed = time.monotonic() - started
        combined = stdout + stderr
        print(f"[{args.name}] percobaan {attempt}/{args.attempts}: rc={rc} "
              f"{'timeout' if timed_out else 'selesai'} dalam {elapsed:.1f}s", flush=True)

        if stdout.strip():
            try:
                with open(args.capture, "w", encoding="utf-8") as fh:
                    fh.write(stdout)
            except OSError as exc:
                print(f"error: tak bisa menulis {args.capture}: {exc}", file=sys.stderr)
                return 1
            print(f"[{args.name}] laporan ditulis ke {args.capture} "
                  f"({len(stdout)} byte)", flush=True)
            _write_summary(args.name, "GREEN",
                           f"laporan diperoleh pada percobaan {attempt}.")
            return 0

        # stdout kosong. Kalau ini kegagalan jaringan (atau timeout), ulangi;
        # kalau bukan, itu kesalahan nyata — jangan ditelan.
        verdict, reason = classify(combined, rc, timed_out)
        if verdict == "RED" and not timed_out:
            print(f"[{args.name}] gagal dan bukan karena jaringan: {reason}",
                  file=sys.stderr)
            _write_summary(args.name, "RED", f"gagal tanpa laporan: {reason}")
            return 1

        if attempt < args.attempts:
            delay = args.base_delay * (2 ** (attempt - 1))
            print(f"[{args.name}] tak ada laporan; mencoba ulang dalam {delay:.0f}s...",
                  flush=True)
            time.sleep(delay)

    # Tak pernah menjawab. Di mode fail-closed (job `schedule`) itu MERAH; selain
    # itu hijau sesuai kebijakan, TAPI berkasnya sengaja tidak ditulis supaya
    # hilir bisa membedakan "tak bisa diperiksa" dari "bersih".
    if hang_is_red(args):
        print(f"[{args.name}] tidak ada laporan setelah {args.attempts} percobaan "
              "— MERAH (mode fail-closed).", file=sys.stderr, flush=True)
        _write_summary(
            args.name, "RED",
            f"Probe tidak pernah mendapat jawaban setelah {args.attempts} percobaan "
            "dan dijalankan dalam mode **fail-closed** (job terjadwal): pemeriksaan "
            "yang tak bisa dijalankan tidak boleh tampak lulus.",
        )
        return 1
    print(f"[{args.name}] tidak ada laporan setelah {args.attempts} percobaan "
          f"— dihitung hijau (jaringan), berkas tidak ditulis.", flush=True)
    _annotate_hang(args.name)
    _write_summary(
        args.name, "HANG",
        f"Probe tidak pernah mendapat jawaban setelah {args.attempts} percobaan. "
        "Dihitung **hijau** sesuai kebijakan, dan berkas laporan sengaja TIDAK "
        "ditulis sehingga pemeriksaan di hilir melewatinya dengan alasan yang "
        "jujur — bukan seolah-olah ignore-nya sudah diverifikasi.",
    )
    return 0


def hang_is_red(args) -> bool:
    """Mode fail-closed: pemeriksaan yang tak pernah menjawab dihitung MERAH.

    Kebijakan pemilik repo: tak ada respons = hijau. Kontrol kompensasinya
    (audit PR #930): job `schedule` harian menjalankan mode ini, sehingga
    pemeriksaan yang terus-menerus "tak menjawab" tidak bisa membisukan temuan
    tanpa batas waktu — paling lama satu hari, lalu tampil merah di `main`.
    Diaktifkan lewat `--hang-is-red` atau `DEP_CHECK_HANG_IS_RED=1` (diset
    workflow untuk event `schedule`).
    """
    return bool(getattr(args, "hang_is_red", False)) or (
        os.environ.get("DEP_CHECK_HANG_IS_RED", "").strip() == "1"
    )


def _annotate_hang(name: str) -> None:
    """Anotasi workflow yang TERLIHAT untuk hijau-karena-jaringan.

    Ringkasan step hanya terbaca bila seseorang membukanya; anotasi `warning`
    muncul di halaman run dan di PR, jadi "hijau" dan "tak pernah berjalan"
    tidak tampak identik.
    """
    print(
        f"::warning title={name}: tidak pernah mendapat jawaban::"
        "Dihitung hijau sesuai kebijakan (jaringan), tetapi TIDAK ada temuan "
        "yang terverifikasi bebas. Jalankan ulang bila butuh kepastian; job "
        "terjadwal harian berjalan fail-closed."
    )


def _write_summary(name: str, verdict: str, detail: str) -> None:
    icons = {"GREEN": "✅ OK", "RED": "❌ FAIL", "HANG": "✅ OK (jaringan)"}
    body = f"### {name}: {icons[verdict]}\n\n{detail}\n"
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if path:
        try:
            with open(path, "a", encoding="utf-8") as fh:
                fh.write(body + "\n")
        except OSError:
            pass
    print(body)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--name", required=False, help="nama pemeriksaan untuk ringkasan")
    ap.add_argument("--attempts", type=int, default=4)
    ap.add_argument("--base-delay", type=float, default=5.0)
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("--allow-failure", action="store_true")
    ap.add_argument(
        "--hang-is-red",
        action="store_true",
        help="mode fail-closed: pemeriksaan yang tak pernah menjawab = MERAH "
             "(juga via DEP_CHECK_HANG_IS_RED=1; dipakai job `schedule`)",
    )
    ap.add_argument(
        "--capture",
        metavar="PATH",
        help="tulis stdout percobaan yang berhasil ke PATH (untuk probe yang "
             "muatannya adalah keluaran, bukan exit code)",
    )
    ap.add_argument(
        "--selftest",
        action="store_true",
        help="uji klasifikasi lawan kasus yang diketahui, lalu keluar",
    )
    ap.add_argument("cmd", nargs=argparse.REMAINDER)
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    if not args.name:
        print("error: --name wajib (kecuali --selftest)", file=sys.stderr)
        return 2

    cmd = args.cmd
    if cmd and cmd[0] == "--":
        cmd = cmd[1:]
    if not cmd:
        print("error: no command given (use -- <cmd>)", file=sys.stderr)
        return 2

    if args.capture:
        return capture(cmd, args)

    summary_lines: list[str] = []
    verdict, reason, output = "RED", "tidak pernah dijalankan", ""

    for attempt in range(1, args.attempts + 1):
        started = time.monotonic()
        rc, stdout, stderr, timed_out = run_once(cmd, args.timeout)
        output = strip_ansi(stdout + stderr)
        elapsed = time.monotonic() - started
        verdict, reason = classify(output, rc, timed_out)
        tag = f"percobaan {attempt}/{args.attempts}"
        print(f"[{args.name}] {tag}: {verdict} ({reason}) dalam {elapsed:.1f}s", flush=True)

        if verdict == "GREEN":
            break

        if verdict == "RED":
            # Sinyal nyata — jangan retry. Mengulang temuan sungguhan hanya
            # menunda jawabannya.
            break

        # HANG: coba lagi dengan backoff, selama masih ada percobaan.
        if attempt < args.attempts:
            delay = args.base_delay * (2 ** (attempt - 1))
            print(f"[{args.name}] mencoba ulang dalam {delay:.0f}s...", flush=True)
            time.sleep(delay)

    # Fail-closed (job `schedule`): tak pernah menjawab = MERAH, bukan hijau.
    if verdict == "HANG" and hang_is_red(args):
        verdict = "RED"
        reason = f"tak pernah menjawab setelah {args.attempts} percobaan ({reason}); mode fail-closed"
    elif verdict == "HANG":
        _annotate_hang(args.name)

    # ── Ringkasan ────────────────────────────────────────────────────────────
    icons = {"GREEN": "✅ OK", "RED": "❌ FAIL", "HANG": "✅ OK (jaringan)"}
    if verdict == "HANG":
        detail = (
            f"Pemeriksaan **tidak pernah mendapat jawaban** setelah {args.attempts} "
            f"percobaan ({reason}). Dihitung **hijau** sesuai kebijakan: pemeriksaan "
            "yang tak bisa dijalankan karena jaringan tidak membuktikan apa pun tentang "
            "dependency proyek. **Tidak ada temuan yang terverifikasi bebas di sini** — "
            "jalankan ulang saat jaringan sehat bila butuh kepastian."
        )
    elif verdict == "GREEN":
        detail = f"Alat berjalan ({reason}). Tidak ada temuan."
    else:
        detail = f"Alat berjalan dan melaporkan masalah ({reason})."

    summary_lines.append(f"### {args.name}: {icons[verdict]}")
    summary_lines.append("")
    summary_lines.append(detail)
    summary_lines.append("")

    if output.strip():
        # Ekor buta 100 baris menyembunyikan justru bagian yang dibutuhkan saat
        # RED. `cargo audit` mencetak judul advisory lebih dulu, lalu tabel
        # dependency-tree yang panjang (ratusan baris), jadi memotong dari ekor
        # membuang header-nya dan menyisakan pohonnya: pembaca melihat "1
        # vulnerability found" tanpa nama crate, ID, atau versi yang
        # diperbaiki. Kejadian nyata 2026-09-25.
        #
        # Jadi bila ada sinyal temuan, sorot blok yang menjawabnya DI ATAS ekor
        # biasa. Pemilihannya ada di `extract_findings` (lihat docstring-nya
        # untuk alasan ia memarse per-blok alih-alih mengangkur ke `error:`).
        signal_lines: list[str] = []
        if verdict == "RED":
            signal_lines = extract_findings(output)

        summary_lines.append("<details><summary>Keluaran (ekor 100 baris)</summary>")
        summary_lines.append("")
        if signal_lines:
            summary_lines.append("Baris sinyal (diekstrak dari seluruh keluaran):")
            summary_lines.append("")
            summary_lines.append("```")
            summary_lines.extend(signal_lines[:40])
            summary_lines.append("```")
            summary_lines.append("")
            summary_lines.append("Ekornya:")
            summary_lines.append("")
        summary_lines.append("```")
        summary_lines.extend(output.strip().splitlines()[-100:])
        summary_lines.append("```")
        summary_lines.append("</details>")
        summary_lines.append("")

    body = "\n".join(summary_lines)
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        try:
            with open(summary_path, "a", encoding="utf-8") as fh:
                fh.write(body + "\n")
        except OSError:
            pass
    print(body)

    if verdict == "RED" and not args.allow_failure:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())