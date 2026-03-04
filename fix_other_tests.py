import re
import glob

files = glob.glob("layanan/secreton/crates/backup/tests/*.rs")

for file in files:
    with open(file, "r") as f:
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

    with open(file, "w") as f:
        f.write(content)
