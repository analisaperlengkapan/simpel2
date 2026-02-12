-- Create test user with NIP as username and password
-- Password will be hashed using Argon2id

-- First, let's create a simple user with a known password hash
-- We'll use a pre-computed hash for password "199203142014031001"

INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    enabled,
    email_verified,
    realm_id,
    nip,
    nama,
    jabatan,
    satker_code,
    require_password_change,
    created_at,
    updated_at
) VALUES (
    gen_random_uuid(),
    '199203142014031001',
    '199203142014031001@kejaksaan.go.id',
    -- This is Argon2id hash for password "199203142014031001"
    -- Generated with: m=65536, t=10, p=4
    '$argon2id$v=19$m=65536,t=10,p=4$yy7MPp4/ZZ31p+ykbPuJFg$ykgPhu7dLJXgMOeQfk/YSCcyDciRM74CpvWiad92tr0',
    true,
    true,
    (SELECT id FROM realms WHERE name = 'master' LIMIT 1),
    '199203142014031001',
    'Admin Perlengkapan',
    'Kasubag Perlengkapan',
    'UNKNOWN',
    true,
    NOW(),
    NOW()
)
ON CONFLICT (username) DO UPDATE SET
    password_hash = EXCLUDED.password_hash,
    updated_at = NOW();
