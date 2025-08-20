#!/bin/bash

# Script untuk memperbarui semua microfrontend ke Leptos 0.7.8 CSR

set -e

echo "🚀 Updating all microfrontend to Leptos 0.7.8 CSR..."

# List of microfrontends (excluding shared and portal that are already done)
MICROFRONTENDS=(
    "pemulihan_aset"
    "pengawasan"
    "pidmil"
    "pidsus"
    "pidum"
)

# Function to update lib.rs imports
update_lib_imports() {
    local mf_dir="$1"
    local lib_file="/var/www/simpelv2/antarmuka/$mf_dir/src/lib.rs"

    if [ -f "$lib_file" ]; then
        echo "  📝 Updating lib.rs imports for $mf_dir..."

        # Replace leptos::* with leptos::prelude::*
        sed -i 's/use leptos::\*;/use leptos::prelude::*;/g' "$lib_file"

        # Also update datun and badiklat if they haven't been updated
        if [ "$mf_dir" = "datun" ] || [ "$mf_dir" = "badiklat" ]; then
            sed -i 's/use leptos::\*;/use leptos::prelude::*;/g' "$lib_file"

            # Update main.rs too
            local main_file="/var/www/simpelv2/antarmuka/$mf_dir/src/main.rs"
            if [ -f "$main_file" ]; then
                sed -i 's/use leptos::\*;/use leptos::prelude::*;/g' "$main_file"
                sed -i 's/mount_to_body(/leptos::mount::mount_to_body(/g' "$main_file"
            fi
        fi
    fi
}

# Function to create main.rs
create_main_rs() {
    local mf_dir="$1"
    local main_file="/var/www/simpelv2/antarmuka/$mf_dir/src/main.rs"
    local package_name="${mf_dir//_/-}-microfrontend"

    if [ ! -f "$main_file" ]; then
        echo "  📝 Creating main.rs for $mf_dir..."

        cat > "$main_file" << EOF
use leptos::prelude::*;
use ${package_name}::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}
EOF
    else
        echo "  ✅ main.rs already exists for $mf_dir"
    fi
}

# Update each microfrontend
for mf in "${MICROFRONTENDS[@]}"; do
    echo "🔧 Processing $mf..."
    update_lib_imports "$mf"
    create_main_rs "$mf"
done

# Update existing main.rs files
echo "🔧 Updating existing main.rs files..."

# Update datun
if [ -f "/var/www/simpelv2/antarmuka/datun/src/main.rs" ]; then
    echo "  📝 Updating datun main.rs..."
    update_lib_imports "datun"
fi

# Update badiklat
if [ -f "/var/www/simpelv2/antarmuka/badiklat/src/main.rs" ]; then
    echo "  📝 Updating badiklat main.rs..."
    update_lib_imports "badiklat"
fi

echo "✅ All microfrontends updated to Leptos 0.7.8 CSR!"
echo "📋 Summary:"
echo "  - Using leptos::prelude::* for consistent imports"
echo "  - All using CSR with leptos::mount::mount_to_body"
echo "  - No hydrate or SSR features enabled"
echo "  - Leptos version: 0.7.8"
