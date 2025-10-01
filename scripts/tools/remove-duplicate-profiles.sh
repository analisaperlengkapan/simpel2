#!/bin/bash
# remove-duplicate-profiles.sh
# Remove duplicate profile sections from microfrontend Cargo.toml files

set -euo pipefail

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Removing duplicate profiles from microfrontend Cargo.toml files...${NC}"

# List of microfrontends
MICROFRONTENDS=(
    "antarmuka/badiklat"
    "antarmuka/datun"
    "antarmuka/intel"
    "antarmuka/pidum"
    "antarmuka/pidsus"
    "antarmuka/pidmil"
    "antarmuka/pengawasan"
    "antarmuka/pemulihan_aset"
    "antarmuka/portal"
    "antarmuka/pembinaan/keuangan"
    "antarmuka/pembinaan/perencanaan"
    "antarmuka/pembinaan/perlengkapan"
)

for mf in "${MICROFRONTENDS[@]}"; do
    cargo_file="${mf}/Cargo.toml"
    if [ -f "$cargo_file" ]; then
        # Remove profile sections using sed
        # This removes from [profile.release] or [profile.dev] to the next section or EOF
        sed -i '/^\[profile\./,/^$/d' "$cargo_file"
        echo -e "${GREEN}✅ Cleaned: $cargo_file${NC}"
    fi
done

echo -e "${GREEN}Done!${NC}"
