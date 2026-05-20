#!/usr/bin/env bash
# precommit-secret-scan.sh — Pre-commit hook untuk catch secret leak.
#
# Install:
#   ln -s ../../scripts/precommit-secret-scan.sh .git/hooks/pre-commit
# Atau pakai pre-commit framework dengan .pre-commit-config.yaml.
#
# Behavior:
#   - Scan staged changes pakai gitleaks (jika installed).
#   - Heuristic git grep untuk pola secret di files yang akan di-commit.
#   - Block commit jika ada finding.

set -euo pipefail

CYAN='\033[0;36m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${CYAN}>>> Pre-commit secret scan...${NC}"

# 1. Cek staged .env files
STAGED_ENV=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.env(\.|$)' | grep -vE '\.example$|\.template$' || true)
if [[ -n "$STAGED_ENV" ]]; then
  echo -e "${RED}✗ ERROR: .env file di-stage (selain .example/.template):${NC}"
  echo "$STAGED_ENV"
  echo -e "${YELLOW}  Hapus dari staging: git reset HEAD <file>${NC}"
  exit 1
fi

# 2. Run gitleaks jika ada
if command -v gitleaks >/dev/null 2>&1; then
  echo -e "${CYAN}    gitleaks scanning staged...${NC}"
  if ! gitleaks protect --staged --redact --config .gitleaks.toml --no-banner; then
    echo -e "${RED}✗ gitleaks finding terdeteksi di staged changes.${NC}"
    echo -e "${YELLOW}  Review temuan di atas. Jika false positive, tambahkan ke .gitleaks.toml allowlist.${NC}"
    echo -e "${YELLOW}  Bypass (NOT RECOMMENDED): git commit --no-verify${NC}"
    exit 1
  fi
else
  echo -e "${YELLOW}    gitleaks tidak installed, skip (rekomendasi: 'brew install gitleaks' atau curl release).${NC}"
fi

# 3. Heuristic grep untuk pola secret di staged content
STAGED_DIFF=$(git diff --cached --diff-filter=ACM -U0 \
  -- ':!*.lock' ':!*.md' ':!*.example' ':!*.template' ':!docs/' ':!tests/' ':!**/fixtures/' || true)

if echo "$STAGED_DIFF" | grep -iE '^\+[^+].*(password|secret|token|api[_-]?key|app[_-]?key)\s*[:=]\s*["'\'']?[A-Za-z0-9+/_=-]{16,}' >/dev/null; then
  echo -e "${YELLOW}⚠ Possible secret pattern di staged changes:${NC}"
  echo "$STAGED_DIFF" | grep -iE '^\+.*(password|secret|token|api[_-]?key|app[_-]?key).*[:=].*[A-Za-z0-9+/_=-]{16,}' | head -10
  echo -e "${YELLOW}  Pastikan ini bukan secret real. Jika dummy/test, tambahkan keyword 'test', 'dummy', 'example', atau move ke file .example/.template.${NC}"
  read -rp "Lanjut commit? [y/N] " -n 1 ANSWER
  echo
  if [[ ! "$ANSWER" =~ ^[yY]$ ]]; then
    echo -e "${RED}Commit dibatalkan.${NC}"
    exit 1
  fi
fi

echo -e "${GREEN}✓ Pre-commit secret scan pass.${NC}"
exit 0
