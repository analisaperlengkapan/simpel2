#!/usr/bin/env python3
"""Siapkan data awal ukuran pakaian dinas dari berkas satker untuk diimpor ke SIMPEL.

Berkas satker (xlsx, ditulis tangan, puluhan ribu baris) tidak boleh dimuat apa adanya: NIP-nya
rusak (notasi ilmiah `1.99E+17` yang memotong tiga digit terakhir, "NIP + nomor lama", spasi),
ada NIP yang dipakai dua orang, dan ukurannya bercampur huruf besar-kecil dan "32.0". Alat ini
membersihkannya DI LUAR aplikasi dan memisahkan:

  * baris yang JELAS  -> lot JSON yang dikirim ke `POST /pakaian-dinas/impor-profil`;
  * baris yang masih DIRAGUKAN -> `karantina.csv` (NIP rusak atau ambigu, NIP ganda yang
    datanya berbeda, NIP dipakai orang lain, digit jenis kelamin tak cocok). Yang diragukan TIDAK
    dimuat: diperiksa ke MySIMKARI nanti, lalu alat ini dijalankan ulang dengan
    `--rujukan-pegawai` (impor aman diulang: ia hanya menimpa hasil impor sebelumnya).

DATA NYATA TIDAK BOLEH MASUK GIT. Karena itu `--keluaran` ditolak bila berada di dalam pohon repo
ini, berkas keluaran dibuat dengan izin 0600, dan laporan yang dicetak hanya berisi JUMLAH.
`karantina.csv` dan lot memuat NIP dan nama: simpan di luar repo, hapus (shred) setelah selesai,
dan jangan ditempel ke tiket atau obrolan.

Hanya pustaka standar (runner ARC tidak punya PyYAML).

Pemakaian:
  siapkan-impor-pakaian-dinas.py --kejagung A.xlsx --kejati B.xlsx --keluaran /aman/di/luar/repo
  siapkan-impor-pakaian-dinas.py ... --rujukan-pegawai mysimkari.csv      # nip[,nama] per baris
  siapkan-impor-pakaian-dinas.py ... --kirim https://host/api/v1/perlengkapan   # SIMPEL_TOKEN di env
  siapkan-impor-pakaian-dinas.py ... --kirim URL --muat                    # BARU benar-benar menulis
  siapkan-impor-pakaian-dinas.py --self-test
"""

from __future__ import annotations

import argparse
import collections
import csv
import datetime
import json
import os
import re
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# ───────────────────────── kolom berkas -> kolom pakaian aplikasi ─────────────────────────
# (jenis, spesifikasi) dinamai seperti di aplikasi. SATU kolom berkas boleh mengisi beberapa kolom
# aplikasi: berkas Kejati hanya punya satu kolom "baju" dan satu "celana" untuk PDH dan PDL, tetapi
# sepatu dipisah (sepatu pdl / sepatu pdh). ASUMSI yang perlu dikonfirmasi pemilik data: ukuran baju
# dan celana yang sama berlaku untuk PDH dan PDL. Mengubahnya cukup di tabel ini.
PETA = {
    "kejagung": {
        "sepatu pdh": [("PDH", "Sepatu Dinas")],
    },
    "kejati": {
        "kaos": [("PDL", "Kaos")],
        "baju": [("PDH", "Pakaian Dinas"), ("PDL", "Baju")],
        "celana": [("PDH", "Celana"), ("PDL", "Celana")],
        "topi": [("PDL", "Topi")],
        "sepatu pdl": [("PDL", "Sepatu")],
        "sepatu pdh": [("PDH", "Sepatu Dinas")],
        # Kolom "nama" yang KEDUA di berkas Kejati (setelah "sepatu pdh") adalah nama dada.
        "nama#2": [("PDL", "Nama Dada")],
    },
}

KATEGORI_JELAS = ("OK18", "LEGACY_APPENDED")


# ───────────────────────── pembaca xlsx (stdlib) ─────────────────────────
NS = {
    "m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
}


def _kolom(ref: str) -> int:
    n = 0
    for ch in re.match(r"[A-Z]+", ref).group(0):
        n = n * 26 + ord(ch) - 64
    return n - 1


class Buku:
    """Pembaca xlsx minimal. Berkas diperlakukan sebagai DATA tak tepercaya (tak ada makro, tak ada rumus)."""

    def __init__(self, path: str):
        self.z = zipfile.ZipFile(path)
        wb = ET.fromstring(self.z.read("xl/workbook.xml"))
        rels = ET.fromstring(self.z.read("xl/_rels/workbook.xml.rels"))
        rid = {r.get("Id"): r.get("Target") for r in rels}
        self.lembar = []
        for s in wb.find("m:sheets", NS):
            t = rid[s.get("{%s}id" % NS["r"])].lstrip("/")
            self.lembar.append((s.get("name"), t if t.startswith("xl/") else "xl/" + t))
        self.bersama: list[str] = []
        if "xl/sharedStrings.xml" in self.z.namelist():
            for _, el in ET.iterparse(self.z.open("xl/sharedStrings.xml")):
                if el.tag.endswith("}si"):
                    self.bersama.append("".join(t.text or "" for t in el.iter("{%s}t" % NS["m"])))
                    el.clear()

    def baris(self, anggota: str):
        """(nomor_baris, {indeks_kolom: nilai}) berurutan."""
        with self.z.open(anggota) as fh:
            for _, el in ET.iterparse(fh):
                if not el.tag.endswith("}row"):
                    continue
                nomor = int(el.get("r") or 0)
                nilai: dict[int, str] = {}
                for c in el:
                    v = c.find("m:v", NS)
                    t = c.get("t")
                    if t == "inlineStr":
                        nilai[_kolom(c.get("r"))] = "".join(x.text or "" for x in c.iter("{%s}t" % NS["m"]))
                    elif v is None or v.text is None:
                        continue
                    elif t == "s":
                        nilai[_kolom(c.get("r"))] = self.bersama[int(v.text)]
                    else:
                        nilai[_kolom(c.get("r"))] = v.text
                el.clear()
                if nilai:
                    yield nomor, nilai


# ───────────────────────── NIP ─────────────────────────
def digit(s: str) -> str:
    return re.sub(r"\D", "", s)


def nip_valid(d: str) -> bool:
    """Struktur NIP 18 digit: YYYYMMDD lahir, lalu TMT YYYYMM (ASN) atau YYYY+`21` (PPPK), 1|2 jenis kelamin, 3 digit urut."""
    if len(d) != 18 or not d.isdigit():
        return False
    try:
        datetime.date(int(d[0:4]), int(d[4:6]), int(d[6:8]))
    except ValueError:
        return False
    if not 1940 <= int(d[0:4]) <= 2010:
        return False
    tahun, kode = int(d[8:12]), d[12:14]
    if kode == "21":
        return 2019 <= tahun <= 2030
    return 1960 <= tahun <= 2030 and 1 <= int(kode) <= 12


def klasifikasi_nip(mentah: str) -> tuple[str, str | None]:
    """(kategori, nip_18_digit_atau_potongan). Hanya OK18 dan LEGACY_APPENDED yang jelas."""
    s = mentah.strip()
    d = digit(s)
    kelompok = re.findall(r"\d+", s)
    if d == "":
        return "KOSONG", None
    if re.fullmatch(r"\d\.\d+E\+\d+", s.replace(",", ".").upper()):
        return "NOTASI_ILMIAH", re.sub(r"\D", "", s.upper().split("E")[0])
    if len(d) == 18:
        return ("OK18" if nip_valid(d) else "INVALID18"), d
    # "<18 digit> <nomor lama>" atau "8 6 1 3 <nomor lama>" (NIP dipecah spasi)
    if len(kelompok[0]) == 18 and len(kelompok) > 1 and 5 <= len(d[18:]) <= 10:
        return ("LEGACY_APPENDED" if nip_valid(kelompok[0]) else "INVALID18"), kelompok[0]
    if [len(g) for g in kelompok[:4]] == [8, 6, 1, 3] and len(kelompok) > 4:
        j = "".join(kelompok[:4])
        return ("LEGACY_APPENDED" if nip_valid(j) else "INVALID18"), j
    if len(d) in (16, 17, 19, 20):
        return f"PANJANG_{len(d)}", d
    if len(d) < 16:
        return "TERLALU_PENDEK", d
    return "PANJANG_LAIN", d


def digit_jk_beda(nip: str, jk: str | None) -> bool:
    """Digit ke-15 NIP: 1 laki-laki, 2 perempuan. Beda dengan kolom gender = salah satunya keliru."""
    return jk in ("L", "P") and len(nip) == 18 and nip[14] in "12" and ((nip[14] == "1") != (jk == "L"))


GELAR = re.compile(r"\b(DR|PROF|IR|H|HJ|DRS|DRA|DRG|DRH)\b\.?", re.I)


def inti_nama(nama: str) -> str:
    """Nama dibandingkan tanpa gelar di belakang koma, gelar di depan, tanda baca, dan huruf besar-kecil."""
    s = GELAR.sub(" ", nama.split(",")[0].upper())
    return re.sub(r"[^A-Z0-9]", "", s)


# ───────────────────────── nilai sel ─────────────────────────
def norm_ukuran(mentah: str) -> str | None:
    s = mentah.strip()
    if s in ("", "-", "0", "0.0"):
        return None
    if re.fullmatch(r"\d+\.0+", s):
        s = s.split(".")[0]
    return s.upper() if re.fullmatch(r"[A-Za-z0-9]+", s) else s


def norm_jk(mentah: str) -> str | None:
    s = mentah.strip().upper()[:1]
    return s if s in ("L", "P") else None


def norm_hijab(mentah: str) -> bool | None:
    s = mentah.strip().lower()
    if s in ("ya", "y"):
        return True
    if s in ("tidak", "t"):
        return False
    return None


# ───────────────────────── membaca satu berkas ─────────────────────────
def baca_berkas(path: str, jenis: str) -> list[dict]:
    """Daftar rekam {berkas, baris, nip_asli, nama, jk, hijab, ukuran{(jenis,spek): nilai}} dari lembar pertama."""
    peta = PETA[jenis]
    buku = Buku(path)
    _, anggota = buku.lembar[0]
    judul: dict[int, str] = {}
    keluar: list[dict] = []
    for nomor, sel in buku.baris(anggota):
        if not judul:
            lihat: collections.Counter = collections.Counter()
            for i in sorted(sel):
                h = str(sel[i]).strip().lower()
                lihat[h] += 1
                judul[i] = h if lihat[h] == 1 else f"{h}#{lihat[h]}"
            wajib = {"nip", "nama"}
            if not wajib <= set(judul.values()):
                raise SystemExit(f"{path}: judul kolom tak memuat {sorted(wajib - set(judul.values()))}")
            continue
        per = {judul[i]: str(v) for i, v in sel.items() if i in judul}
        ukuran: dict[tuple[str, str], str] = {}
        for kolom, tujuan in peta.items():
            nilai = norm_ukuran(per.get(kolom, ""))
            if nilai is not None:
                for t in tujuan:
                    ukuran[t] = nilai
        keluar.append(
            {
                "berkas": jenis,
                "baris": nomor,
                "nip_asli": per.get("nip", "").strip(),
                "nama": " ".join(per.get("nama", "").split()),
                "jk": norm_jk(per.get("gender", "")),
                "hijab": norm_hijab(per.get("hijab", "")),
                "ukuran": ukuran,
            }
        )
    return keluar


# ───────────────────────── rujukan pegawai (mis. ekspor MySIMKARI) ─────────────────────────
def baca_rujukan(path: str) -> dict[str, str]:
    """{nip: inti_nama_atau_kosong} dari CSV/teks `nip[,nama]` per baris (tanpa judul wajib)."""
    hasil: dict[str, str] = {}
    with open(path, encoding="utf-8", errors="replace", newline="") as fh:
        for kolom in csv.reader(fh):
            if not kolom:
                continue
            nip = digit(kolom[0])
            if len(nip) == 18:
                hasil[nip] = inti_nama(kolom[1]) if len(kolom) > 1 else ""
    return hasil


def usulan_nip(kategori: str, d: str | None, nama: str, rujukan: dict[str, str]) -> list[str]:
    """NIP kandidat dari rujukan untuk NIP rusak, dengan nama yang sama. Tak pernah dimuat otomatis."""
    if not rujukan or not d or not nama:
        return []
    inti = inti_nama(nama)
    kolam = [n for n, nm in rujukan.items() if nm == inti]
    if kategori == "NOTASI_ILMIAH":
        hit = [c for c in kolam if c.startswith(d[:15])]
    elif kategori in ("PANJANG_16", "PANJANG_17"):
        hit = [c for c in kolam if _subseq(d, c) and c[:6] == d[:6]]
    elif kategori in ("PANJANG_19", "PANJANG_20"):
        hit = [c for c in kolam if _subseq(c, d) and c[:6] == d[:6]]
    elif kategori == "INVALID18":
        hit = [c for c in kolam if sum(a != b for a, b in zip(c, d)) <= 2]
    else:
        hit = kolam if len(kolam) == 1 else []
    return sorted({c for c in hit if nip_valid(c)})


def _subseq(pendek: str, panjang: str) -> bool:
    it = iter(panjang)
    return all(ch in it for ch in pendek)


# ───────────────────────── memilah ─────────────────────────
def siapkan(rekam: list[dict], rujukan: dict[str, str] | None = None, terima_digit_jk: bool = False,
            muat_pemulihan: bool = False):
    """(jelas, karantina, jumlah). `jelas` = baris siap kirim; `karantina` = baris + alasan."""
    jumlah: collections.Counter = collections.Counter()
    karantina: list[dict] = []
    calon: dict[str, list[dict]] = collections.defaultdict(list)

    def tahan(r: dict, alasan: str, usulan: list[str] | None = None):
        karantina.append({**r, "alasan": alasan, "usulan": ";".join(usulan or [])})
        jumlah[f"karantina:{alasan}"] += 1

    for r in rekam:
        jumlah[f"dibaca:{r['berkas']}"] += 1
        if not r["ukuran"]:
            jumlah["dilewati:tanpa_ukuran"] += 1
            continue
        kategori, d = klasifikasi_nip(r["nip_asli"])
        if kategori not in KATEGORI_JELAS:
            usul = usulan_nip(kategori, d, r["nama"], rujukan or {})
            if usul and len(usul) == 1 and muat_pemulihan:
                calon[usul[0]].append({**r, "nip": usul[0], "pulih": True})
                jumlah["pulih:dimuat"] += 1
            else:
                tahan(r, f"nip_{kategori.lower()}", usul)
            continue
        if digit_jk_beda(d, r["jk"]) and not terima_digit_jk:
            tahan(r, "digit_jk_tidak_cocok")
            continue
        if rujukan is not None:
            if d not in rujukan:
                tahan(r, "nip_tidak_ada_di_rujukan", usulan_nip("INVALID18", d, r["nama"], rujukan))
                continue
            if rujukan[d] and rujukan[d] != inti_nama(r["nama"]):
                tahan(r, "nama_beda_di_rujukan")
                continue
        calon[d].append({**r, "nip": d})

    jelas: list[dict] = []
    for nip, daftar in calon.items():
        nama_inti = {inti_nama(x["nama"]) for x in daftar}
        if len(nama_inti) > 1:
            for x in daftar:
                tahan(x, "nip_dipakai_orang_lain")
            continue
        gabung = dict(daftar[0])
        gabung["ukuran"] = dict(daftar[0]["ukuran"])
        galat = False
        for x in daftar[1:]:
            for k, v in x["ukuran"].items():
                if k in gabung["ukuran"] and gabung["ukuran"][k] != v:
                    galat = True
                gabung["ukuran"].setdefault(k, v)
            for kolom in ("jk", "hijab"):
                if x[kolom] is not None and gabung[kolom] is not None and x[kolom] != gabung[kolom]:
                    galat = True
                if gabung[kolom] is None:
                    gabung[kolom] = x[kolom]
        if galat:
            for x in daftar:
                tahan(x, "ganda_data_berbeda")
            continue
        if len(daftar) > 1:
            jumlah["digabung:ganda_sama_orang"] += len(daftar) - 1
        jelas.append(gabung)
    jumlah["jelas"] = len(jelas)
    jelas.sort(key=lambda x: (x["berkas"], x["baris"]))
    return jelas, karantina, jumlah


def ke_json(r: dict) -> dict:
    return {
        "rujukan": f"{r['berkas']}:{r['baris']}",
        "nip": r["nip"],
        "nama": r["nama"],
        "jenis_kelamin": r["jk"],
        "with_hijab": r["hijab"],
        "ukuran": [
            {"jenis": j, "spesifikasi": s, "ukuran": v} for (j, s), v in sorted(r["ukuran"].items())
        ],
    }


# ───────────────────────── keluaran (izin 0600, di luar repo) ─────────────────────────
def tolak_di_dalam_repo(keluaran: Path) -> None:
    try:
        keluaran.resolve().relative_to(REPO)
    except ValueError:
        return
    raise SystemExit(
        f"--keluaran {keluaran} berada di dalam pohon repo ({REPO}). Data nyata tak boleh masuk git: "
        "pilih direktori di luar repo."
    )


def tulis_aman(path: Path, isi: str) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8", newline="") as fh:
        fh.write(isi)


def tulis_keluaran(keluaran: Path, jelas, karantina, jumlah, ukuran_lot: int, dry_run: bool) -> list[Path]:
    keluaran.mkdir(parents=True, exist_ok=True)
    os.chmod(keluaran, 0o700)
    lot_file: list[Path] = []
    for i in range(0, len(jelas), ukuran_lot):
        p = keluaran / f"lot-{i // ukuran_lot + 1:04d}.json"
        tulis_aman(p, json.dumps({"dry_run": dry_run, "baris": [ke_json(r) for r in jelas[i:i + ukuran_lot]]},
                                 ensure_ascii=False))
        lot_file.append(p)
    import io

    buf = io.StringIO()
    w = csv.writer(buf)
    w.writerow(["rujukan", "alasan", "nip_asli", "nama", "usulan_nip"])
    for r in karantina:
        w.writerow([f"{r['berkas']}:{r['baris']}", r["alasan"], r["nip_asli"], r["nama"], r["usulan"]])
    tulis_aman(keluaran / "karantina.csv", buf.getvalue())
    tulis_aman(keluaran / "laporan.json", json.dumps(dict(sorted(jumlah.items())), indent=2, ensure_ascii=False))
    return lot_file


# ───────────────────────── mengirim ke aplikasi ─────────────────────────
def kirim(url: str, lot_file: list[Path], keluaran: Path, muat: bool) -> int:
    token = os.environ.get("SIMPEL_TOKEN", "")
    if not token:
        raise SystemExit("--kirim butuh token admin di environment SIMPEL_TOKEN (jangan lewat argumen).")
    total: collections.Counter = collections.Counter()
    alasan: collections.Counter = collections.Counter()
    ditolak_baris: list[list[str]] = []
    for p in lot_file:
        badan = json.loads(p.read_text(encoding="utf-8"))
        badan["dry_run"] = not muat
        req = urllib.request.Request(
            url.rstrip("/") + "/pakaian-dinas/impor-profil",
            data=json.dumps(badan).encode(),
            headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=300) as r:
                h = json.loads(r.read())["data"]
        except urllib.error.HTTPError as e:
            raise SystemExit(f"{p.name}: HTTP {e.code} {e.read()[:300]!r}")
        for k in ("diterima", "baris_dimuat", "profil_baru", "profil_sudah_ada", "ukuran_ditulis",
                  "ukuran_dilewati_sudah_diisi"):
            total[k] += h[k]
        for d in h["ditolak"]:
            alasan[d["alasan"].split(":")[0][:60]] += 1
            ditolak_baris.append([d["rujukan"], d["nip_tersamar"], d["alasan"]])
        print(f"{p.name}: diterima={h['diterima']} dimuat={h['baris_dimuat']} ditolak={len(h['ditolak'])}")
    import io

    buf = io.StringIO()
    csv.writer(buf).writerows([["rujukan", "nip_tersamar", "alasan"]] + ditolak_baris)
    tulis_aman(keluaran / "ditolak-server.csv", buf.getvalue())
    print(("MEMUAT" if muat else "PEMERIKSAAN (tidak menulis)"), dict(total))
    for a, n in alasan.most_common():
        print(f"  ditolak server: {n:6d}  {a}")
    return 0


# ───────────────────────── swauji ─────────────────────────
def _xlsx(path: Path, baris: list[list[str]]) -> None:
    def sel(r: int, c: int, v: str) -> str:
        ref = chr(65 + c) + str(r)
        v = v.replace("&", "&amp;").replace("<", "&lt;")
        return f'<c r="{ref}" t="inlineStr"><is><t>{v}</t></is></c>'

    rows = "".join(
        f'<row r="{i + 1}">{"".join(sel(i + 1, j, v) for j, v in enumerate(b))}</row>' for i, b in enumerate(baris)
    )
    with zipfile.ZipFile(path, "w") as z:
        z.writestr("xl/workbook.xml", '<workbook xmlns="%s" xmlns:r="%s"><sheets><sheet name="daftar" sheetId="1" '
                   'r:id="rId1"/></sheets></workbook>' % (NS["m"], NS["r"]))
        z.writestr("xl/_rels/workbook.xml.rels",
                   '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
                   '<Relationship Id="rId1" Target="worksheets/sheet1.xml"/></Relationships>')
        z.writestr("xl/worksheets/sheet1.xml", f'<worksheet xmlns="{NS["m"]}"><sheetData>{rows}</sheetData></worksheet>')


def self_test() -> int:
    gagal = 0

    def ok(kondisi: bool, pesan: str) -> None:
        nonlocal gagal
        if not kondisi:
            print(f"  GAGAL: {pesan}")
            gagal += 1

    # NIP: struktur, PPPK, potongan nomor lama, notasi ilmiah, jenis kelamin
    ok(nip_valid("199203142014031001"), "NIP ASN yang sah")
    ok(nip_valid("199901012024212001"), "NIP PPPK (YYYY + 21) yang sah")
    ok(not nip_valid("199213142014031001"), "bulan lahir 13 ditolak")
    ok(klasifikasi_nip("199203142014031001 123456789") == ("LEGACY_APPENDED", "199203142014031001"), "NIP + nomor lama dipotong")
    ok(klasifikasi_nip("19920314 201403 1 001 123456789")[0] == "LEGACY_APPENDED", "NIP dipecah spasi + nomor lama")
    ok(klasifikasi_nip("1.99E+17")[0] == "NOTASI_ILMIAH", "notasi ilmiah tak dianggap NIP")
    ok(klasifikasi_nip("")[0] == "KOSONG" and klasifikasi_nip("12345")[0] == "TERLALU_PENDEK", "kosong dan pendek")
    ok(digit_jk_beda("199203142014032001", "L") and not digit_jk_beda("199203142014031001", "L"), "digit jenis kelamin")
    ok(norm_ukuran("32.0") == "32" and norm_ukuran("xl") == "XL" and norm_ukuran("-") is None, "ukuran dirapikan")
    ok(inti_nama("Dr. Budi Santoso, S.H., M.H.") == inti_nama("BUDI SANTOSO"), "gelar tak mempengaruhi nama")

    with tempfile.TemporaryDirectory() as tmp:
        t = Path(tmp)
        kj, kt = t / "kejagung.xlsx", t / "kejati.xlsx"
        A, B, C, D, E = "199203142014031001", "198001012005011002", "199504152020122001", "198507102010011003", "197001012000031004"
        _xlsx(kj, [["No", "Nama", "NIP", "Gender", "Hijab", "Sepatu PDH"],
                   ["1", "Budi Santoso", A, "L", "Tidak", "42.0"],            # juga di Kejati, sama
                   ["2", "Siti Aminah", C, "P", "YA", "38"],
                   ["3", "Rusak Satu", "1.99E+17", "L", "Tidak", "41"],        # notasi ilmiah -> karantina
                   ["4", "Tanpa Sepatu", D, "L", "Tidak", ""]])                # tanpa ukuran -> dilewati
        _xlsx(kt, [["No", "Nama", "NIP", "gender", "hijab", "kaos", "baju", "celana", "topi", "sepatu pdl",
                    "sepatu pdh", "nama"],
                   ["1", "Budi Santoso", A + " 123456789", "L", "Tidak", "xl", "L", "32.0", "58.0", "42.0", "42.0", "BUDI S"],
                   ["2", "Orang Lain", B, "L", "Tidak", "M", "M", "30", "57", "40", "40", ""],
                   ["3", "Nama Berbeda", B, "L", "Tidak", "L", "L", "31", "57", "41", "41", ""],   # NIP dipakai orang lain
                   ["4", "Ganda Sama", E, "L", "Tidak", "M", "M", "30", "57", "40", "40", ""],
                   ["5", "Ganda Sama", E, "L", "Tidak", "M", "M", "30", "57", "40", "40", ""],     # identik -> digabung
                   ["6", "Ganda Beda", D, "L", "Tidak", "M", "M", "30", "57", "40", "40", ""],
                   ["7", "Ganda Beda", D, "L", "Tidak", "L", "M", "30", "57", "40", "40", ""],     # data berbeda -> karantina
                   ["8", "Jenis Kelamin Salah", "199203142014032002", "L", "Tidak", "M", "M", "30", "57", "40", "40", ""]])
        rekam = baca_berkas(str(kj), "kejagung") + baca_berkas(str(kt), "kejati")
        ok(len(rekam) == 12, f"12 rekam dibaca, ada {len(rekam)}")
        jelas, karantina, jumlah = siapkan(rekam)
        nips = {r["nip"] for r in jelas}
        ok(A in nips and C in nips and E in nips, f"A, C, E jelas: {sorted(nips)}")
        ok(B not in nips and D not in nips, "B (dipakai dua orang) dan D (ganda berbeda) tertahan")
        ok(len(jelas) == 3, f"3 baris jelas, ada {len(jelas)}")
        a = next(r for r in jelas if r["nip"] == A)
        # Budi: sepatu PDH dari dua berkas sama (42) digabung; baju mengisi PDH DAN PDL; nama dada dari kolom kedua
        ok(a["ukuran"].get(("PDH", "Sepatu Dinas")) == "42" and a["ukuran"].get(("PDL", "Sepatu")) == "42", "sepatu PDH dan PDL")
        ok(a["ukuran"].get(("PDH", "Pakaian Dinas")) == "L" and a["ukuran"].get(("PDL", "Baju")) == "L", "baju untuk PDH dan PDL")
        ok(a["ukuran"].get(("PDL", "Kaos")) == "XL" and a["ukuran"].get(("PDL", "Topi")) == "58", "kaos dan topi (58.0 -> 58)")
        ok(a["ukuran"].get(("PDL", "Nama Dada")) == "BUDI S", "kolom nama kedua = nama dada")
        ok(a["ukuran"].get(("PDH", "Celana")) == "32", "celana 32.0 -> 32")
        alasan = {k.split(":", 1)[1] for k in jumlah if k.startswith("karantina:")}
        ok({"nip_notasi_ilmiah", "nip_dipakai_orang_lain", "ganda_data_berbeda", "digit_jk_tidak_cocok"} <= alasan, f"alasan karantina: {alasan}")
        ok(jumlah["dilewati:tanpa_ukuran"] == 1 and jumlah["digabung:ganda_sama_orang"] >= 1, f"hitungan: {dict(jumlah)}")

        # Verifikasi dengan rujukan: NIP yang tak ada di sana tertahan, usulan hanya untuk nama yang sama
        ruj = t / "ruj.csv"
        ruj.write_text(f"{A},Budi Santoso\n{C},Siti Aminah\n{E},Ganda Sama\n199103142014031009,Rusak Satu\n", encoding="utf-8")
        r = baca_rujukan(str(ruj))
        jelas2, kar2, jumlah2 = siapkan(rekam, r)
        ok({x["nip"] for x in jelas2} == {A, C, E}, "dengan rujukan: tetap A, C, E")
        rusak = [k for k in kar2 if k["nama"] == "Rusak Satu"]
        ok(len(rusak) == 1 and rusak[0]["usulan"] == "199103142014031009", f"usulan NIP dari rujukan: {rusak}")
        ok(all(x["berkas"] != "kejagung" or x["nama"] != "Rusak Satu" for x in jelas2), "yang rusak tak dimuat otomatis")
        jelas3, _, j3 = siapkan(rekam, r, muat_pemulihan=True)
        ok(any(x["nip"] == "199103142014031009" for x in jelas3) and j3["pulih:dimuat"] == 1, "--muat-pemulihan memuat yang unik dan sama namanya")

        # Keluaran: izin 0600, laporan tanpa data pribadi, lot sesuai ukuran, dan ditolak di dalam repo
        keluar = t / "keluar"
        lots = tulis_keluaran(keluar, jelas, karantina, jumlah, 2, True)
        ok(len(lots) == 2, "3 baris, 2 per lot = 2 lot")
        for f in list(keluar.iterdir()):
            ok((f.stat().st_mode & 0o777) == 0o600, f"{f.name} izin 0600")
        laporan = (keluar / "laporan.json").read_text()
        ok(A not in laporan and "Budi" not in laporan, "laporan tanpa NIP dan nama")
        badan = json.loads(lots[0].read_text())
        ok(badan["dry_run"] is True and all(len(b["nip"]) == 18 for b in badan["baris"]), "lot berbentuk permintaan")
        try:
            tolak_di_dalam_repo(REPO / "docs" / "keluar")
            ok(False, "keluaran di dalam repo harus ditolak")
        except SystemExit:
            pass
        tolak_di_dalam_repo(keluar)
    print("swauji " + ("GAGAL" if gagal else "lulus"))
    return 1 if gagal else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--kejagung", help="berkas xlsx Kejagung (hanya sepatu PDH)")
    ap.add_argument("--kejati", help="berkas xlsx Kejati (PDH + PDL)")
    ap.add_argument("--keluaran", help="direktori keluaran, DI LUAR repo")
    ap.add_argument("--lot", type=int, default=1000, help="baris per lot (maks 2000)")
    ap.add_argument("--rujukan-pegawai", help="CSV nip[,nama] (mis. ekspor MySIMKARI): verifikasi dan usulan perbaikan")
    ap.add_argument("--terima-digit-jk-beda", action="store_true", help="muat baris yang digit jenis kelamin NIP-nya beda")
    ap.add_argument("--muat-pemulihan", action="store_true", help="muat NIP rusak yang pulih unik dari rujukan dengan nama sama")
    ap.add_argument("--kirim", metavar="URL", help="kirim lot ke API (SIMPEL_TOKEN di environment)")
    ap.add_argument("--muat", action="store_true", help="dengan --kirim: benar-benar menulis (bawaan: hanya memeriksa)")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    if not (a.keluaran and (a.kejagung or a.kejati)):
        ap.error("butuh --keluaran dan minimal satu dari --kejagung/--kejati")
    if not 1 <= a.lot <= 2000:
        ap.error("--lot harus 1..2000")
    keluaran = Path(a.keluaran)
    tolak_di_dalam_repo(keluaran)
    rekam: list[dict] = []
    for jenis in ("kejagung", "kejati"):
        berkas = getattr(a, jenis)
        if berkas:
            rekam += baca_berkas(berkas, jenis)
    rujukan = baca_rujukan(a.rujukan_pegawai) if a.rujukan_pegawai else None
    jelas, karantina, jumlah = siapkan(rekam, rujukan, a.terima_digit_jk_beda, a.muat_pemulihan)
    lot_file = tulis_keluaran(keluaran, jelas, karantina, jumlah, a.lot, dry_run=not a.muat)
    print(f"dibaca {len(rekam)} baris -> jelas {len(jelas)} ({len(lot_file)} lot), karantina {len(karantina)}")
    for k, n in sorted(jumlah.items()):
        print(f"  {n:7d}  {k}")
    print(f"keluaran di {keluaran} (izin 0600; berisi NIP dan nama, jangan masuk repo atau obrolan)")
    if a.kirim:
        return kirim(a.kirim, lot_file, keluaran, a.muat)
    return 0


if __name__ == "__main__":
    sys.exit(main())
