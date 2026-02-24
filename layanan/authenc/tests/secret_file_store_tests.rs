//! Tests for FileVault (file-based vault provider)
//! NOTE: FileVault has been migrated to Secreton. These tests are disabled.

extern crate authenc;

// FileVault no longer exists in authenc::secreton_client
// use authenc::secreton_client::{Vault, file_vault::FileVault};
// use std::fs;

#[tokio::test]
#[ignore = "FileVault migrated to Secreton"]
async fn test_file_vault_get_secret() {
    // NOTE: This test is disabled as FileVault has been migrated to Secreton
    // TODO: Create equivalent tests in layanan/secreton
    /*
    // Setup: create a temp directory and secret file
    let temp_dir = tempfile::tempdir().unwrap();
    let secret_path = temp_dir.path().join("myrealm").join("mysecret");
    fs::create_dir_all(secret_path.parent().unwrap()).unwrap();
    fs::write(&secret_path, "supersecret").unwrap();

    let vault = FileVault::new(temp_dir.path());
    let secret = vault.get_secret("mysecret", Some("myrealm")).await;
    assert!(secret.is_some());
    assert_eq!(secret.unwrap().value, "supersecret");
    */
}
