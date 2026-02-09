use deadpool_postgres::Pool;
use tracing::info;

pub async fn init_db(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "
        CREATE TABLE IF NOT EXISTS perencanaan_pengadaan (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            nama_kegiatan TEXT NOT NULL,
            kode_rekening TEXT NOT NULL,
            pagu_anggaran BIGINT NOT NULL,
            tanggal_mulai DATE NOT NULL,
            tanggal_selesai DATE NOT NULL,
            status TEXT NOT NULL DEFAULT 'Draft',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
    ";

    client.execute(query, &[]).await?;
    info!("Database initialized: perencanaan_pengadaan table checked/created.");

    Ok(())
}
