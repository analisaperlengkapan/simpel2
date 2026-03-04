import re

with open("layanan/perlengkapan/Cargo.toml", "r") as f:
    content = f.read()

content = content.replace('"crates/integrasi",\n', '')

with open("layanan/perlengkapan/Cargo.toml", "w") as f:
    f.write(content)
