-- A permit is born DRAFT with `status_kode` NULL.
--
-- `izin_pemakaian_bmn.status_kode` is the denormalised twin of `status`, written by
-- `sync_status_kode` on every transition (3000 Draft ... 3011 RevisiOperator). The
-- INSERT in `create` set `status = 'DRAFT'` and forgot the twin, so every permit that
-- had not yet been submitted carried NULL: invisible to anything that joins the
-- workflow status master for a label (the dashboard's "Status Izin Pemakaian"
-- printed the raw word DRAFT next to the localised others). Measured on staging
-- (rc32): 3 of 7 permits, all drafts, status_kode NULL; every other row 3004.
--
-- The code now writes it at INSERT. This backfills the rows born before that.
--
-- Only DRAFT: every other state is reached through a transition, which syncs the
-- code, and a row whose code disagrees with its status for any other reason is a
-- different defect that this must not paper over. Idempotent (a second run finds
-- nothing NULL), and it touches no row that already has a code.
UPDATE perlengkapan.izin_pemakaian_bmn
   SET status_kode = 3000
 WHERE status = 'DRAFT'
   AND status_kode IS NULL;
