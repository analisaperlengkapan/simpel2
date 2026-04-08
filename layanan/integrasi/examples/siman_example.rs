use layanan_integrasi::siman::{
    SimanAssetCategory, fetch_all_aset_paginated, get_aset_alat_persenjataan,
    get_aset_angkutan_bermotor, get_aset_by_category, get_aset_khusus_tik, get_row_count,
};
use layanan_integrasi::{Config, MonsaktiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration from .env
    let config = Config::from_env()?;

    // Create client
    let mut client: MonsaktiClient = MonsaktiClient::new(config).await?;

    println!("=== SIMAN API v2.0 Example ===\n");

    // Example 1: Get row count for a specific asset category
    println!("1. Getting row count for Tanah (Land Assets)...");
    let count: i64 = get_row_count(&mut client, SimanAssetCategory::Tanah, None).await?;
    println!("   Total Tanah records: {}\n", count);

    // Example 2: Fetch specific range of assets
    println!("2. Fetching first 10 Tanah records...");
    let tanah_data: Vec<serde_json::Value> =
        get_aset_by_category(&mut client, SimanAssetCategory::Tanah, 1, 10, None).await?;
    println!("   Retrieved {} records", tanah_data.len());

    if let Some(first) = tanah_data.first() {
        println!("   Sample record: {}", serde_json::to_string_pretty(first)?);
    }
    println!();

    // Example 3: Fetch all assets for a category with automatic pagination
    println!("3. Fetching all Gedung Bangunan with auto pagination...");
    let gedung_data: Vec<serde_json::Value> = fetch_all_aset_paginated(
        &mut client,
        SimanAssetCategory::GedungBangunan,
        1000, // chunk size
    )
    .await?;
    println!(
        "   Total Gedung Bangunan retrieved: {}\n",
        gedung_data.len()
    );

    // Example 4: Iterate through all asset categories
    println!("4. Getting row counts for all asset categories:");
    for category in SimanAssetCategory::all() {
        match get_row_count(&mut client, category, None).await {
            Ok(count) => {
                println!("   {:30} : {:>8} records", category.description(), count)
            }
            Err(e) => println!("   {:30} : Error - {}", category.description(), e),
        }
    }
    println!();

    // Example 5: Save data to file
    println!("5. Saving Tanah data to JSON file...");
    let tanah_json = serde_json::to_value(&tanah_data)?;
    client.save_to_json(&tanah_json, "siman_tanah.json").await?;
    println!("   Saved to data/siman_tanah.json\n");

    // Example 6: Save to CSV
    println!("6. Saving Tanah data to CSV file...");
    client.save_to_csv(&tanah_json, "siman_tanah.csv").await?;
    println!("   Saved to data/siman_tanah.csv\n");

    // Example 7: Fetch multiple categories in parallel
    println!("7. Fetching multiple categories (first 100 records each)...");
    let categories = vec![
        SimanAssetCategory::AngkutanBermotor,
        SimanAssetCategory::AlatBesar,
        SimanAssetCategory::Rumah,
    ];

    for category in categories {
        let data: Vec<serde_json::Value> =
            get_aset_by_category(&mut client, category, 1, 100, None).await?;
        println!(
            "   {:30} : {} records fetched",
            category.description(),
            data.len()
        );
    }
    println!();

    // Example 8: Working with specific asset types using convenience functions
    println!("8. Using convenience functions for specific asset types...");

    let kendaraan: Vec<serde_json::Value> = get_aset_angkutan_bermotor(&mut client, 1, 50).await?;
    println!(
        "   Angkutan Bermotor (vehicles): {} records",
        kendaraan.len()
    );

    let senjata: Vec<serde_json::Value> = get_aset_alat_persenjataan(&mut client, 1, 50).await?;
    println!("   Alat Persenjataan (weapons): {} records", senjata.len());

    let tik: Vec<serde_json::Value> = get_aset_khusus_tik(&mut client, 1, 50).await?;
    println!("   Khusus TIK (IT equipment): {} records", tik.len());
    println!();

    println!("=== Example completed successfully! ===");

    Ok(())
}
