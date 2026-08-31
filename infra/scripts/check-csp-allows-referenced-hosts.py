#!/usr/bin/env python3
"""Setiap host eksternal yang dirujuk kode harus diizinkan CSP yang menyajikannya.

Sebuah `<img>` ke host yang tidak ada di `img-src` ditolak browser **tanpa satu
pun pesan yang terlihat pengguna**: tidak ada galat di layar, tidak ada
permintaan gagal di log server, hanya gambar yang tidak muncul. Itu kelas
kegagalan yang sama dengan NetworkPolicy yang menjatuhkan paket diam-diam, dan
sudah berulang di repo ini.

Penjaga ini menurunkan daftar host dari KODE — konstanta `MEDIA_MYSIMKARI` dan
kerabatnya — lalu memastikan tiap host itu muncul di setiap CSP yang menyajikan
frontend. Daftarnya diturunkan, bukan ditulis tangan, karena daftar tulis-tangan
tidak ikut tumbuh saat host baru ditambahkan.

Berkas SUMBER-nya pun kini dipindai, bukan didaftar. Versi pertama menyebut
`lib/ui/src/components/foto_pegawai.rs` secara harfiah, dan modul itu pindah ke
`lib-core` begitu `layanan-integrasi` juga membutuhkannya — memindahkan satu
berkas mematikan penjaganya. Itu cacat yang sama, satu tingkat lebih dalam:
cakupan yang benar dengan daftar tulis-tangan yang tak ikut bergerak.

Hanya pustaka standar: image runner ARC tidak punya PyYAML (exit 127, tiga kali
di job yang sama).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# Sumber host: setiap `pub const X: &str = "https://..."` di pustaka bersama
# dan di kedua frontend — di mana pun berkasnya berada.
HOST_SOURCE_GLOBS = ("lib/*/src/**/*.rs", "antarmuka/*/src/**/*.rs")

# Setiap berkas yang menyetel CSP untuk sebuah frontend.
CSP_FILES = [
    ROOT / "infra/helm/simpel/values.yaml",
    ROOT / "antarmuka/portal/nginx.conf",
    ROOT / "antarmuka/perlengkapan/nginx.conf",
]

# `re.DOTALL` lewat `\s*`: rustfmt memecah konstanta panjang setelah `=`,
# dan versi pertama penjaga ini hanya mencocokkan satu baris — sehingga
# menjalankan `cargo fmt` membuatnya menemukan NOL host, yang tanpa penjagaan
# di bawah akan lulus selamanya.
CONST_RE = re.compile(r'pub const [A-Z_]+: &str =\s*"(https://[^/"]+)')


def hosts() -> set[str]:
    found: set[str] = set()
    scanned = 0
    for glob in HOST_SOURCE_GLOBS:
        for path in ROOT.glob(glob):
            if "target" in path.parts:
                continue
            scanned += 1
            found.update(CONST_RE.findall(path.read_text(encoding="utf-8", errors="replace")))
    if not scanned:
        print(
            "Nol berkas terpindai — glob sumbernya tidak cocok apa pun, jadi "
            "penjaga ini tidak menjaga apa-apa.",
            file=sys.stderr,
        )
        sys.exit(2)
    if not found:
        print(
            "Nol host terturunkan — pola konstanta berubah, dan penjaga yang "
            "tidak menemukan apa pun akan lulus selamanya.",
            file=sys.stderr,
        )
        sys.exit(1)
    return found


def main() -> int:
    wanted = hosts()
    problems = []
    for path in CSP_FILES:
        if not path.exists():
            problems.append(f"{path.relative_to(ROOT)}: tidak ada")
            continue
        text = path.read_text(encoding="utf-8")
        # `https:` terbuka mengizinkan semuanya; itu dianggap memenuhi, tapi
        # host eksplisit lebih disukai dan tidak dipaksakan di sini.
        #
        # Kelasnya `[^;]` dan BUKAN `[^;"']`: direktif CSP dimulai dengan
        # `'self'`, jadi kelas yang mengecualikan kutip berhenti di karakter
        # pertama dan `https:` yang memang ada tak pernah terlihat. Versi
        # pertama penjaga ini melakukan persis itu dan menuduh dua berkas yang
        # sebenarnya sudah benar.
        if re.search(r"img-src[^;]*\bhttps:(?!//)", text):
            continue
        for host in sorted(wanted):
            if host not in text:
                problems.append(
                    f"{path.relative_to(ROOT)}: {host} dirujuk kode tetapi "
                    f"tidak ada di img-src"
                )
    if problems:
        print("CSP tidak mengizinkan host yang dirujuk kode:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        print(
            "\nBrowser menolak gambar dari host yang tidak diizinkan tanpa "
            "pesan apa pun. Tambahkan hostnya, jangan buka `https:` lebar.",
            file=sys.stderr,
        )
        return 1
    print(f"OK: {len(wanted)} host media diizinkan di {len(CSP_FILES)} berkas CSP.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
