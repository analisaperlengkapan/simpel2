-- The workflow status master was missing codes the enums use, and one state had no code at all.
--
-- `ms_workflow_status` is where the dashboard resolves a status code to the label it
-- prints. Where a code has no row, the query falls back to "Kode NNNN"; where the row has
-- no code, to the raw status text. Measured on staging (rc32), every one of these was
-- on screen or one row away from it:
--
--   pemakaian_bmn   3010 Menunggu Approver Satker   the satker approval chain (#96): no row
--   pemakaian_bmn   3011 Revisi Operator            ditto
--   penghapusan_bmn 4009 Dibatalkan                 NEW code: cancelling a usulan only changed the
--                                                   text `status` to CANCELLED and left
--                                                   `status_kode` at 4000, so a cancelled usulan was
--                                                   counted as a Draft ("Draft 9" included 6 of
--                                                   them) and the list printed the raw word
--   penghapusan_bmn 4010 Konsep SK Digenerate (Wilayah)   the Wilayah authority branch (V029): no row,
--   penghapusan_bmn 4011 SK Ditandatangani (Wilayah)      so every such usulan would print "Kode 4010"
--
-- Names are the enums' own `label()`. `ON CONFLICT (kode) DO NOTHING` because the primary
-- key is `kode` alone (the code ranges per module do not overlap) and this must be safe to
-- replay: the runner re-applies every migration's effect on a database that already has it.
INSERT INTO perlengkapan.ms_workflow_status (kode, modul, nama, deskripsi, urutan, is_terminal)
VALUES
  (3010, 'pemakaian_bmn',   'Menunggu Approver Satker',        'Diteruskan Validator Satker, menunggu Approver Satker', 9,  false),
  (3011, 'pemakaian_bmn',   'Revisi Operator',                 'Dikembalikan ke Operator untuk direvisi',                10, false),
  (4009, 'penghapusan_bmn', 'Dibatalkan',                      'Usulan dibatalkan oleh operator',                        10, true),
  (4010, 'penghapusan_bmn', 'Konsep SK Digenerate (Wilayah)',  'Validator Wilayah mengenerate konsep SK (kewenangan Wilayah)', 11, false),
  (4011, 'penghapusan_bmn', 'SK Ditandatangani (Wilayah)',     'SK Wilayah ditandatangani Kepala Kejati dan PDF diunggah', 12, false)
ON CONFLICT (kode) DO NOTHING;

-- Rows cancelled before the code existed still say 4000. Only the cancelled ones:
-- a usulan that is really a Draft keeps 4000.
UPDATE perlengkapan.penghapusan_bmn
   SET status_kode = 4009
 WHERE status = 'CANCELLED'
   AND status_kode IS DISTINCT FROM 4009;
