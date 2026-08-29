mod common;

#[cfg(test)]
mod integration {
    mod dashboard_metrics_test;
    mod dashboard_scope_test;
    mod kebutuhan_analisis_siman_test;
    mod kebutuhan_bmn_scope_test;
    mod kebutuhan_bmn_workflow_test;
    mod kebutuhan_laporan_status_label_test;
    mod kebutuhan_search_test;
    mod pakaian_dinas_campaign_scope_test;
    mod pakaian_dinas_pegawai_scope_test;
    mod pakaian_dinas_ukuran_test;
    mod pakaian_dinas_workflow_test;
    mod pegawai_satker_link_test;
    mod pemakaian_asset_identity_test;
    mod pemakaian_bmn_workflow_test;
    mod pemakaian_detail_scope_test;
    mod pemakaian_monitoring_scope_test;
    mod penghapusan_bmn_workflow_test;
    mod rbac_403_test;
    mod satker_wilayah_test;
    mod siman_dead_columns_test;
    mod sql_relations_exist_test;
}
