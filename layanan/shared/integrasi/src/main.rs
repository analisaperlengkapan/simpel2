/// Main binary untuk SIMPelv2 Layanan Integrasi
/// Menggunakan unified batch processing dengan storage strategy yang fleksibel
/// Menangani integrasi dengan external APIs: MonSAKTI, MySIMKARI, dll.
use simpelv2_integrasi::{
    fetch_all_data, fetch_all_satker, fetch_global_references, fetch_satker_complete,
    fetch_satker_parallel, get_satker_list, storage_from_env, Config, MonsaktiClient,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("=== SIMPelv2 Layanan Integrasi ===");

    // Load configuration
    let config = Config::from_env()?;
    let storage = storage_from_env();
    let mut client = MonsaktiClient::new(config).await?;

    info!("Storage strategy: {:?}", storage);

    // Get KL code from environment or use default
    let kode_kl = std::env::var("KODE_KL").unwrap_or_else(|_| "006".to_string());
    info!("Kode KL: {}", kode_kl);

    // Check processing mode
    let mode = std::env::var("MODE").unwrap_or_else(|_| "complete".to_string());
    let use_parallel = std::env::var("USE_PARALLEL")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if use_parallel {
        info!("⚠️  Parallel mode enabled (may cause rate limiting)");
    }

    match mode.as_str() {
        "complete" => {
            // Fetch everything: global refs + MySIMKARI + all satker
            info!("Mode: COMPLETE (global + MySIMKARI + all satker)");
            fetch_all_data(&mut client, &storage, &kode_kl).await?;
        }
        "satker" => {
            // Fetch all satker only
            info!("Mode: SATKER (all satker for KL{})", kode_kl);
            fetch_all_satker(&mut client, &storage, &kode_kl).await?;
        }
        "global" => {
            // Fetch global references only
            info!("Mode: GLOBAL (reference data only)");
            fetch_global_references(&mut client, &storage, &kode_kl).await?;
        }
        "single" => {
            // Fetch single satker (from TEST_SATKER env var)
            let kdsatker = std::env::var("TEST_SATKER").unwrap_or_else(|_| {
                eprintln!("ERROR: TEST_SATKER environment variable not set");
                std::process::exit(1);
            });

            info!("Mode: SINGLE (satker {})", kdsatker);

            if use_parallel {
                fetch_satker_parallel(&mut client, &storage, &kode_kl, &kdsatker).await?;
            } else {
                fetch_satker_complete(&mut client, &storage, &kode_kl, &kdsatker).await?;
            }
        }
        "list" => {
            // Just list satker, don't fetch
            info!("Mode: LIST (list satker only)");
            let satker_list = get_satker_list(&mut client, &kode_kl).await?;
            info!("Found {} satker(s):", satker_list.len());
            for (idx, kdsatker) in satker_list.iter().enumerate() {
                println!("{}. {}", idx + 1, kdsatker);
            }
        }
        _ => {
            eprintln!("ERROR: Invalid MODE: {}", mode);
            eprintln!("Valid modes: complete, satker, global, single, list");
            std::process::exit(1);
        }
    }

    info!("=== Processing Complete ===");
    Ok(())
}
