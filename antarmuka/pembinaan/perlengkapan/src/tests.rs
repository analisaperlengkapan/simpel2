#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Aset, DashboardStats};
    use serde_json::json;

    #[test]
    fn test_aset_deserialization() {
        let json_data = json!({
            "id": "123",
            "nama": "Laptop",
            "kategori": "Elektronik",
            "kode_bmn": "101.1",
            "merk": "Dell",
            "nup": "1",
            "kondisi": "baik",
            "lokasi": "Gudang",
            "nilai_perolehan": 1000.0,
            "tanggal_perolehan": "2023-01-01",
            "status": "aktif",
            "keterangan": "Test"
        });

        let aset: Aset = serde_json::from_value(json_data).unwrap();

        assert_eq!(aset.nama, "Laptop");
        assert_eq!(aset.kode_bmn, "101.1");
        assert_eq!(aset.nilai_perolehan, Some(1000.0));
    }

    #[test]
    fn test_dashboard_stats_deserialization() {
        let json_data = json!({
            "total_aset": 10,
            "total_pengadaan": 5,
            "total_analisis": 2,
            "aset_aktif": 8,
            "pengadaan_berjalan": 3,
            "analisis_pending": 1
        });

        let stats: DashboardStats = serde_json::from_value(json_data).unwrap();
        assert_eq!(stats.total_aset, 10);
    }
}
