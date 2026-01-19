use std::time::Instant;
use serde_json::json;
use layanan_integrasi::db::json_to_sql_param;
use tokio_postgres::types::{Type, ToSql};
use bytes::BytesMut;

fn main() {
    let iterations = 100_000;
    println!("Benchmarking json_to_sql_param + to_sql with {} iterations...", iterations);

    // Test data
    let num_val = json!(12345.6789);
    // Large array to make allocation cost significant
    let arr_val = json!([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
        {"foo": "bar", "baz": "qux"},
        "some string",
        [1, 2, 3]
    ]);
    let obj_val = json!({
        "key": "value",
        "nested": [1, 2, 3, 4, 5],
        "more": "data",
        "even_more": {"a": 1, "b": 2}
    });

    let mut buffer = BytesMut::with_capacity(4096);

    // 1. Benchmark Number -> String (TEXT column)
    // We simulate a column that is NOT api_id, so it goes to "OwnedString" currently.
    let start = Instant::now();
    for _ in 0..iterations {
        let param = json_to_sql_param(&num_val, "some_number_column");
        buffer.clear();
        param.to_sql(&Type::TEXT, &mut buffer).unwrap();
    }
    let duration_num = start.elapsed();
    println!("Number -> TEXT: {:.2?}", duration_num);

    // 2. Benchmark Array -> String (TEXT column)
    // Column name doesn't end in _data or _json, so it goes to "OwnedJsonString" currently.
    let start = Instant::now();
    for _ in 0..iterations {
        let param = json_to_sql_param(&arr_val, "some_array_column");
        buffer.clear();
        param.to_sql(&Type::TEXT, &mut buffer).unwrap();
    }
    let duration_arr = start.elapsed();
    println!("Array -> TEXT:  {:.2?}", duration_arr);

    // 3. Benchmark Object -> String (TEXT column)
    let start = Instant::now();
    for _ in 0..iterations {
        let param = json_to_sql_param(&obj_val, "some_object_column");
        buffer.clear();
        param.to_sql(&Type::TEXT, &mut buffer).unwrap();
    }
    let duration_obj = start.elapsed();
    println!("Object -> TEXT: {:.2?}", duration_obj);

    println!("Total time: {:.2?}", duration_num + duration_arr + duration_obj);
}
