#!/usr/bin/env python3
"""Penjaga otorisasi: matriks rute perlengkapan + daftar peran-literal.

MENGAPA ADA
-----------
Dua kelas cacat yang sama-sama lolos kompilasi, lolos tes, dan lolos review
karena tak ada yang gagal — hanya seseorang yang seharusnya ditolak yang tidak
ditolak:

1. **Handler tulis tanpa gerbang peran.** Otentikasi kini deny-by-default di
   router (`shared::middleware::require_authentication`), tetapi otentikasi
   bukan otorisasi: 18 handler tulis perlengkapan hanya membaca `Claims` untuk
   mengambil user id (update/hapus periode, `jml_setuju`, prioritas, batch,
   aktivasi izin, auto-expire, ...). Siapa pun yang punya token bisa
   memanggilnya. Aturan di bawah membuat "handler tulis baru tanpa gerbang"
   menjadi MERAH, bukan diam.

2. **Daftar peran-literal yang menyalin `lib_core::authz`.** Pra-#930 ada empat
   definisi "siapa admin" (`starts_with("admin")`, `roles.contains("admin")`,
   `admin | admin_pusat | superadmin`, dan satu lagi + validator_pusat). Mereka
   menyimpang diam-diam dan sebagian besar bug otorisasi berasal dari sana.
   Satu-satunya tempat yang boleh menyebut daftar itu adalah
   `lib/core/src/authz.rs`.

ATURAN
------
R1  Setiap handler pada `routes.rs` harus ditemukan (kalau tidak, matriks buta).
R2  Setiap handler (selain handshake WebSocket) menerima `Claims`.
R3  Setiap handler TULIS (POST/PUT/PATCH/DELETE) memuat token gerbang
    (`require_*`, `.authorize(`/`authorize_as(`, `enforce_*`,
    `validate_delegation_authority`) — atau terdaftar di `WRITE_ALLOWLIST`
    LENGKAP dengan alasan.
R4  Setiap handler BACA (GET) memuat token scope/gerbang — atau terdaftar di
    `READ_ALLOWLIST` (data referensi nasional) dengan alasan.
R5  Tidak ada daftar peran-literal di luar `lib/core/src/authz.rs`
    (`"admin" | "superadmin"`, `== "admin"`, `["admin", "admin_pusat"]`).
    Waiver satu baris: komentar `guard:allow-role-literal` + alasan.
R6  Allowlist tidak boleh basi: entri yang tak lagi cocok dengan rute mana pun
    membuat penjaga MERAH (kalau tidak, allowlist tumbuh jadi lubang permanen).

Allowlist adalah PENGAKUAN, bukan pembebasan: tiap entri memuat alasan yang bisa
dibantah reviewer.

Jalankan:  python3 infra/scripts/check-authz-guards.py
           python3 infra/scripts/check-authz-guards.py --self-test
Hanya pustaka standar (lihat check-guard-imports-are-stdlib.py).
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# --------------------------------------------------------------------------
# Allowlist — (METODE, path) -> alasan
# --------------------------------------------------------------------------

# Handshake WebSocket: peramban tak bisa memasang header Authorization; handler
# memvalidasi token query sendiri (dashboard/websocket.rs).
SELF_AUTH = {("GET", "/dashboard/ws")}

WRITE_ALLOWLIST: dict[tuple[str, str], str] = {
    ("PATCH", "/notifikasi/{id}/read"): "kotak masuk milik sendiri; baris dipilih dengan user_id dari klaim",
    ("POST", "/notifikasi/read-all"): "kotak masuk milik sendiri; dibatasi user_id dari klaim",
    ("POST", "/bantuan/tiket"): "setiap pengguna boleh membuka tiket; pemilik = user_id dari klaim (TicketActor)",
    ("POST", "/bantuan/tiket/{id}/komentar"): "pemilik tiket atau staf; diputuskan TicketActor.access()",
    ("PUT", "/bantuan/tiket/{id}/status"): "khusus staf helpdesk; TicketActor.require_staff() di repository/service",
    ("POST", "/pakaian-dinas/ukuran-pakaian-pegawai"): "swalayan: NIP diambil dari klaim, bukan dari body",
    ("POST", "/workflow/delegations/{id}/revoke"): "hanya pemberi delegasi; DelegationManager::revoke_delegation memeriksa delegator",
}

READ_ALLOWLIST: dict[tuple[str, str], str] = {
    ("GET", "/bank-aset/kodefikasi"): "kodefikasi SIMAN: data referensi nasional, bukan data satker",
    ("GET", "/pakaian-dinas/jenis"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/jenis/{id}"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/spesifikasi"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/spesifikasi/{id}"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/subspesifikasi"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/subspesifikasi/{id}"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/ukuran"): "master referensi ukuran",
    ("GET", "/pakaian-dinas/jenis/{jenis_id}/spesifikasi"): "master referensi pakaian dinas",
    ("GET", "/pakaian-dinas/ukuran-pakaian-pegawai"): "swalayan: NIP dari klaim",
    ("GET", "/kebutuhan-bmn/laporan/status-options"): "daftar opsi status (referensi)",
    ("GET", "/kebutuhan-bmn/wilayah"): "daftar Kejati (referensi)",
    ("GET", "/kebutuhan-bmn/pengajuan/{id}/bmn-referensi"): "daftar BMN yang diizinkan periode (referensi periode nasional)",
    ("GET", "/kebutuhan-bmn/search"): "pencarian TAJUK periode (data nasional); tidak menyentuh isian satker",
    ("GET", "/kebutuhan-bmn/search/suggestions"): "saran nama periode (data nasional)",
    ("GET", "/kebutuhan-bmn/pengajuan/{id}/export"): "stub (belum menghasilkan berkas); tak mengembalikan data",
    ("GET", "/workflow/definitions/{name}"): "definisi alur kerja (kode statis, bukan data)",
    ("GET", "/workflow/definitions"): "definisi alur kerja (kode statis, bukan data)",
    ("GET", "/integrasi/circuit-status"): "status circuit breaker (operasional, tanpa data satker)",
    ("GET", "/bantuan/tiket"): "dibatasi TicketActor.access(): milik sendiri, atau semua untuk staf",
    ("GET", "/bantuan/tiket/{id}"): "dibatasi TicketActor.access()",
    ("GET", "/bantuan/tiket/{id}/komentar"): "dibatasi TicketActor.access()",
}

# Token yang menandai gerbang otorisasi pada handler TULIS.
WRITE_GUARD = re.compile(
    r"\brequire_[a-z_]+\s*\(|\.authorize(?:_as)?\s*\(|\benforce_[a-z_]+\s*\(|"
    r"\bvalidate_delegation_authority\s*\("
)
# Token yang menandai scope/gerbang pada handler BACA.
READ_GUARD = re.compile(
    r"SatkerScope|AsetScope|\bscope\b|\brequire_[a-z_]+\s*\(|\.authorize(?:_as)?\s*\(|"
    r"\benforce_[a-z_]+\s*\(|\bcan\s*\(\s*(?:lib_core::authz::)?Capability|"
    r"\bexport_caller\b|\bclaims\.user_id\b|\bclaims\.nip\b"
)

# Peran-literal: alternasi >=2 nama admin, atau perbandingan ke satu nama admin,
# atau larik berisi >=2 nama admin.
ADMIN_NAMES = r'"(?:admin|superadmin|admin_pusat)"'
LITERAL_PATTERNS = [
    (re.compile(rf"{ADMIN_NAMES}\s*\|\s*{ADMIN_NAMES}"), "alternasi peran admin"),
    (re.compile(rf"==\s*{ADMIN_NAMES}|{ADMIN_NAMES}\s*=="), "perbandingan ke peran admin"),
    (re.compile(rf"\[\s*{ADMIN_NAMES}\s*,\s*{ADMIN_NAMES}"), "larik peran admin"),
    (re.compile(r'starts_with\(\s*"admin'), "prefix-match peran admin"),
]
LITERAL_ALLOWED_FILES = {"lib/core/src/authz.rs"}
LITERAL_SCAN_DIRS = [
    "layanan/perlengkapan/src",
    "layanan/authenc/crates",
    "layanan/gateway/src",
    "lib/backend/src",
    "lib/core/src",
]

FN_RE = re.compile(r"pub\s+(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", re.S)
EXTRACTORS = re.compile(r"State\(|Path\(|Json\(|Query\(|Claims|Multipart|WebSocketUpgrade|ClientIp")
METHOD_RE = re.compile(r"\b(get|post|put|delete|patch)\(\s*([\w:]+)\s*\)")


def balanced(src: str, i: int, open_c: str = "(", close_c: str = ")") -> int:
    depth, j = 1, i
    while j < len(src) and depth:
        if src[j] == open_c:
            depth += 1
        elif src[j] == close_c:
            depth -= 1
        j += 1
    return j


def index_functions(src_root: Path) -> dict[str, list[dict]]:
    fns: dict[str, list[dict]] = {}
    for p in sorted(src_root.rglob("*.rs")):
        src = p.read_text(encoding="utf-8", errors="replace")
        for m in FN_RE.finditer(src):
            sig_end = balanced(src, m.end())
            sig = src[m.end() : sig_end - 1]
            k = src.find("{", sig_end)
            if k == -1:
                continue
            body = src[k : balanced(src, k + 1, "{", "}")]
            fns.setdefault(m.group(1), []).append(
                {"file": str(p.relative_to(src_root)), "sig": sig, "body": body}
            )
    return fns


def pick(fns: dict[str, list[dict]], name: str, hint: str) -> dict | None:
    """Handler vs service method of the same name: prefer the one with extractors."""
    cands = fns.get(name, [])
    scored = []
    for c in cands:
        s = 0
        if EXTRACTORS.search(c["sig"]):
            s += 4
        if "handler" in c["file"] or c["file"].endswith("api.rs"):
            s += 2
        if hint and hint in c["file"]:
            s += 3
        scored.append((s, c))
    scored.sort(key=lambda x: -x[0])
    return scored[0][1] if scored else None


def parse_routes(routes_src: str) -> list[tuple[str, str, str, str]]:
    """-> [(METHOD, path, handler, hint)]"""
    out = []
    pos = 0
    while True:
        i = routes_src.find(".route(", pos)
        if i == -1:
            break
        end = balanced(routes_src, i + len(".route("))
        inner = routes_src[i + len(".route(") : end - 1]
        # Komentar di dalam `.route(` (mis. penjelasan sebelum path) tak boleh
        # membuat rute tak terbaca — itu yang membuat parser buta pada rute yang
        # justru paling banyak dikomentari.
        inner = re.sub(r"//[^\n]*", "", inner)
        m = re.match(r'\s*"([^"]+)"\s*,(.*)', inner, re.S)
        if m:
            for meth, handler in METHOD_RE.findall(m.group(2)):
                parts = handler.split("::")
                out.append((meth.upper(), m.group(1), parts[-1], parts[-2] if len(parts) > 1 else ""))
        pos = end
    return out


def check_routes(perl_src: Path) -> list[str]:
    problems: list[str] = []
    routes_file = perl_src / "routes.rs"
    if not routes_file.exists():
        return [f"R1: {routes_file} tidak ada"]
    routes = parse_routes(routes_file.read_text(encoding="utf-8"))
    if not routes:
        return ["R1: routes.rs tidak menghasilkan satu pun rute — parser buta?"]
    fns = index_functions(perl_src)

    seen_write, seen_read = set(), set()
    for meth, path, handler, hint in routes:
        key = (meth, path)
        fn = pick(fns, handler, hint)
        if fn is None:
            problems.append(f"R1: {meth} {path}: handler `{handler}` tidak ditemukan")
            continue
        if key in SELF_AUTH:
            continue
        if "Claims" not in fn["sig"]:
            problems.append(f"R2: {meth} {path} ({handler}, {fn['file']}): tidak menerima `Claims`")
        if meth in ("POST", "PUT", "PATCH", "DELETE"):
            seen_write.add(key)
            if not WRITE_GUARD.search(fn["body"]) and key not in WRITE_ALLOWLIST:
                problems.append(
                    f"R3: {meth} {path} ({handler}, {fn['file']}): handler TULIS tanpa gerbang peran/policy. "
                    "Tambahkan `claims.require_*(...)` / `Policy.authorize(...)`, atau daftarkan di "
                    "WRITE_ALLOWLIST dengan alasan."
                )
        else:
            seen_read.add(key)
            if not READ_GUARD.search(fn["sig"] + fn["body"]) and key not in READ_ALLOWLIST:
                problems.append(
                    f"R4: {meth} {path} ({handler}, {fn['file']}): handler BACA tanpa scope/gerbang. "
                    "Pakai SatkerScope::from_claims / require_*, atau daftarkan di READ_ALLOWLIST dengan alasan."
                )

    all_keys = {(m, p) for m, p, _, _ in routes}
    for label, allow in (("WRITE_ALLOWLIST", WRITE_ALLOWLIST), ("READ_ALLOWLIST", READ_ALLOWLIST)):
        for key in allow:
            if key not in all_keys:
                problems.append(f"R6: {label} memuat {key[0]} {key[1]} yang tak lagi ada di routes.rs — hapus entri basi")
    # Entri allowlist yang sebenarnya SUDAH punya gerbang juga basi: pembebasan yang tak dipakai.
    for meth, path, handler, hint in routes:
        key = (meth, path)
        fn = pick(fns, handler, hint)
        if fn is None:
            continue
        if key in WRITE_ALLOWLIST and WRITE_GUARD.search(fn["body"]):
            problems.append(f"R6: WRITE_ALLOWLIST {meth} {path} sudah punya gerbang — hapus dari allowlist")
        if key in READ_ALLOWLIST and READ_GUARD.search(fn["sig"] + fn["body"]):
            problems.append(f"R6: READ_ALLOWLIST {meth} {path} sudah punya scope/gerbang — hapus dari allowlist")
    return problems


def strip_tests(src: str) -> str:
    """Buang modul tes (`#[cfg(test)]` ke bawah): tes sah memakai literal peran."""
    i = src.find("#[cfg(test)]")
    return src if i == -1 else src[:i]


def check_literals(repo: Path) -> list[str]:
    problems: list[str] = []
    for d in LITERAL_SCAN_DIRS:
        base = repo / d
        if not base.exists():
            continue
        for p in sorted(base.rglob("*.rs")):
            rel = p.relative_to(repo).as_posix()
            if rel in LITERAL_ALLOWED_FILES or "/tests/" in rel or rel.endswith(("tests.rs", "_tests.rs", "_test.rs")):
                continue
            body = strip_tests(p.read_text(encoding="utf-8", errors="replace"))
            for lineno, line in enumerate(body.splitlines(), 1):
                code = line.split("//", 1)[0]
                if "guard:allow-role-literal" in line:
                    continue
                for pat, what in LITERAL_PATTERNS:
                    if pat.search(code):
                        problems.append(
                            f"R5: {rel}:{lineno}: {what} — pakai lib_core::authz "
                            f"(is_admin_role / is_iam_admin_role / ADMIN_ROLES): {line.strip()[:110]}"
                        )
                        break
    return problems


def run(repo: Path) -> list[str]:
    return check_routes(repo / "layanan/perlengkapan/src") + check_literals(repo)


# --------------------------------------------------------------------------
# Self-test: penjaga yang tak pernah terlihat merah tak membuktikan apa pun.
# --------------------------------------------------------------------------

ROUTES_FIXTURE = """
pub fn create_routes() {
    Router::new()
        .route("/a", post(m::guarded_write))
        .route("/b", post(m::unguarded_write))
        .route("/c", get(m::scoped_read))
        .route("/d", get(m::unscoped_read))
        .route("/e", put(m::no_claims_write))
        .route("/f", post(m::missing_handler_zzz))
}
"""

HANDLERS_FIXTURE = """
pub async fn guarded_write(claims: Claims, Json(b): Json<X>) -> R { claims.require_role("operator_satker")?; ok() }
pub async fn unguarded_write(claims: Claims, Json(b): Json<X>) -> R { ok() }
pub async fn scoped_read(claims: Claims) -> R { let s = SatkerScope::from_claims(&claims); ok() }
pub async fn unscoped_read(claims: Claims) -> R { ok() }
pub async fn no_claims_write(Json(b): Json<X>) -> R { ok() }
"""


def _mk(tmp: Path, files: dict[str, str]) -> None:
    for rel, content in files.items():
        p = tmp / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")


def self_test() -> int:
    failures: list[str] = []

    def expect(cond: bool, msg: str) -> None:
        if not cond:
            failures.append(msg)

    global WRITE_ALLOWLIST, READ_ALLOWLIST
    saved = (WRITE_ALLOWLIST, READ_ALLOWLIST)
    WRITE_ALLOWLIST, READ_ALLOWLIST = {}, {}
    try:
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            _mk(tmp, {
                "layanan/perlengkapan/src/routes.rs": ROUTES_FIXTURE,
                "layanan/perlengkapan/src/m/handlers.rs": HANDLERS_FIXTURE,
            })
            probs = check_routes(tmp / "layanan/perlengkapan/src")
            txt = "\n".join(probs)
            expect("R3: POST /b" in txt, "handler tulis tanpa gerbang HARUS merah (R3)")
            expect("R3: POST /a" not in txt, "handler tulis BERGERBANG harus hijau")
            expect("R4: GET /d" in txt, "handler baca tanpa scope HARUS merah (R4)")
            expect("R4: GET /c" not in txt, "handler baca bersscope harus hijau")
            expect("R2: PUT /e" in txt, "handler tanpa Claims HARUS merah (R2)")
            expect("R1: POST /f" in txt, "handler tak ditemukan HARUS merah (R1)")

            # Allowlist membebaskan — dan allowlist basi merah.
            WRITE_ALLOWLIST = {("POST", "/b"): "uji", ("POST", "/hilang"): "basi", ("POST", "/a"): "sudah bergerbang"}
            probs = "\n".join(check_routes(tmp / "layanan/perlengkapan/src"))
            expect("R3: POST /b" not in probs, "allowlist harus membebaskan R3")
            expect("R6: WRITE_ALLOWLIST memuat POST /hilang" in probs, "allowlist basi HARUS merah (R6)")
            expect("R6: WRITE_ALLOWLIST POST /a sudah punya gerbang" in probs, "allowlist tak terpakai HARUS merah (R6)")
            WRITE_ALLOWLIST = {}

        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            _mk(tmp, {
                "layanan/perlengkapan/src/a.rs": 'fn f(r: &str) -> bool { matches!(r, "admin" | "superadmin") }\n',
                "layanan/perlengkapan/src/b.rs": 'fn f(r: &str) -> bool { r == "admin" }\n',
                "layanan/perlengkapan/src/c.rs": 'const L: &[&str] = &["admin", "admin_pusat"];\n',
                "layanan/perlengkapan/src/d.rs": 'fn f(r: &str) -> bool { r.starts_with("admin_") }\n',
                "layanan/perlengkapan/src/ok_waiver.rs": 'fn f(r: &str) -> bool { r == "admin" } // guard:allow-role-literal label\n',
                "layanan/perlengkapan/src/ok_test.rs": 'fn t() { let _ = ["admin", "admin_pusat"]; }\n',
                "layanan/perlengkapan/src/ok_mod.rs": '#[cfg(test)]\nmod tests { fn t() { let _ = "admin" == "admin"; } }\n',
                "layanan/perlengkapan/src/ok_single.rs": 'fn f(r: &str) -> &str { match r { "admin" => "Administrator", _ => r } }\n',
                "lib/core/src/authz.rs": 'pub const A: &[&str] = &["admin", "superadmin", "admin_pusat"];\n',
            })
            probs = "\n".join(check_literals(tmp))
            for f in ("a.rs", "b.rs", "c.rs", "d.rs"):
                expect(f"src/{f}" in probs, f"peran-literal di {f} HARUS merah (R5)")
            for f in ("ok_waiver.rs", "ok_test.rs", "ok_mod.rs", "ok_single.rs", "authz.rs"):
                expect(f"src/{f}" not in probs, f"{f} tidak boleh merah (waiver/tes/sumber kebenaran/label)")
    finally:
        WRITE_ALLOWLIST, READ_ALLOWLIST = saved

    if failures:
        print("self-test GAGAL:")
        for f in failures:
            print("  -", f)
        return 1
    print("self-test lulus (aturan R1-R6 terbukti bisa merah dan bisa hijau)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    problems = run(ROOT)
    if problems:
        print(f"check-authz-guards: {len(problems)} temuan\n")
        for p in problems:
            print("  ✗", p)
        print(
            "\nPerbaiki dengan gerbang sungguhan. Allowlist (WRITE_/READ_ALLOWLIST di skrip ini) "
            "hanya untuk endpoint swalayan/referensi dan wajib beralasan."
        )
        return 1
    print("check-authz-guards: rute perlengkapan bergerbang & tak ada daftar peran-literal di luar lib_core::authz.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
