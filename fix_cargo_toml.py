import re

with open("Cargo.toml", "r") as f:
    content = f.read()

# Remove 'layanan/authenc' from workspace members
content = re.sub(r'\s*"layanan/authenc",\s*# Legacy - migrated to crates/\n', '\n', content)

# Add anyhow to workspace.dependencies if not present
if "anyhow =" not in content and 'anyhow = { version = "1.0" }' not in content:
    # Find a good spot to insert it, maybe under [workspace.dependencies]
    if "[workspace.dependencies]" in content:
        content = content.replace("[workspace.dependencies]\n", "[workspace.dependencies]\nanyhow = \"1.0\"\n")

with open("Cargo.toml", "w") as f:
    f.write(content)
