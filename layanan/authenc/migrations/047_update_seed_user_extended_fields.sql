-- Migration: 047_update_seed_user_extended_fields.sql
-- Purpose: Update seed admin user with extended fields from migration 046
-- Also set require_password_change = true so admin must change NIP-based password

BEGIN;

-- Update seed user with extended fields
UPDATE users SET
    nip = '199203142014031001',
    nama = 'Admin Perlengkapan',
    jabatan = 'Kasubag Perlengkapan',
    satker_code = '0100000',
    require_password_change = true,
    updated_at = NOW()
WHERE username = '199203142014031001';

COMMIT;
