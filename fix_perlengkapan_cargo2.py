import re

with open("layanan/perlengkapan/Cargo.toml", "r") as f:
    content = f.read()

deps = """
anyhow = "1.0"
async-trait = "0.1"
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.30", features = ["full"] }
tracing = "0.1"
uuid = { version = "1.4", features = ["v4", "serde"] }
"""

# Let's just look at what dependencies are missing by copying from root Cargo.toml
with open("Cargo.toml", "r") as f2:
    root_content = f2.read()

root_deps = root_content.split("[workspace.dependencies]")[1].split("[")[0]

if "[workspace.dependencies]" not in content:
    content += "\n[workspace.dependencies]\n"

with open("layanan/perlengkapan/Cargo.toml", "w") as f:
    f.write(content)
