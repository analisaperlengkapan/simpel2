import re

with open("layanan/perlengkapan/Cargo.toml", "r") as f:
    content = f.read()

# Add anyhow to workspace.dependencies if not present
if "anyhow =" not in content and 'anyhow = { version = "1.0" }' not in content:
    if "[workspace.dependencies]" in content:
        content = content.replace("[workspace.dependencies]\n", "[workspace.dependencies]\nanyhow = \"1.0\"\n")
    else:
        content += "\n[workspace.dependencies]\nanyhow = \"1.0\"\n"

with open("layanan/perlengkapan/Cargo.toml", "w") as f:
    f.write(content)
