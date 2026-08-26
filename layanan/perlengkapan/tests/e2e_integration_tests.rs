mod common;

#[cfg(test)]
mod integration {
    mod dashboard_metrics_test;
    mod kebutuhan_bmn_workflow_test;
    mod pakaian_dinas_ukuran_test;
    mod pakaian_dinas_workflow_test;
    mod pemakaian_asset_identity_test;
    mod pemakaian_bmn_workflow_test;
    mod pemakaian_monitoring_scope_test;
    mod penghapusan_bmn_workflow_test;
    mod rbac_403_test;
    mod satker_wilayah_test;
    mod siman_dead_columns_test;
    mod sql_relations_exist_test;
}
