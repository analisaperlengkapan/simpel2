#!/bin/bash
# ===========================================================================
# Bidirectional GitHub-GitLab Sync Setup Script
# ===========================================================================
# This script helps configure the required secrets and tokens for
# bidirectional synchronization between GitHub and GitLab repositories.
#
# Usage: ./scripts/setup-bidirectional-sync.sh [--check|--setup|--test]
# ===========================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="analisaperlengkapan/simpel2"
GITLAB_URL="https://gitlab.kejaksaan.go.id"
GITLAB_PROJECT="irhas/simpel2"

# Function to print colored output
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    local all_ok=true

    # Check for required commands
    for cmd in git curl jq; do
        if command -v $cmd &> /dev/null; then
            print_success "$cmd is installed"
        else
            print_error "$cmd is not installed"
            all_ok=false
        fi
    done

    # Check Git remotes
    echo ""
    print_info "Checking Git remotes..."

    if git remote get-url origin &> /dev/null; then
        ORIGIN_URL=$(git remote get-url origin)
        print_success "origin remote: $ORIGIN_URL"
    else
        print_error "No origin remote configured"
        all_ok=false
    fi

    if git remote get-url gitlab &> /dev/null; then
        GITLAB_REMOTE=$(git remote get-url gitlab | sed 's/:[^:]*@/:***@/')
        print_success "gitlab remote: $GITLAB_REMOTE"
    else
        print_warning "No gitlab remote configured (will be added by workflow)"
    fi

    if [ "$all_ok" = true ]; then
        echo ""
        print_success "All prerequisites met!"
        return 0
    else
        echo ""
        print_error "Some prerequisites are missing"
        return 1
    fi
}

# Function to display setup instructions
show_setup_instructions() {
    print_header "Setup Instructions"

    cat << 'EOF'
To enable bidirectional sync, you need to configure the following:

┌─────────────────────────────────────────────────────────────────────────┐
│                         GITHUB CONFIGURATION                             │
└─────────────────────────────────────────────────────────────────────────┘

1. Go to: https://github.com/analisaperlengkapan/simpel2/settings/secrets/actions

2. Add the following secrets:

   GITLAB_MIRROR_URL
   ─────────────────
   Value: https://oauth2:<GITLAB_TOKEN>@gitlab.kejaksaan.go.id/irhas/simpel2.git

   Replace <GITLAB_TOKEN> with your GitLab Project Access Token.

   GITLAB_API_TOKEN
   ────────────────
   Value: <GITLAB_TOKEN>

   This is the same token, used for API calls to create Merge Requests.

3. (Optional) Add repository variables:
   Settings > Secrets and variables > Actions > Variables

   AUTO_MERGE_SYNC = true     # Auto-merge sync MRs when pipeline passes
   CREATE_DIVERGENCE_ISSUES = true  # Create issues when branches diverge

┌─────────────────────────────────────────────────────────────────────────┐
│                         GITLAB CONFIGURATION                             │
└─────────────────────────────────────────────────────────────────────────┘

1. Create a GitLab Project Access Token:
   Project > Settings > Access Tokens

   - Name: github-sync-token
   - Role: Maintainer
   - Scopes: api, write_repository
   - Expiration: Set appropriate date (recommend 1 year)

2. Go to: Project > Settings > CI/CD > Variables

3. Add the following variable:

   GITHUB_TOKEN
   ────────────
   Type: Variable
   Environment scope: All
   Protect variable: Yes (recommended)
   Mask variable: Yes
   Value: <GITHUB_PAT>

   Create a GitHub Fine-grained PAT at:
   https://github.com/settings/tokens?type=beta

   Permissions needed:
   - Repository: analisaperlengkapan/simpel2
   - Contents: Read and write
   - Pull requests: Read and write
   - Metadata: Read

┌─────────────────────────────────────────────────────────────────────────┐
│                         SYNC MARKERS                                     │
└─────────────────────────────────────────────────────────────────────────┘

The following markers are used to identify sync commits and prevent loops:

Commit Message Prefixes:
  [github-sync] - Commit synced from GitHub to GitLab
  [gitlab-sync] - Commit synced from GitLab to GitHub
  [skip sync]   - Skip sync for this commit

Branch Name Patterns:
  sync/from-github-* - Branches created for GitHub → GitLab sync
  sync/from-gitlab-* - Branches created for GitLab → GitHub sync

Author Emails:
  github-gitlab-sync[bot]@users.noreply.github.com (GitHub workflow)
  gitlab-github-sync@kejaksaan.go.id (GitLab CI)

EOF
}

# Function to test GitHub API access
test_github_api() {
    print_header "Testing GitHub API Access"

    read -p "Enter your GitHub Personal Access Token: " -s GITHUB_TOKEN
    echo ""

    echo "Testing API access..."

    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$GITHUB_REPO")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" -eq 200 ]; then
        REPO_NAME=$(echo "$BODY" | jq -r '.full_name')
        PERMISSIONS=$(echo "$BODY" | jq -r '.permissions')

        print_success "Successfully accessed repository: $REPO_NAME"
        echo ""
        echo "Permissions:"
        echo "$PERMISSIONS" | jq .

        # Check if we can create PRs
        CAN_PUSH=$(echo "$BODY" | jq -r '.permissions.push')
        if [ "$CAN_PUSH" = "true" ]; then
            print_success "Token has push permission (can create PRs)"
        else
            print_warning "Token may not have push permission"
        fi
    else
        print_error "Failed to access repository (HTTP $HTTP_CODE)"
        echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    fi
}

# Function to test GitLab API access
test_gitlab_api() {
    print_header "Testing GitLab API Access"

    read -p "Enter your GitLab Access Token: " -s GITLAB_TOKEN
    echo ""

    echo "Testing API access..."

    PROJECT_ENCODED=$(echo "$GITLAB_PROJECT" | sed 's/\//%2F/g')

    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "PRIVATE-TOKEN: $GITLAB_TOKEN" \
        "$GITLAB_URL/api/v4/projects/$PROJECT_ENCODED")

    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" -eq 200 ]; then
        PROJECT_NAME=$(echo "$BODY" | jq -r '.path_with_namespace')
        ACCESS_LEVEL=$(echo "$BODY" | jq -r '.permissions.project_access.access_level // .permissions.group_access.access_level // "N/A"')

        print_success "Successfully accessed project: $PROJECT_NAME"
        echo ""
        echo "Access Level: $ACCESS_LEVEL"

        # Access levels: 10=Guest, 20=Reporter, 30=Developer, 40=Maintainer, 50=Owner
        if [ "$ACCESS_LEVEL" -ge 40 ] 2>/dev/null; then
            print_success "Token has Maintainer or higher access (can create MRs)"
        elif [ "$ACCESS_LEVEL" -ge 30 ] 2>/dev/null; then
            print_warning "Token has Developer access (may be able to create MRs)"
        else
            print_warning "Token may not have sufficient permissions"
        fi
    else
        print_error "Failed to access project (HTTP $HTTP_CODE)"
        echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    fi
}

# Function to check sync status
check_sync_status() {
    print_header "Checking Sync Status"

    # Fetch both remotes
    print_info "Fetching remotes..."
    git fetch origin main --depth=50 2>/dev/null || true

    if git remote get-url gitlab &> /dev/null; then
        git fetch gitlab main --depth=50 2>/dev/null || true

        ORIGIN_SHA=$(git rev-parse origin/main 2>/dev/null || echo "UNKNOWN")
        GITLAB_SHA=$(git rev-parse gitlab/main 2>/dev/null || echo "UNKNOWN")

        echo ""
        echo "Branch Status:"
        echo "  origin/main (GitHub): $ORIGIN_SHA"
        echo "  gitlab/main:          $GITLAB_SHA"
        echo ""

        if [ "$ORIGIN_SHA" = "$GITLAB_SHA" ]; then
            print_success "Repositories are IN SYNC"
        elif [ "$ORIGIN_SHA" = "UNKNOWN" ] || [ "$GITLAB_SHA" = "UNKNOWN" ]; then
            print_warning "Could not determine sync status"
        else
            print_warning "Repositories are OUT OF SYNC"

            # Try to determine direction
            if git merge-base --is-ancestor gitlab/main origin/main 2>/dev/null; then
                AHEAD=$(git rev-list --count gitlab/main..origin/main 2>/dev/null || echo "?")
                echo "  GitHub is ahead by $AHEAD commits"
            elif git merge-base --is-ancestor origin/main gitlab/main 2>/dev/null; then
                AHEAD=$(git rev-list --count origin/main..gitlab/main 2>/dev/null || echo "?")
                echo "  GitLab is ahead by $AHEAD commits"
            else
                echo "  Branches have diverged"
            fi
        fi
    else
        print_warning "GitLab remote not configured"
        echo ""
        echo "To add GitLab remote, run:"
        echo "  git remote add gitlab https://oauth2:<TOKEN>@gitlab.kejaksaan.go.id/irhas/simpel2.git"
    fi
}

# Function to show help
show_help() {
    cat << EOF
Bidirectional GitHub-GitLab Sync Setup Script

Usage: $0 [COMMAND]

Commands:
  --check       Check prerequisites and current configuration
  --setup       Show detailed setup instructions
  --test-github Test GitHub API access with a token
  --test-gitlab Test GitLab API access with a token
  --status      Check current sync status between repositories
  --help        Show this help message

Examples:
  $0 --check        # Verify prerequisites are installed
  $0 --setup        # See configuration instructions
  $0 --status       # Check if repos are in sync
  $0 --test-github  # Verify GitHub token works
  $0 --test-gitlab  # Verify GitLab token works

For more information, see:
  docs/BIDIRECTIONAL_SYNC_ANALYSIS.md
EOF
}

# Main script
case "${1:-}" in
    --check)
        check_prerequisites
        ;;
    --setup)
        show_setup_instructions
        ;;
    --test-github)
        test_github_api
        ;;
    --test-gitlab)
        test_gitlab_api
        ;;
    --status)
        check_sync_status
        ;;
    --help|"")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
