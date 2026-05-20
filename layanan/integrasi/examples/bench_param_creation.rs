use layanan_integrasi::db::json_to_sql_param;
use serde_json::{Value, json};
use std::time::Instant;

fn main() {
    let size = 100_000;
    println!("Preparing {} records...", size);

    // Create sample data
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        data.push(json!({
            "id": format!("00000000-0000-0000-0000-{:012x}", i),
            "nama": format!("Name {}", i),
            "description": "Some long description text that might be cloned repeatedly if we are not careful. ".repeat(5),
            "amount": i,
            "is_active": i % 2 == 0,
            "details": {
                "field1": "value1",
                "field2": i
            }
        }));
    }

    let columns = vec![
        "id",
        "nama",
        "description",
        "amount",
        "is_active",
        "details",
    ];

    println!("Starting benchmark (json_to_sql_param allocation)...");
    let start = Instant::now();

    let mut param_count = 0;

    for item in &data {
        if let Some(obj) = item.as_object() {
            // Simulate the loop in bulk_insert_postgres
            let mut params = Vec::new();
            for col in &columns {
                let value = obj.get(*col).unwrap_or(&Value::Null);
                // Call the function
                params.push(json_to_sql_param(value, col));
            }
            param_count += params.len();
            // In the real code, these would be used in a query then dropped.
            // We drop them here at the end of iteration.
        }
    }

    let duration = start.elapsed();
    println!("Processed {} params in {:.2?}", param_count, duration);
    println!("Time per record: {:.2?}", duration / size as u32);
}
