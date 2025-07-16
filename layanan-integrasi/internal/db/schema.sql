-- Tabel master satker dari MySimkari
CREATE TABLE IF NOT EXISTS mysimkari_satker (
    id TEXT PRIMARY KEY,
    nama_satker TEXT NOT NULL,
    provinsi TEXT,
    wilayah TEXT,
    kategori_satker TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Tabel data pegawai hasil tarikan MySimkari
CREATE TABLE IF NOT EXISTS mysimkari_pegawai (
    nip TEXT PRIMARY KEY,
    nama TEXT NOT NULL,
    email_dinas TEXT,
    golpang TEXT,
    jabatan TEXT,
    jenis_jabatan_terakhir TEXT,
    agama TEXT,
    jenis_kelamin TEXT,
    gol_kd TEXT,
    eselon TEXT,
    foto TEXT,
    satker_id TEXT REFERENCES mysimkari_satker(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

