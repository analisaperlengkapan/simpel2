-- ============================================================================
-- Migration 005: drop role_permissions for the removed mapping_kodefikasi (#113)
-- ============================================================================
-- `002_seed.sql` grants two roles permission on resource 'mapping_kodefikasi':
--   - validator_pusat  'manage' {read,approve,reject}   scope pusat  (:741)
--   - admin            'admin'  {create,read,...}       scope all    (:753)
--
-- The perlengkapan module that resource named is deleted in this same change
-- (#113 — it was dead on arrival: it joined a `perlengkapan.ms_barang` table no
-- migration creates, and selected a `nama` column that does not exist on
-- authenc.satkers). Its six routes are unmounted and its table is dropped by
-- perlengkapan V005.
--
-- These rows are inert once the routes are gone — nothing checks them. They are
-- removed anyway because a grant naming a resource that no longer exists reads
-- like a live capability to anyone auditing RBAC, and an admin UI enumerating
-- permissions would happily list "mapping_kodefikasi" as something the system
-- can do. Deleting them keeps the permission table an honest description of the
-- surface that actually exists.
--
-- Targeted by primary key rather than by `resource = '...'` so this cannot
-- delete a future unrelated row that happens to reuse the name.
-- ============================================================================

DELETE FROM authenc.role_permissions
WHERE id IN (
    'c81a011d-1945-4d3d-a539-c9f9272d62f6',  -- validator_pusat / manage
    '1ed3c8af-c1f4-460d-9f52-54fc69e30914'   -- admin / admin
);
