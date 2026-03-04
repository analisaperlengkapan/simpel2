import re

with open("layanan/secreton/crates/backup/tests/property_tests.rs", "r") as f:
    content = f.read()

# Replace invalid syntax with valid Rust struct init
# The broken syntax looks like:
# StorageConfig::Local(LocalStorageConfig {
#     path: temp_dir.path().to_string_lossy().to_string(),
#     Ok(())
content = re.sub(
    r"StorageConfig::Local\(LocalStorageConfig \{\s*path:\s*(.*?),\s*Ok\(\(\)\)",
    r"StorageConfig::Local(LocalStorageConfig { path: \1 })",
    content
)

with open("layanan/secreton/crates/backup/tests/property_tests.rs", "w") as f:
    f.write(content)
