import re

with open("layanan/perlengkapan/Cargo.toml", "r") as f:
    content = f.read()

# Fix path dependencies
content = content.replace('path = "lib/common"', 'path = "../../lib/common"')
content = content.replace('path = "lib/perlengkapan"', 'path = "../../lib/perlengkapan"')
content = content.replace('path = "layanan/secreton/crates/types"', 'path = "../../layanan/secreton/crates/types"')

with open("layanan/perlengkapan/Cargo.toml", "w") as f:
    f.write(content)
