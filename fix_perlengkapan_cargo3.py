import re

with open("Cargo.toml", "r") as f:
    root_content = f.read()

deps_section = root_content.split("[workspace.dependencies]")[1]
if "\n[" in deps_section:
    deps_section = deps_section.split("\n[")[0]

with open("layanan/perlengkapan/Cargo.toml", "r") as f:
    content = f.read()

if "[workspace.dependencies]" in content:
    content = content.split("[workspace.dependencies]")[0]

content += "[workspace.dependencies]\n" + deps_section

with open("layanan/perlengkapan/Cargo.toml", "w") as f:
    f.write(content)
