/// Example: Fetch data dari MonSAKTI + MySIMKARI dengan storage strategy yang fleksibel
/// Fitur:
/// - Auto-retry dengan token reset ketika token expired
/// - Flexible storage: Database, JSON, CSV, atau Both
/// - Optimized untuk KL006 (Kejaksaan RI)
/// - Batch processing untuk semua satker
/// - Support MySIMKARI integration
/// Setup:
/// 1. Copy .env.example ke .env
/// 2. Isi DATABASE_URL (jika menggunakan database)
/// 3. Isi token untuk setiap modul (ADM, ANG, BEN, dll)
/// 4. Isi MYSIMKARI_TOKEN
/// 5. Set STORAGE_TYPE (database/json/csv/both)
/// 6. Jalankan migrations (jika menggunakan database)
/// Usage:
/// ```bash
/// # Fetch ke database (default)
/// cargo run --example fetch_to_database
/// # Fetch ke JSON files
/// STORAGE_TYPE=json cargo run --example fetch_to_database
/// # Fetch ke database dan JSON
/// STORAGE_TYPE=both cargo run --example fetch_to_database
/// # Test dengan satu satker
/// TEST_SATKER=123456 cargo run --example fetch_to_database
/// # Hanya MySIMKARI
/// FETCH_MODE=mysimkari cargo run --example fetch_to_database
/// ```
use layanan_integrasi::{
    Config, KL_KEJAKSAAN, MonsaktiClient, fetch_all_data, fetch_all_satker, fetch_mysimkari,
    fetch_satker_complete, storage_from_env,
};
use tracing::{Level, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("=== MonSAKTI + MySIMKARI Data Fetcher ===");

    // Load konfigurasi
    let config = Config::from_env()?;
    let storage = storage_from_env();

    info!("Storage strategy: {:?}", storage);

    // Validasi database jika diperlukan
    if matches!(
        storage,
        layanan_integrasi::StorageStrategy::Database
            | layanan_integrasi::StorageStrategy::Both { .. }
    ) && config.db_config.is_none()
    {
        eprintln!("ERROR: DATABASE_URL tidak dikonfigurasi!");
        eprintln!("Silakan set DATABASE_URL di file .env");
        eprintln!("Atau gunakan STORAGE_TYPE=json untuk simpan ke file");
        std::process::exit(1);
    }

    // Buat client
    let mut client = MonsaktiClient::new(config).await?;
    info!("Client berhasil dibuat");
    info!("Kode KL: {}", KL_KEJAKSAAN);

    // Cek mode operasi
    let test_satker = std::env::var("TEST_SATKER").ok();
    let fetch_mode = std::env::var("FETCH_MODE").unwrap_or_else(|_| "all".to_string());

    match fetch_mode.as_str() {
        "mysimkari" => {
            info!("Mode: MySIMKARI only");
            fetch_mysimkari(&mut client, &storage).await?;
        }
        "monsakti" => {
            if let Some(kdsatker) = test_satker {
                info!("Mode: MonSAKTI testing - satker {}", kdsatker);
                fetch_satker_complete(&mut client, &storage, KL_KEJAKSAAN, &kdsatker).await?;
            } else {
                info!(
                    "Mode: MonSAKTI production - semua satker KL{}",
                    KL_KEJAKSAAN
                );
                fetch_all_satker(&mut client, &storage, KL_KEJAKSAAN).await?;
            }
        }
        "all" | _ => {
            if let Some(kdsatker) = test_satker {
                info!("Mode: Testing - MySIMKARI + satker {}", kdsatker);
                fetch_mysimkari(&mut client, &storage).await?;
                fetch_satker_complete(&mut client, &storage, KL_KEJAKSAAN, &kdsatker).await?;
            } else {
                info!(
                    "Mode: Production - MySIMKARI + semua satker KL{}",
                    KL_KEJAKSAAN
                );
                fetch_all_data(&mut client, &storage, KL_KEJAKSAAN).await?;
            }
        }
    }

    info!("=== Selesai ===");
    Ok(())
}
