mod common;

#[cfg(test)]
mod integration {
    mod audit_trail_test;
    mod bank_aset_dashboard_breakdown_test;
    mod bank_aset_filter_options_test;
    mod dashboard_metrics_test;
    mod dashboard_scope_test;
    mod kebutuhan_analisis_siman_test;
    mod kebutuhan_bmn_scope_test;
    mod kebutuhan_bmn_workflow_test;
    mod kebutuhan_laporan_status_label_test;
    mod kebutuhan_satker_audit_test;
    mod kebutuhan_search_test;
    mod kodefikasi_barang_test;
    mod migration_runner_test;
    mod pakaian_dinas_campaign_scope_test;
    mod pakaian_dinas_hapus_jenis_test;
    mod pakaian_dinas_hapus_kampanye_test;
    mod pakaian_dinas_laporan_cakupan_test;
    mod pakaian_dinas_pegawai_scope_test;
    mod pakaian_dinas_pengisian_test;
    mod pakaian_dinas_revisi_test;
    mod pakaian_dinas_spesifikasi_filter_test;
    mod pakaian_dinas_ukuran_test;
    mod pakaian_dinas_workflow_test;
    mod pegawai_satker_link_test;
    mod pemakaian_asset_identity_test;
    mod pemakaian_bmn_workflow_test;
    mod pemakaian_detail_scope_test;
    mod pemakaian_header_detail_test;
    mod pemakaian_holder_from_nip_test;
    mod pemakaian_monitoring_scope_test;
    mod pemakaian_sequential_booking_test;
    mod penghapusan_bmn_workflow_test;
    mod penghapusan_items_from_siman_test;
    mod rbac_403_test;
    mod rbac_bfla_test;
    mod satker_wilayah_test;
    mod siman_dead_columns_test;
    mod sql_relations_exist_test;
    mod workflow_status_vocabulary_test;
}
