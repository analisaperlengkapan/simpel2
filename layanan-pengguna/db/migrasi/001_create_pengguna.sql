CREATE SCHEMA IF NOT EXISTS pengguna;

CREATE TABLE pengguna.pengguna (
    id SERIAL PRIMARY KEY,
    nama TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    satker TEXT,
    role TEXT NOT NULL
);

-- Optional seed
INSERT INTO pengguna.pengguna (nama, username, satker, role)
VALUES ('Administrator Utama', 'admin', 'Kejaksaan Agung', 'superadmin')
ON CONFLICT (username) DO NOTHING;
