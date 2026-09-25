#!/usr/bin/env python3
"""Generate the README screenshot gallery from the capture report.

The gallery is DERIVED from `capture-report.json` (written by `capture.mjs`),
not hand-maintained. A hand-written list is the failure this whole exercise is
guarding against: it is correct on the day it is written, and the next route
added to the router appears in the app but not in the documentation.

Every image referenced here has already passed `check.py` — the generator
refuses to publish a screenshot that is blank, uniform or collapsed, so the
gallery cannot contain a picture that shows nothing.

Output: a Markdown fragment on stdout, plus `docs/assets/screenshots/README.md`
(the per-image index with metrics).

Usage:  python3 tests/e2e/screenshots/gallery.py [--report FILE] [--out FILE]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SHOTS = REPO / "docs" / "assets" / "screenshots"

# Human titles for the route families, so the gallery reads as documentation
# rather than as a file listing. Keyed by a path fragment; the LONGEST matching
# prefix wins, so `/admin/workflow-monitoring` is not swallowed by `/admin`.
SECTIONS: list[tuple[str, str, str]] = [
    # (path fragment, section title, one-line purpose)
    ("/portal/login", "Masuk & Sesi", "Titik masuk SSO dan hasil akhir sebuah sesi."),
    ("/portal/callback", "Masuk & Sesi", "Penukaran authorization code menjadi sesi."),
    ("/portal/mfa", "Autentikasi Dua Faktor", "Enrolment, verifikasi, dan kode pemulihan."),
    ("/perlengkapan/simpel/v2/login", "Masuk & Sesi", "Halaman masuk mandiri Perlengkapan."),
    ("/portal/apps", "Portal", "Katalog microfrontend dan pintasan modul."),
    ("/portal/notifications", "Portal", "Kotak masuk lintas-modul."),
    ("/portal/profile", "Akun", "Identitas dan pengaturan akun pengguna."),
    ("/portal/passkeys", "Akun", "Passkey/WebAuthn sebagai faktor kedua."),
    ("/portal/password", "Akun", "Perubahan kata sandi."),
    ("/portal/sessions", "Akun", "Sesi aktif dan pencabutannya."),
    ("/portal/settings", "Akun", "Preferensi tampilan dan bahasa."),
    ("/portal/admin", "Administrasi Portal", "IAM: pengguna, peran, klien OAuth2, audit."),
    ("/portal/dashboard", "Portal", "Ringkasan bagi pengguna yang sudah masuk."),
    ("/portal", "Portal", "Pintu masuk portal."),
    ("/bank-aset", "Bank Aset", "Katalog BMN: daftar, dashboard, sebaran, dan QR."),
    ("/kebutuhan-bmn", "Kebutuhan BMN", "Usulan kebutuhan per satker dan alur persetujuannya."),
    ("/pakaian-dinas", "Pakaian Dinas", "Jenis, campaign pengajuan, ukuran, dan laporan."),
    ("/pengelolaan/pemakaian", "Pemakaian BMN", "Izin pemakaian beserta rantai persetujuannya."),
    ("/pengelolaan/penghapusan", "Penghapusan BMN", "Usulan penghapusan sampai Konsep SK."),
    ("/analitik", "Analitik & Roadmap", "Proyeksi kebutuhan sarana-prasarana."),
    ("/notifikasi", "Notifikasi", "Kotak masuk notifikasi Perlengkapan."),
    ("/bantuan", "Bantuan", "Panduan, FAQ, dan helpdesk."),
    ("/admin/roles", "Administrasi Perlengkapan", "Peran dan hak akses."),
    ("/admin/audit", "Administrasi Perlengkapan", "Jejak audit."),
    ("/admin/master", "Administrasi Perlengkapan", "Data referensi."),
    ("/admin/templates", "Administrasi Perlengkapan", "Template dokumen."),
    ("/admin/workflow-monitoring", "Administrasi Perlengkapan", "Pemantauan SLA alur kerja."),
    ("/admin/workflow-delegation", "Administrasi Perlengkapan", "Pelimpahan wewenang validator."),
    ("/admin/workflow", "Administrasi Perlengkapan", "Definisi alur kerja."),
    ("/admin", "Administrasi Perlengkapan", "Halaman administrasi."),
    ("/dashboard", "Dashboard & Beranda", "Beranda aplikasi dan indeks modul."),
    ("/", "Dashboard & Beranda", "Beranda aplikasi dan indeks modul."),
]

# Order sections deterministically, not by dict insertion (which depends on the
# route order the router happens to be written in).
SECTION_ORDER = [
    "Portal",
    "Masuk & Sesi",
    "Autentikasi Dua Faktor",
    "Akun",
    "Administrasi Portal",
    "Dashboard & Beranda",
    "Bank Aset",
    "Kebutuhan BMN",
    "Pakaian Dinas",
    "Pemakaian BMN",
    "Penghapusan BMN",
    "Analitik & Roadmap",
    "Notifikasi",
    "Bantuan",
    "Administrasi Perlengkapan",
    "Kontrol Akses (RBAC)",
]


def section_for(path: str) -> str:
    best: tuple[int, str] | None = None
    for frag, title, _ in SECTIONS:
        if frag in path:
            if best is None or len(frag) > best[0]:
                best = (len(frag), title)
    return best[1] if best else "Lain-lain"


def blurb_for(title: str) -> str:
    for _, t, blurb in SECTIONS:
        if t == title:
            return blurb
    return ""


def humanise(route: str) -> str:
    """A readable title when the route has no curated section."""
    tail = route.rstrip("/").split("/")[-1] or "beranda"
    return tail.replace("-", " ").replace("_", " ").capitalize()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--report", default=str(SHOTS / "capture-report.json"))
    ap.add_argument("--out", default=str(SHOTS / "GALLERY.md"))
    # Image references are relative to the file that CONTAINS the gallery. The
    # fragment lives beside the images; the root README lives two directories up
    # and needs `docs/` in front of every path.
    ap.add_argument("--prefix", default="", help="prepended to every image path")
    # When set, the generated block REPLACES the region between the two markers
    # in the named file instead of being written standalone. Without this the
    # README copy is a second, hand-pasted list that silently goes stale the
    # moment a route is added — the exact failure `--out` alone would reintroduce.
    ap.add_argument(
        "--embed",
        default=None,
        help="path to a file whose BEGIN/END marker region is replaced",
    )
    args = ap.parse_args()

    report_path = Path(args.report)
    if not report_path.exists():
        print(f"missing {report_path} — run capture.mjs first", file=sys.stderr)
        return 2
    report = json.loads(report_path.read_text())

    image_report_path = SHOTS / "image-report.json"
    if not image_report_path.exists():
        print(f"missing {image_report_path} — run check.py first", file=sys.stderr)
        return 2
    image_metrics = {m["file"]: m for m in json.loads(image_report_path.read_text())}

    # Refuse to publish anything check.py rejected: a gallery is a claim that
    # every listed page renders, and a blank PNG makes that claim false.
    rejected = {
        name: m["flags"] for name, m in image_metrics.items() if m.get("flags")
    }
    # Same reasoning one layer up: a page that rendered a 404, a panic, or a raw
    # enum is not something to advertise as app coverage. The capture report is
    # the authority on that, so a non-empty finding blocks publication rather
    # than being silently illustrated.
    defects = {
        f["file"]: f["flags"]
        for f in report.get("findings", [])
        if f.get("flags")
    }
    unresolved = report.get("unresolved") or []

    # Validate BEFORE writing anything. The checks below gate publication, so
    # running them after `write_text` would leave a rejected gallery already
    # embedded in the README with only the exit code disagreeing — the file, not
    # the exit code, is what a reader sees.
    if rejected or unresolved or defects:
        if rejected:
            print(f"\nREFUSED to publish {len(rejected)} rejected image(s):", file=sys.stderr)
            for name, flags in sorted(rejected.items()):
                print(f"  {name}: {', '.join(flags)}", file=sys.stderr)
        if unresolved:
            print(f"\nREFUSED to publish: {len(unresolved)} route(s) not captured:", file=sys.stderr)
            for u in unresolved:
                print(f"  {u}", file=sys.stderr)
        if defects:
            print(
                f"\nREFUSED to publish: {len(defects)} page(s) carry a defect flag:",
                file=sys.stderr,
            )
            for name, flags in sorted(defects.items()):
                print(f"  {name}: {', '.join(flags)}", file=sys.stderr)
        return 1

    groups: dict[str, list[dict]] = {}
    for shot in report["capturedRoutes"]:
        title = section_for(shot["route"])
        groups.setdefault(title, []).append(shot)

    lines: list[str] = []
    lines.append("<!-- Generated by tests/e2e/screenshots/gallery.py — do not edit by hand. -->")
    lines.append("")
    lines.append(
        f"Seluruh **{report['captured']} tampilan** di bawah ini diambil otomatis dari aplikasi "
        "yang benar-benar berjalan (satu origin, sama seperti produksi), memakai sesi hasil login "
        "nyata ke *Authenc*. Setiap gambar telah diperiksa: tidak ada yang kosong, putih polos, "
        "atau berisi halaman 404."
    )
    lines.append("")
    lines.append(
        "| Cara memperbarui | Perintah |"
    )
    lines.append("|---|---|")
    lines.append(
        "| Ambil ulang seluruh gambar | `node tests/e2e/screenshots/capture.mjs` |"
    )
    lines.append("| Periksa kualitas gambar | `python3 tests/e2e/screenshots/check.py` |")
    lines.append(
        "| Bentuk ulang galeri ini (termasuk README) | "
        "`python3 tests/e2e/screenshots/gallery.py --prefix docs/assets/screenshots/ "
        "--embed README.md` |"
    )
    lines.append("")

    for title in SECTION_ORDER:
        if title not in groups:
            continue
        shots = sorted(groups[title], key=lambda s: s["route"])
        lines.append(f"### {title}")
        lines.append("")
        blurb = blurb_for(title)
        if blurb:
            lines.append(blurb)
            lines.append("")
        for shot in shots:
            name = shot["file"]
            m = image_metrics.get(name, {})
            size = f"{m.get('width', '?')}×{m.get('height', '?')}"
            caption = shot["route"]
            if "(operator)" in shot["route"]:
                caption = f"`{shot['route']}` — ditolak oleh guard"
            else:
                caption = f"`{shot['route']}`"
            lines.append(f"**{caption}**")
            lines.append("")
            lines.append(f"![{shot['route']}]({args.prefix}{name})")
            lines.append("")
            lines.append(f"<sub>{size} piksel</sub>")
            lines.append("")

    out = Path(args.out)
    out.write_text("\n".join(lines))

    print(f"wrote {out} ({len(report['capturedRoutes'])} images in {len(groups)} sections)")

    if args.embed:
        embed_path = Path(args.embed)
        text = embed_path.read_text()
        begin = "<!-- BEGIN AUTOGENERATED SCREENSHOTS"
        end = "<!-- END AUTOGENERATED SCREENSHOTS -->"
        i = text.find(begin)
        j = text.find(end)
        if i == -1 or j == -1:
            print(
                f"missing screenshot markers in {embed_path} — expected a line "
                f"starting with {begin!r} and a line {end!r}",
                file=sys.stderr,
            )
            return 2
        # Keep the BEGIN marker comment itself (it carries the "do not edit"
        # warning), replace only what sits between the two markers.
        begin_line_end = text.find("\n", i)
        block = "\n" + "\n".join(lines) + "\n\n"
        embed_path.write_text(text[: begin_line_end + 1] + block + text[j:])
        print(f"embedded gallery into {embed_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())