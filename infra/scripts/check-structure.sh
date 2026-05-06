#!/bin/bash
set -e

echo "Running Repository Structure Check..."

# Allowed folders in root
declare -a ALLOWED_FOLDERS=(".github" ".vscode" ".gitlab" "docs" "infra" "layanan" "antarmuka" "lib" "monolith" "tests" "target")
# Allowed root files (besides .md docs which are checked separately)
declare -a ALLOWED_FILES=("Cargo.toml" "Cargo.lock" "README.md" "CONTRIBUTING.md" "AGENTS.md" "CHANGELOG.md" "deny.toml" "playwright.config.ts" ".gitignore" ".editorconfig" ".gitleaks.toml" "docker-compose.yml" "docker-compose.build.yml" "SIMPEL.code-workspace")

FAILED=0

echo -e "\n1. Checking for invalid directories in root..."
for dir in */; do
  dir=${dir%/}
  if [[ ! " ${ALLOWED_FOLDERS[@]} " =~ " ${dir} " ]]; then
    echo "❌ ERROR: Directory '$dir' is not allowed in root."
    FAILED=1
  fi
done

echo -e "\n2. Checking for invalid md files..."
# All md files must be in docs/ except specific allowed ones in root and domain AGENTS.md
while IFS= read -r file; do
  # Remove leading ./
  clean_file=${file#./}
  file_name=${clean_file##*/}

  # Allow README, CONTRIBUTING, AGENTS, CHANGELOG in any location
  if [[ "$file_name" == "README.md" || "$file_name" == "CONTRIBUTING.md" || "$file_name" == "AGENTS.md" || "$file_name" == "CHANGELOG.md" ]]; then
    continue
  fi

  # Allow anything inside any docs/ subtree
  if [[ "$clean_file" == *"/docs/"* || "$clean_file" == docs/* ]]; then
    continue
  fi

  # Allow AGENTS.md inside any folder
  if [[ "$clean_file" == */AGENTS.md ]]; then
    continue
  fi

  echo "❌ ERROR: Markdown file '$clean_file' is outside allowed directories (should be in docs/)."
  FAILED=1
done < <(find . \
  \( -path './.*' -o -path '*/.*' -o -path './target' -o -path '*/target' -o -path '*/node_modules' \) -prune -o \
  -type f -name "*.md" -print)

if [ $FAILED -ne 0 ]; then
  echo -e "\n❌ Structure test failed."
  exit 1
else
  echo -e "\n✅ Structure test passed."
fi
