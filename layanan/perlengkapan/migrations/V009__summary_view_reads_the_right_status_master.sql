-- `vw_kebutuhan_bmn_summary` labelled its status from the wrong master.
--
-- Two tables describe this one workflow, and they disagree from code 2004 down:
--
--   kode  KebutuhanBmnStatus (writes the column)   ms_aktivitas_bmn (joined here)
--   2002  SubmitWilayah  "Diajukan ke Validator Wilayah"   SUBMIT_SATKER
--   2004  SubmitPusat    "Diajukan ke Validator Pusat"     ANALISIS_KELAYAKAN
--   2005  AnalisisKelayakan "Analisis Kelayakan"           PENYUSUNAN_PRIORITAS
--
-- `ms_aktivitas_bmn.nama` holds the machine token, so the kebutuhan list showed
-- users `DRAFT` and `SUBMIT_SATKER` verbatim; worse, from 2004 the token names
-- the WRONG step, so a request merely queued for Validator Pusat was announced
-- as already under feasibility analysis. `ms_workflow_status.nama` is the
-- Indonesian label and agrees with the enum the workflow actually obeys.
--
-- Keyed by `modul` as well as `kode`: status codes are unique only within a
-- module, and this view is over `pengajuan_kebutuhan_bmn` alone.
--
-- `ms_aktivitas_bmn` is left in place — the pakaian-dinas queries still read it
-- (through `COALESCE(deskripsi, nama)`, which is why they never showed a raw
-- token), and its `kode` values remain the workflow's activity ids.
--
-- Column names, types and order are unchanged, so CREATE OR REPLACE is legal
-- and no dependent object needs dropping.

CREATE OR REPLACE VIEW perlengkapan.vw_kebutuhan_bmn_summary AS
 SELECT p.id,
    p.nama,
    p.tahun,
    p.status_kode,
    w.nama AS status_nama,
    count(DISTINCT ps.id) AS total_satker,
    count(DISTINCT psb.id) AS total_barang,
    COALESCE(sum(psb.jumlah), (0)::bigint) AS total_jumlah_diminta,
    COALESCE(sum(psb.jml_setuju), (0)::bigint) AS total_jumlah_disetujui,
    p.created_at,
    p.updated_at
   FROM (((perlengkapan.pengajuan_kebutuhan_bmn p
     LEFT JOIN perlengkapan.ms_workflow_status w
       ON ((p.status_kode = w.kode) AND (w.modul = 'kebutuhan_bmn'::character varying)))
     LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON ((p.id = ps.pengajuan_id)))
     LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang psb ON ((ps.id = psb.pengajuan_satker_id)))
  GROUP BY p.id, p.nama, p.tahun, p.status_kode, w.nama, p.created_at, p.updated_at;
