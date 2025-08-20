#!/bin/bash

# Script untuk memperbaiki semua microfrontend ke Leptos 0.7.8 dengan API yang benar

set -e

echo "🔧 Fixing Leptos 0.7.8 imports and API usage..."

# Function to fix lib.rs
fix_lib_rs() {
    local mf_dir="$1"
    local lib_file="/var/www/simpelv2/antarmuka/$mf_dir/src/lib.rs"

    if [ -f "$lib_file" ]; then
        echo "  📝 Fixing lib.rs for $mf_dir..."

        # Fix imports for Leptos 0.7.8
        sed -i 's/use leptos::\*;/use leptos::prelude::*;/g' "$lib_file"

        # Add missing imports for router components
        if grep -q "Router\|Route\|Routes" "$lib_file"; then
            sed -i '/use leptos_router::\*;/a\
use leptos_router::components::{Router, Route, Routes};' "$lib_file"
        fi

        # Remove any remaining ::* patterns that might break
        sed -i 's/use leptos_router::\*;//g' "$lib_file"
    fi
}

# Function to fix pages.rs
fix_pages_rs() {
    local mf_dir="$1"
    local pages_file="/var/www/simpelv2/antarmuka/$mf_dir/src/pages.rs"

    if [ -f "$pages_file" ]; then
        echo "  📝 Fixing pages.rs for $mf_dir..."

        # Fix imports
        sed -i 's/use leptos::\*;/use leptos::prelude::*;/g' "$pages_file"

        # Fix Callback type import
        sed -i 's/leptos::Callback/leptos::callback::Callback/g' "$pages_file"

        # Add prelude import at the top if not exists
        if ! grep -q "use leptos::prelude::\*;" "$pages_file"; then
            sed -i '1i use leptos::prelude::*;' "$pages_file"
        fi
    fi
}

# Function to fix main.rs name import
fix_main_rs() {
    local mf_dir="$1"
    local main_file="/var/www/simpelv2/antarmuka/$mf_dir/src/main.rs"

    if [ -f "$main_file" ]; then
        echo "  📝 Fixing main.rs for $mf_dir..."

        # Convert package name properly
        local package_name=$(echo "$mf_dir" | sed 's/_/-/g')
        sed -i "s/use ${mf_dir//_/-}-microfrontend/use ${package_name}_microfrontend/g" "$main_file"
    fi
}

# List of microfrontends to fix
MICROFRONTENDS=(
    "intel"
    "pemulihan_aset"
    "pengawasan"
    "pidmil"
    "pidsus"
    "pidum"
    "datun"
    "badiklat"
)

# Fix each microfrontend
for mf in "${MICROFRONTENDS[@]}"; do
    echo "🔧 Fixing $mf..."
    fix_lib_rs "$mf"
    fix_pages_rs "$mf"
    fix_main_rs "$mf"
done

echo "✅ Basic fixes applied. Some manual fixes may still be needed."
