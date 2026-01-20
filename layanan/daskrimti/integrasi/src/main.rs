/// Main binary untuk SIMPelv2 Layanan Integrasi
/// Menggunakan unified batch processing dengan storage strategy yang fleksibel
/// Menangani integrasi dengan external APIs: MonSAKTI, MySIMKARI, dll.
use clap::{Parser, ValueEnum};
use layanan_integrasi::{
    Config, MonsaktiClient, fetch_all_data, fetch_all_data_with_mysimkari, fetch_all_satker,
    fetch_all_satker_with_modules, fetch_all_satker_with_modules_from_db, fetch_global_references,
    fetch_satker_complete, fetch_satker_parallel, fetch_satker_with_modules, get_satker_list,
    storage_from_env,
};
use tracing::info;

#[derive(Debug, Clone, ValueEnum)]
enum Source {
    /// Semua source (MonSAKTI + MySIMKARI + SIMAN)
    All,
    /// Hanya MonSAKTI
    Monsakti,
    /// Hanya MySIMKARI
    Mysimkari,
    /// Hanya SIMAN
    Siman,
}

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// Fetch semua data (tergantung source: MonSAKTI=global+satker, All=global+MySIMKARI+satker)
    Complete,
    /// Fetch semua satker saja (MonSAKTI per-satker data)
    Satker,
    /// Fetch global references saja (MonSAKTI reference data)
    Global,
    /// Fetch single satker (perlu --test-satker)
    Single,
    /// List satker saja (tidak fetch)
    List,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source API yang akan dijalankan
    #[arg(short, long, value_enum, default_value = "all")]
    source: Source,

    /// Mode processing
    #[arg(short, long, value_enum, default_value = "complete")]
    mode: Mode,

    /// Kode KL (default: 006 untuk Kejaksaan)
    #[arg(short, long, default_value = "006")]
    kode_kl: String,

    /// Kode satker untuk mode single
    #[arg(long)]
    test_satker: Option<String>,

    /// Enable parallel processing (may cause rate limiting)
    #[arg(short, long, default_value = "false")]
    parallel: bool,

    /// SIMAN: Kategori aset spesifik (untuk testing per kategori)
    /// Options: alat-besar, alat-persenjataan, angkutan-bermotor, tak-berwujud,
    ///          tetap-lainnya, tanah, bangunan-air, gedung-bangunan, instalasi-jaringan,
    ///          jalan-jembatan, kdp, khusus-tik, non-tik, rumah, tetap-renovasi
    #[arg(long)]
    siman_category: Option<String>,

    /// MonSAKTI: Modul spesifik (untuk testing per modul)
    /// Options: persediaan, aset-tetap, transaksi, kontrak-pengadaan, data-supplier,
    ///          realisasi-belanja, saldo-anggaran
    #[arg(long)]
    monsakti_module: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args = Args::parse();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("=== SIMPelv2 Layanan Integrasi ===");
    info!("Source: {:?}", args.source);
    info!("Mode: {:?}", args.mode);
    info!("Kode KL: {}", args.kode_kl);

    // Load configuration
    let config = Config::from_env()?;
    let storage = storage_from_env();
    let mut client = MonsaktiClient::new(config).await?;

    info!("Storage strategy: {:?}", storage);

    if args.parallel {
        info!("⚠️  Parallel mode enabled (may cause rate limiting)");
    }

    // Check which source to process
    match args.source {
        Source::Mysimkari => {
            info!(">>> Processing MySIMKARI only");
            process_mysimkari(&mut client, &storage).await?;
        }
        Source::Monsakti => {
            info!(">>> Processing MonSAKTI only");
            process_monsakti(&mut client, &storage, &args).await?;
        }
        Source::Siman => {
            info!(">>> Processing SIMAN only");
            process_siman(&mut client, &storage, args.siman_category.as_deref()).await?;
        }
        Source::All => {
            info!(">>> Processing ALL sources (MonSAKTI + MySIMKARI + SIMAN)");

            // For "All" source with "Complete" mode, use the combined function
            if matches!(args.mode, Mode::Complete) {
                info!("Mode: COMPLETE (all sources: global + MySIMKARI + all satker)");
                if let Err(e) =
                    fetch_all_data_with_mysimkari(&mut client, &storage, &args.kode_kl).await
                {
                    tracing::error!("Complete fetch failed: {}", e);
                }
            } else {
                // Process in sequence for other modes
                info!("Step 1/3: MonSAKTI");
                if let Err(e) = process_monsakti(&mut client, &storage, &args).await {
                    tracing::error!("MonSAKTI failed: {}", e);
                }

                info!("Step 2/3: MySIMKARI");
                if let Err(e) = process_mysimkari(&mut client, &storage).await {
                    tracing::error!("MySIMKARI failed: {}", e);
                }

                info!("Step 3/3: SIMAN");
                if let Err(e) = process_siman(&mut client, &storage, None).await {
                    tracing::error!("SIMAN failed: {}", e);
                }
            }
        }
    }

    info!("=== Processing Complete ===");
    Ok(())
}

/// Process MonSAKTI based on mode
async fn process_monsakti(
    client: &mut MonsaktiClient,
    storage: &layanan_integrasi::StorageStrategy,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_filter = args.monsakti_module.as_deref();

    if let Some(module) = module_filter {
        info!("🎯 Filtering MonSAKTI module: {}", module);
    }

    match args.mode {
        Mode::Complete => {
            info!("Mode: COMPLETE (global + all satker)");
            if module_filter.is_some() {
                // With module filter, skip global refs and go straight to satker data
                fetch_all_satker_with_modules(client, storage, &args.kode_kl, module_filter)
                    .await?;
            } else {
                fetch_all_data(client, storage, &args.kode_kl).await?;
            }
        }
        Mode::Satker => {
            info!("Mode: SATKER (all satker for KL{})", args.kode_kl);
            if module_filter.is_some() {
                // Gunakan fungsi baru yang ambil satker list dari database (lebih efisien)
                fetch_all_satker_with_modules_from_db(
                    client,
                    storage,
                    &args.kode_kl,
                    module_filter,
                )
                .await?;
            } else {
                fetch_all_satker(client, storage, &args.kode_kl).await?;
            }
        }
        Mode::Global => {
            info!("Mode: GLOBAL (reference data only)");
            fetch_global_references(client, storage, &args.kode_kl).await?;
        }
        Mode::Single => {
            let kdsatker = args
                .test_satker
                .as_ref()
                .ok_or("ERROR: --test-satker required for single mode")?;

            info!("Mode: SINGLE (satker {})", kdsatker);

            if args.parallel {
                fetch_satker_parallel(client, storage, &args.kode_kl, kdsatker).await?;
            } else if module_filter.is_some() {
                fetch_satker_with_modules(client, storage, &args.kode_kl, kdsatker, module_filter)
                    .await?;
            } else {
                fetch_satker_complete(client, storage, &args.kode_kl, kdsatker).await?;
            }
        }
        Mode::List => {
            info!("Mode: LIST (list satker only)");
            let satker_list = get_satker_list(client, &args.kode_kl).await?;
            info!("Found {} satker(s):", satker_list.len());
            for (idx, kdsatker) in satker_list.iter().enumerate() {
                println!("{}. {}", idx + 1, kdsatker);
            }
        }
    }
    Ok(())
}

/// Process MySIMKARI
async fn process_mysimkari(
    client: &mut MonsaktiClient,
    storage: &layanan_integrasi::StorageStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    use layanan_integrasi::batch::fetchers::fetch_mysimkari;

    info!("Fetching MySIMKARI data...");
    fetch_mysimkari(client, storage).await?;

    Ok(())
}

/// Process SIMAN - Fetch all asset categories
async fn process_siman(
    client: &mut MonsaktiClient,
    storage: &layanan_integrasi::StorageStrategy,
    category_filter: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use layanan_integrasi::siman::{
        SimanAssetCategory, endpoints::fetch_all_assets_with_pagination,
    };

    info!("========================================");
    info!("🏛️  SIMAN v2.0 - Sistem Informasi Manajemen Aset Negara");
    info!("========================================");

    // All 15 asset categories from official documentation
    let all_categories = vec![
        SimanAssetCategory::AlatBesar,
        SimanAssetCategory::AlatPersenjataan,
        SimanAssetCategory::AngkutanBermotor,
        SimanAssetCategory::TakBerwujud,
        SimanAssetCategory::TetapLainnya,
        SimanAssetCategory::Tanah,
        SimanAssetCategory::BangunanAir,
        SimanAssetCategory::GedungBangunan,
        SimanAssetCategory::InstalasiJaringan,
        SimanAssetCategory::JalandanJembatan,
        SimanAssetCategory::KDP,
        SimanAssetCategory::KhususTIK,
        SimanAssetCategory::NonTIK,
        SimanAssetCategory::Rumah,
        SimanAssetCategory::TetapRenovasi,
    ];

    // Filter categories if specific category requested
    let categories: Vec<SimanAssetCategory> = if let Some(filter) = category_filter {
        info!("🎯 Filtering: Only processing category '{}'", filter);
        all_categories
            .into_iter()
            .filter(|cat| {
                cat.description().to_lowercase().replace(" ", "-") == filter.to_lowercase()
            })
            .collect()
    } else {
        all_categories
    };

    if categories.is_empty() {
        tracing::error!(
            "❌ No matching category found for filter: {:?}",
            category_filter
        );
        return Ok(());
    }

    let mut total_success = 0;
    let mut total_failed = 0;

    for category in categories {
        info!("");
        info!("📦 Processing: {}", category.description());

        match fetch_all_assets_with_pagination(client, storage, category).await {
            Ok((success, failed)) => {
                info!(
                    "✅ {} completed: {} records fetched, {} failed",
                    category.description(),
                    success,
                    failed
                );
                total_success += success;
                total_failed += failed;
            }
            Err(e) => {
                tracing::error!("❌ {} failed: {}", category.description(), e);
                total_failed += 1;
            }
        }
    }

    info!("");
    info!("========================================");
    info!("📊 SIMAN Processing Summary");
    info!("========================================");
    info!("✅ Total records fetched: {}", total_success);
    info!("❌ Total failed: {}", total_failed);
    info!("========================================");

    Ok(())
}
