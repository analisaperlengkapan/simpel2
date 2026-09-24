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
    r"timed? ?out",  # "timed out" / "timeout"
    r"operation was canceled",  # reqwest cancellation (koneksi putus)
    r"incomplete message",
    r"unexpected eof",
    r"connection closed before message completed",
    r"tls handshake",
    r"certificate verify failed",
    r"schannel",
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
    r"unable to access '",
    r"git protocol error",
    r"remote end hung up",
    r"early eof",
    r"the remote end hung up unexpectedly",
    # Docker (composer audit menarik image PHP).
    r"error response from daemon",
    r"manifest unknown",  # registry tak menjawab / tag hilang sementara
    r"failed to resolve source metadata",
    r"docker: error",
    # Galat unduh yang eksplisit milik alat-alat ini.
    r"could not download",
    r"failed to download",
    r"error downloading",
    r"download failed",
    r"failed to update advisory database",
    r"failed to fetch advisory",
    r"error updating index",
    r"failed to get (crate|index|registry)",
    r"crates\.io",
    r"spurious network error",
    r"the lock file needs to be updated but --locked was passed",  # crate baru, index tak terbaca
]

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


def classify(output: str, returncode: int, timed_out: bool) -> tuple[str, str]:
    """Tentukan GREEN / RED / HANG beserta alasannya.

    Urutannya penting: timeout dan kegagalan jaringan diperiksa setelah
    memastikan tak ada sinyal keras yang muncul. Kalau sebuah temuan sudah
    tercetak di output, proses yang kemudian mati karena jaringan tetap RED —
    temuan itu nyata dan kita sudah melihatnya.
    """
    if DECISIVE_RE.search(output):
        return "RED", "output memuat sinyal temuan/konfigurasi — bukan kegagalan jaringan"

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
    ("jaringan: connect refused",
     "error: failed to update advisory database: error sending request: tcp connect error", 1, False, "HANG"),
    ("jaringan: DNS",
     "fatal: unable to access 'https://github.com/rustsec/advisory-db/': "
     "Could not resolve host: github.com", 128, False, "HANG"),
    ("jaringan: TLS", "error: tls handshake failure: certificate verify failed", 1, False, "HANG"),
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

    if failures:
        print("\nSELF-TEST GAGAL:", file=sys.stderr)
        for f in failures:
            print("  " + f, file=sys.stderr)
        return 1
    print(f"\nself-test lulus ({len(SELF_TESTS)} kasus)")
    return 0


def run_once(cmd: list[str], timeout: int) -> tuple[int, str, bool]:
    """Jalankan sekali. Kembalikan (returncode, output gabungan, apakah timeout).

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
        return proc.returncode, (proc.stdout or "") + (proc.stderr or ""), False
    except subprocess.TimeoutExpired as e:
        partial = ""
        for stream in (e.stdout, e.stderr):
            if stream:
                partial += stream.decode() if isinstance(stream, bytes) else stream
        return 124, partial, True
    except FileNotFoundError as e:
        # Alatnya tidak ada di PATH: itu kesalahan setup, bukan temuan dan bukan
        # jaringan. Jangan dianggap hijau.
        return 127, f"error: command not found: {e}", False


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--name", required=False, help="nama pemeriksaan untuk ringkasan")
    ap.add_argument("--attempts", type=int, default=4)
    ap.add_argument("--base-delay", type=float, default=5.0)
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("--allow-failure", action="store_true")
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

    summary_lines: list[str] = []
    verdict, reason, output = "RED", "tidak pernah dijalankan", ""

    for attempt in range(1, args.attempts + 1):
        started = time.monotonic()
        rc, output, timed_out = run_once(cmd, args.timeout)
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
        summary_lines.append("<details><summary>Keluaran (ekor 100 baris)</summary>")
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