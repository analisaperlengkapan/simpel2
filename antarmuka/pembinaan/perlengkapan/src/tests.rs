#[cfg(test)]
mod tests {

    use crate::api::{Asset, DashboardStats};
    use serde_json::json;

    #[test]
    fn test_aset_deserialization() {
        let json_data = json!({
            "id": "123",
            "nama_aset": "Laptop",
            "kategori_aset": "Elektronik",
            "kode_barang": "101.1",
            "merk": "Dell",
            "no_aset": "1",
            "kondisi": "baik",
            "lokasi": "Gudang",
            "nilai_perolehan": 1000.0,
            "tgl_perolehan": "2023-01-01",
            "updated_at": "2023-01-01T00:00:00Z"
        });

        let aset: Asset = serde_json::from_value(json_data).unwrap();

        assert_eq!(aset.nama_aset, Some("Laptop".to_string()));
        assert_eq!(aset.kode_barang, Some("101.1".to_string()));
        assert_eq!(aset.nilai_perolehan, Some(1000.0));
    }

    #[test]
    fn test_dashboard_stats_deserialization() {
        let json_data = json!({
            "total_aset": 10,
            "total_nilai_aset": 1000000.0,
            "total_satker": 5,
            "aset_baik": 8,
            "aset_rusak": 2,
            "categories": []
        });

        let stats: DashboardStats = serde_json::from_value(json_data).unwrap();
        assert_eq!(stats.total_aset, 10);
        assert_eq!(stats.aset_baik, 8);
    }
}
