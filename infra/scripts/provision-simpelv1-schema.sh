#!/usr/bin/env bash
# Provision STRUKTUR (bukan data) database SIMPel v1 di STAGING.
#
# Mengapa skrip ini ada
# ---------------------
# `dbsimpelv1` di staging dibuat KOSONG oleh postgres-init (0 tabel). v1 (Laravel)
# tak punya migrasi di repo — satu-satunya sumber skemanya adalah dump produksi
# `monolith/simpelv1/database/dbsimpelv1.sql.gz` (182 MB, 259 tabel, ±5,1 juta
# baris data nyata). Akibatnya setiap halaman v1 di staging menjawab error
# "relation ... does not exist".
#
# Aturan yang TIDAK boleh dilanggar: staging memakai data MOCK. Dump itu berasal
# dari produksi dan berisi data nyata (pegawai, aset, kontrak) — ia TIDAK BOLEH
# masuk ke staging. Maka skrip ini menyaring dump menjadi DDL saja SEBELUM apa pun
# menyentuh database, lalu MEMVERIFIKASI bahwa hasil saringan tak mengandung satu
# pun baris data; bila ada, ia berhenti dan tidak menulis apa-apa.
#
# Yang dibuang dari dump:
#   - blok `COPY ... FROM stdin; ... \.`  → data
#   - `INSERT INTO ...`                   → data (bila dump format insert)
#   - `SELECT pg_catalog.setval(...)`     → nilai sequence produksi (turunan data)
#   - `\restrict` / `\unrestrict`         → meta-perintah psql dari pg_dump ≥17.6
#   - `SET transaction_timeout`           → hanya dikenal PG17; staging PG16
#   - `ALTER ... OWNER TO ...`            → role `postgres` tak ada di staging
#   - `CREATE SCHEMA public` / `ALTER SCHEMA public ...` → sudah ada dari initdb
#
# Pemakaian
# ---------
#   infra/scripts/provision-simpelv1-schema.sh [DUMP.sql.gz]            # dry-run
#   infra/scripts/provision-simpelv1-schema.sh [DUMP.sql.gz] --apply    # terapkan
#
#   DUMP default : monolith/simpelv1/database/dbsimpelv1.sql.gz
#   NAMESPACE    : simpelv2-staging (satu-satunya yang diterima; lihat di bawah)
#   POD          : postgres-0
#
# Dry-run menulis DDL hasil saringan ke $OUT (default: berkas sementara) dan
# mencetak ringkasan. `--apply` menjalankannya dalam SATU transaksi dengan
# ON_ERROR_STOP, dan menolak bila `dbsimpelv1` sudah berisi tabel (tidak menimpa).
#
# Fungsi utama ada di `filter_schema` supaya dapat diuji tanpa cluster:
#   infra/scripts/provision-simpelv1-schema.sh --self-test
set -euo pipefail

NAMESPACE="${NAMESPACE:-simpelv2-staging}"
POD="${POD:-postgres-0}"
DB="${DB:-dbsimpelv1}"

# Stream filter: DDL masuk ke stdin, DDL bersih keluar ke stdout.
filter_schema() {
  awk '
    # Di dalam blok COPY: buang semuanya sampai terminator `\.`.
    in_copy { if ($0 == "\\.") in_copy = 0; next }
    # Daftar kolom boleh memuat `;` (dump produksi punya tabel ber-kolom
    # "KODE_KEMENTERIAN"";""KDUAKPB"" ...), jadi jangan melarang `;` di tengah.
    /^COPY .* FROM stdin;$/ { in_copy = 1; next }

    /^\\(un)?restrict( |$)/ { next }
    /^SET transaction_timeout/ { next }
    /^INSERT INTO / { data_dropped++; next }
    /^SELECT pg_catalog\.setval\(/ { next }
    /^ALTER [A-Z ]+ .* OWNER TO / { next }
    /^CREATE SCHEMA public;$/ { next }
    /^ALTER SCHEMA public / { next }
    /^-- \*not\* creating schema, since initdb creates it$/ { next }
    { print }
  '
}

# Pagar: keluaran filter_schema TIDAK boleh mengandung data. Mengembalikan 0 bila bersih.
assert_no_data() {
  local file="$1" bad
  bad=$(grep -nE '^(COPY |INSERT INTO |\\\.$|SELECT pg_catalog\.setval)' "$file" | head -5 || true)
  if [ -n "$bad" ]; then
    echo "DITOLAK: hasil saringan masih mengandung data:" >&2
    echo "$bad" >&2
    return 1
  fi
}

self_test() {
  local tmp out
  tmp=$(mktemp -d)
  # Ekspansi $tmp SEKARANG: trap RETURN berjalan setelah `local tmp` keluar dari
  # cakupan, dan di bawah `set -u` itu "tmp: unbound variable".
  trap "rm -rf '$tmp'" RETURN
  cat >"$tmp/in.sql" <<'SQL'
\restrict abc123
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
CREATE SCHEMA mon_sakti;
ALTER SCHEMA mon_sakti OWNER TO postgres;
-- *not* creating schema, since initdb creates it
ALTER SCHEMA public OWNER TO postgres;
CREATE TABLE public.pegawai (nip text NOT NULL, nama text);
ALTER TABLE public.pegawai OWNER TO postgres;
COPY public.pegawai (nip, nama) FROM stdin;
199001012020011001	Nama Nyata
ALTER TABLE ini bukan DDL tapi isi sebuah baris data
\.
INSERT INTO public.pegawai VALUES ('1', 'x');
SELECT pg_catalog.setval('public.pegawai_id_seq', 146284, true);
CREATE TABLE public.sesudah_copy (id integer);
COPY public."impor" ("KODE"";""KDUAKPB"";") FROM stdin;
baris;dengan;titik-koma
\.
CREATE TABLE public.sesudah_copy_aneh (id integer);
\unrestrict abc123
SQL
  out="$tmp/out.sql"
  filter_schema <"$tmp/in.sql" >"$out"

  # 1. DDL yang sah bertahan — termasuk yang berada TEPAT setelah blok COPY.
  for want in 'CREATE SCHEMA mon_sakti;' 'CREATE TABLE public.pegawai' \
              'CREATE TABLE public.sesudah_copy' 'CREATE TABLE public.sesudah_copy_aneh' \
              "SET client_encoding = 'UTF8';"; do
    grep -qF "$want" "$out" || { echo "SELF-TEST GAGAL: hilang '$want'" >&2; return 1; }
  done
  # 2. Yang harus dibuang, hilang — termasuk baris di dalam COPY yang MENYERUPAI DDL.
  for gone in 'restrict' 'transaction_timeout' 'OWNER TO' 'Nama Nyata' \
              'bukan DDL' 'INSERT INTO' 'setval' 'COPY ' 'titik-koma'; do
    if grep -qF "$gone" "$out"; then echo "SELF-TEST GAGAL: masih ada '$gone'" >&2; return 1; fi
  done
  assert_no_data "$out"

  # 3. Arah sebaliknya: pagar HARUS menolak keluaran yang mengandung data.
  printf 'COPY public.x (a) FROM stdin;\n1\n\\.\n' >"$tmp/dirty.sql"
  if assert_no_data "$tmp/dirty.sql" 2>/dev/null; then
    echo "SELF-TEST GAGAL: pagar meloloskan data" >&2; return 1
  fi
  echo "self-test OK (saringan membuang data, mempertahankan DDL, pagar menolak data)"
}

main() {
  local dump="" apply=0 arg
  for arg in "$@"; do
    case "$arg" in
      --self-test) self_test; return ;;
      --apply) apply=1 ;;
      -h|--help) sed -n '2,45p' "$0"; return ;;
      *) dump="$arg" ;;
    esac
  done

  local root
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  dump="${dump:-$root/monolith/simpelv1/database/dbsimpelv1.sql.gz}"
  [ -r "$dump" ] || { echo "dump tak terbaca: $dump" >&2; return 2; }

  # Produksi memegang data v1 NYATA; skrip ini hanya untuk staging.
  case "$NAMESPACE" in
    *staging*) ;;
    *) echo "DITOLAK: NAMESPACE=$NAMESPACE bukan staging. Skrip ini hanya untuk staging." >&2; return 2 ;;
  esac

  local out="${OUT:-$(mktemp -t simpelv1-schema.XXXXXX.sql)}"
  case "$dump" in
    *.gz) gzip -dc "$dump" ;;
    *)    cat "$dump" ;;
  esac | filter_schema >"$out"
  assert_no_data "$out" || { rm -f "$out"; return 1; }

  local tables
  tables=$(grep -c '^CREATE TABLE ' "$out" || true)
  echo "DDL tersaring: $out ($(wc -l <"$out") baris, $tables tabel, 0 baris data)"
  [ "$tables" -gt 0 ] || { echo "DITOLAK: tak satu pun CREATE TABLE — dump salah?" >&2; return 1; }

  if [ "$apply" -ne 1 ]; then
    echo "dry-run — tambahkan --apply untuk menerapkan ke $NAMESPACE/$POD ($DB)"
    return
  fi

  local existing
  existing=$(kubectl -n "$NAMESPACE" exec "$POD" -- sh -c \
    "psql -U \"\$POSTGRES_USER\" -d $DB -Atc \"select count(*) from information_schema.tables where table_schema not in ('pg_catalog','information_schema')\"")
  if [ "$existing" != "0" ]; then
    echo "DITOLAK: $DB sudah berisi $existing tabel — tidak menimpa." >&2
    return 1
  fi

  kubectl -n "$NAMESPACE" exec -i "$POD" -- sh -c \
    "psql -U \"\$POSTGRES_USER\" -d $DB -v ON_ERROR_STOP=1 --single-transaction -q" <"$out"
  echo "diterapkan: $tables tabel di $NAMESPACE/$POD ($DB), tanpa data."
}

main "$@"
