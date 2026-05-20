# Bidirectional GitHub-GitLab Synchronization Analysis

## Executive Summary

This document analyzes the feasibility and recommended approach for implementing bidirectional synchronization between the GitHub repository (`analisaperlengkapan/simpel2`) and the GitLab repository (`https://gitlab.kejaksaan.go.id/irhas/simpel2`) using a PR/MR-based approach instead of direct pushes.

---

## 1. Current State Analysis

### Existing GitHub Action (`sync-to-gitlab.yml`)

```yaml
# Current approach: Direct mirror push
git push --mirror --prune "$GITLAB_MIRROR_URL"
```

**Issues with current approach:**

- Unidirectional (GitHub → GitLab only)
- Uses `--mirror --prune` which overwrites everything on GitLab
- No review process before changes are applied
- Risk of overwriting work done directly on GitLab
- No audit trail of sync operations

---

## 2. API Mechanisms Available

### 2.1 GitHub Actions → Create GitLab Merge Requests

**GitLab REST API v4:**

```bash
# Create Merge Request
POST https://gitlab.kejaksaan.go.id/api/v4/projects/:id/merge_requests

# Required parameters:
{
  "source_branch": "sync/from-github-{timestamp}",
  "target_branch": "main",
  "title": "[Sync] Changes from GitHub main",
  "description": "Automated sync from GitHub\n\nCommits: ...",
  "remove_source_branch": true,
  "squash": false
}

# Authentication:
# - Project Access Token (recommended): PRIVATE-TOKEN header
# - Personal Access Token: PRIVATE-TOKEN header
# - OAuth2 token: Authorization: Bearer header
```

**Required GitLab Permissions:**

- `api` scope for full API access
- `write_repository` for pushing branches
- Maintainer role or higher on the project

**GitHub Action Implementation:**

```yaml
- name: Create GitLab Merge Request
  run: |
    curl --request POST \
      --header "PRIVATE-TOKEN: ${{ secrets.GITLAB_API_TOKEN }}" \
      --header "Content-Type: application/json" \
      --data '{
        "source_branch": "sync/from-github-${{ github.run_id }}",
        "target_branch": "main",
        "title": "[Sync] GitHub → GitLab: ${{ github.event.head_commit.message }}",
        "remove_source_branch": true
      }' \
      "https://gitlab.kejaksaan.go.id/api/v4/projects/irhas%2Fsimpel2/merge_requests"
```

### 2.2 GitLab CI/CD → Create GitHub Pull Requests

**GitHub REST API v3:**

```bash
# Create Pull Request
POST https://api.github.com/repos/analisaperlengkapan/simpel2/pulls

# Required parameters:
{
  "title": "[Sync] Changes from GitLab main",
  "body": "Automated sync from GitLab\n\nMerge Request: ...",
  "head": "sync/from-gitlab-{timestamp}",
  "base": "main",
  "maintainer_can_modify": true
}

# Authentication:
# - Personal Access Token: Authorization: token header
# - GitHub App: Authorization: Bearer header (JWT)
# - Fine-grained PAT (recommended): Authorization: Bearer header
```

**Required GitHub Permissions:**

- `repo` scope for full repository access
- `pull_requests: write` for creating PRs
- `contents: write` for pushing branches

**GitLab CI Implementation:**

```yaml
create-github-pr:
  stage: sync
  script:
    - |
      curl -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        -d '{
          "title": "[Sync] GitLab → GitHub: '"$CI_COMMIT_TITLE"'",
          "body": "Automated sync from GitLab MR",
          "head": "sync/from-gitlab-'"$CI_PIPELINE_ID"'",
          "base": "main"
        }' \
        "https://api.github.com/repos/analisaperlengkapan/simpel2/pulls"
  only:
    - main
```

---

## 3. Loop Prevention Strategies

### 3.1 Commit Message Markers

**Approach:** Add identifiable markers to sync commits

```bash
# GitHub → GitLab commits
[github-sync] Original commit message

# GitLab → GitHub commits
[gitlab-sync] Original commit message
```

**Detection:**

```yaml
# GitHub Action - Skip if commit is from GitLab
- if: "!contains(github.event.head_commit.message, '[gitlab-sync]')"

# GitLab CI - Skip if commit is from GitHub
rules:
  - if: '$CI_COMMIT_MESSAGE !~ /\[github-sync\]/'
```

### 3.2 Author/Committer Detection

**Approach:** Use dedicated bot accounts for sync operations

```bash
# GitHub sync bot
github-gitlab-sync[bot]@users.noreply.github.com

# GitLab sync bot
gitlab-github-sync@kejaksaan.go.id
```

**Detection:**

```yaml
# GitHub Action
- if: github.event.head_commit.author.email != 'gitlab-github-sync@kejaksaan.go.id'

# GitLab CI
rules:
  - if: '$GITLAB_USER_EMAIL != "github-gitlab-sync[bot]@users.noreply.github.com"'
```

### 3.3 Branch Name Pattern Exclusion

**Approach:** Never sync branches that match sync patterns

```yaml
# Exclude sync branches from triggering workflows
on:
  push:
    branches:
      - main
    branches-ignore:
      - "sync/**"
```

### 3.4 CI Variable/Label Markers

**Approach:** Set environment variables or labels to track sync origin

```yaml
# Set in sync commits
env:
  SYNC_ORIGIN: "github"  # or "gitlab"

# Check before running
- if: env.SYNC_ORIGIN != 'gitlab'
```

### 3.5 Recommended: Combined Approach

Use multiple detection methods for robustness:

1. **Primary:** Commit message prefix `[sync:origin]`
2. **Secondary:** Dedicated committer email
3. **Tertiary:** Branch name pattern `sync/*`
4. **Quaternary:** CI skip directive `[ci skip:sync]`

---

## 4. Branch Comparison Strategies

### 4.1 Git-Based Comparison

```bash
# Fetch both remotes
git remote add github https://github.com/analisaperlengkapan/simpel2.git
git remote add gitlab https://gitlab.kejaksaan.go.id/irhas/simpel2.git
git fetch github main
git fetch gitlab main

# Compare commits
GITHUB_SHA=$(git rev-parse github/main)
GITLAB_SHA=$(git rev-parse gitlab/main)

if [ "$GITHUB_SHA" != "$GITLAB_SHA" ]; then
  # Determine which is ahead
  if git merge-base --is-ancestor github/main gitlab/main; then
    echo "GitLab is ahead - sync to GitHub"
  elif git merge-base --is-ancestor gitlab/main github/main; then
    echo "GitHub is ahead - sync to GitLab"
  else
    echo "Branches have diverged - manual intervention needed"
  fi
fi
```

### 4.2 API-Based Comparison

```bash
# Get GitHub main SHA
GITHUB_SHA=$(curl -s \
  -H "Authorization: token $GITHUB_TOKEN" \
  "https://api.github.com/repos/analisaperlengkapan/simpel2/git/ref/heads/main" \
  | jq -r '.object.sha')

# Get GitLab main SHA
GITLAB_SHA=$(curl -s \
  -H "PRIVATE-TOKEN: $GITLAB_TOKEN" \
  "https://gitlab.kejaksaan.go.id/api/v4/projects/irhas%2Fsimpel2/repository/branches/main" \
  | jq -r '.commit.id')

# Compare
if [ "$GITHUB_SHA" != "$GITLAB_SHA" ]; then
  echo "Branches differ - sync needed"
fi
```

### 4.3 Commit Log Comparison

```bash
# Get commits unique to GitHub
git log gitlab/main..github/main --oneline

# Get commits unique to GitLab
git log github/main..gitlab/main --oneline
```

---

## 5. Recommended Architecture

### 5.1 High-Level Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     BIDIRECTIONAL SYNC ARCHITECTURE                      │
└─────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────┐
                            │  Developer   │
                            └──────┬───────┘
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
            ┌───────────────┐             ┌───────────────┐
            │    GitHub     │             │    GitLab     │
            │   main branch │             │  main branch  │
            └───────┬───────┘             └───────┬───────┘
                    │                             │
                    ▼                             ▼
        ┌───────────────────────┐   ┌───────────────────────┐
        │   GitHub Action       │   │    GitLab CI Job      │
        │   (sync-checker)      │   │   (sync-checker)      │
        └───────────┬───────────┘   └───────────┬───────────┘
                    │                             │
                    ▼                             ▼
        ┌───────────────────────┐   ┌───────────────────────┐
        │ Is commit from sync?  │   │ Is commit from sync?  │
        │ [gitlab-sync] prefix  │   │ [github-sync] prefix  │
        └───────────┬───────────┘   └───────────┬───────────┘
                    │                             │
              ┌─────┴─────┐                 ┌─────┴─────┐
              │Yes     No │                 │Yes     No │
              ▼           ▼                 ▼           ▼
           [SKIP]    [CONTINUE]          [SKIP]    [CONTINUE]
                          │                             │
                          ▼                             ▼
              ┌───────────────────────┐   ┌───────────────────────┐
              │ Compare with GitLab   │   │ Compare with GitHub   │
              │ main branch           │   │ main branch           │
              └───────────┬───────────┘   └───────────┬───────────┘
                          │                             │
                    ┌─────┴─────┐                 ┌─────┴─────┐
                    │Same   Diff│                 │Same   Diff│
                    ▼           ▼                 ▼           ▼
                 [DONE]   [CONTINUE]           [DONE]   [CONTINUE]
                               │                             │
                               ▼                             ▼
              ┌───────────────────────┐   ┌───────────────────────┐
              │ Push to sync branch   │   │ Push to sync branch   │
              │ sync/from-github-xxx  │   │ sync/from-gitlab-xxx  │
              └───────────┬───────────┘   └───────────┬───────────┘
                          │                             │
                          ▼                             ▼
              ┌───────────────────────┐   ┌───────────────────────┐
              │ Create GitLab MR      │   │ Create GitHub PR      │
              │ via GitLab API        │   │ via GitHub API        │
              └───────────┬───────────┘   └───────────┬───────────┘
                          │                             │
                          ▼                             ▼
              ┌───────────────────────┐   ┌───────────────────────┐
              │ Auto-merge if clean   │   │ Auto-merge if clean   │
              │ (optional)            │   │ (optional)            │
              └───────────────────────┘   └───────────────────────┘
```

### 5.2 GitHub Actions Workflow

```yaml
# .github/workflows/bidirectional-sync.yml
name: Bidirectional GitLab Sync

on:
  push:
    branches:
      - main
  workflow_dispatch:
    inputs:
      force_sync:
        description: "Force sync even if no differences detected"
        type: boolean
        default: false

concurrency:
  group: bidirectional-sync
  cancel-in-progress: false

env:
  SYNC_PREFIX_GITLAB: "[gitlab-sync]"
  SYNC_PREFIX_GITHUB: "[github-sync]"
  SYNC_AUTHOR_EMAIL: "github-gitlab-sync[bot]@users.noreply.github.com"
  GITLAB_PROJECT: "irhas/simpel2"
  GITLAB_URL: "https://gitlab.kejaksaan.go.id"

jobs:
  detect-origin:
    name: Detect Commit Origin
    runs-on: self-hosted
    outputs:
      is_sync_commit: ${{ steps.check.outputs.is_sync }}
      should_sync: ${{ steps.check.outputs.should_sync }}
    steps:
      - name: Check if sync commit
        id: check
        run: |
          COMMIT_MSG="${{ github.event.head_commit.message }}"
          AUTHOR_EMAIL="${{ github.event.head_commit.author.email }}"

          # Check for sync markers
          if [[ "$COMMIT_MSG" == *"${{ env.SYNC_PREFIX_GITLAB }}"* ]] || \
             [[ "$AUTHOR_EMAIL" == "${{ env.SYNC_AUTHOR_EMAIL }}" ]]; then
            echo "is_sync=true" >> $GITHUB_OUTPUT
            echo "should_sync=false" >> $GITHUB_OUTPUT
            echo "⏭️ Skipping: This is a sync commit from GitLab"
          else
            echo "is_sync=false" >> $GITHUB_OUTPUT
            echo "should_sync=true" >> $GITHUB_OUTPUT
            echo "✅ Proceeding: This is an original commit"
          fi

  compare-branches:
    name: Compare with GitLab
    needs: detect-origin
    if: needs.detect-origin.outputs.should_sync == 'true'
    runs-on: self-hosted
    outputs:
      github_ahead: ${{ steps.compare.outputs.github_ahead }}
      commits_to_sync: ${{ steps.compare.outputs.commits_to_sync }}
      sync_branch: ${{ steps.compare.outputs.sync_branch }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Configure Git
        run: |
          git config --global user.email "${{ env.SYNC_AUTHOR_EMAIL }}"
          git config --global user.name "GitHub-GitLab Sync Bot"

      - name: Add GitLab remote
        run: |
          git remote add gitlab "${{ secrets.GITLAB_MIRROR_URL }}" || true
          git fetch gitlab main --depth=100

      - name: Compare branches
        id: compare
        run: |
          GITHUB_SHA=$(git rev-parse HEAD)
          GITLAB_SHA=$(git rev-parse gitlab/main 2>/dev/null || echo "")

          if [ -z "$GITLAB_SHA" ]; then
            echo "⚠️ Could not fetch GitLab main branch"
            echo "github_ahead=false" >> $GITHUB_OUTPUT
            exit 0
          fi

          if [ "$GITHUB_SHA" = "$GITLAB_SHA" ]; then
            echo "✅ Branches are in sync"
            echo "github_ahead=false" >> $GITHUB_OUTPUT
          else
            # Check if GitHub is ahead
            if git merge-base --is-ancestor gitlab/main HEAD; then
              COMMITS=$(git log gitlab/main..HEAD --oneline | wc -l)
              echo "📤 GitHub is ahead by $COMMITS commits"
              echo "github_ahead=true" >> $GITHUB_OUTPUT
              echo "commits_to_sync=$COMMITS" >> $GITHUB_OUTPUT
              echo "sync_branch=sync/from-github-${{ github.run_id }}" >> $GITHUB_OUTPUT
            else
              echo "⚠️ Branches have diverged - manual intervention may be needed"
              echo "github_ahead=diverged" >> $GITHUB_OUTPUT
            fi
          fi

  create-gitlab-mr:
    name: Create GitLab Merge Request
    needs: [detect-origin, compare-branches]
    if: needs.compare-branches.outputs.github_ahead == 'true'
    runs-on: self-hosted
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Configure Git
        run: |
          git config --global user.email "${{ env.SYNC_AUTHOR_EMAIL }}"
          git config --global user.name "GitHub-GitLab Sync Bot"

      - name: Create sync branch and push to GitLab
        env:
          GITLAB_MIRROR_URL: ${{ secrets.GITLAB_MIRROR_URL }}
        run: |
          SYNC_BRANCH="${{ needs.compare-branches.outputs.sync_branch }}"

          # Create sync branch
          git checkout -b "$SYNC_BRANCH"

          # Add sync marker to last commit
          git commit --amend -m "${{ env.SYNC_PREFIX_GITHUB }} ${{ github.event.head_commit.message }}"

          # Push sync branch to GitLab
          git push "$GITLAB_MIRROR_URL" "$SYNC_BRANCH" --force

      - name: Create Merge Request via API
        env:
          GITLAB_API_TOKEN: ${{ secrets.GITLAB_API_TOKEN }}
        run: |
          SYNC_BRANCH="${{ needs.compare-branches.outputs.sync_branch }}"
          PROJECT_ENCODED=$(echo "${{ env.GITLAB_PROJECT }}" | sed 's/\//%2F/g')

          # Create Merge Request
          RESPONSE=$(curl -s -w "\n%{http_code}" \
            --request POST \
            --header "PRIVATE-TOKEN: $GITLAB_API_TOKEN" \
            --header "Content-Type: application/json" \
            --data "{
              \"source_branch\": \"$SYNC_BRANCH\",
              \"target_branch\": \"main\",
              \"title\": \"${{ env.SYNC_PREFIX_GITHUB }} Sync from GitHub main\",
              \"description\": \"## Automated Sync from GitHub\\n\\n**Source:** GitHub main branch\\n**Commits:** ${{ needs.compare-branches.outputs.commits_to_sync }}\\n**Triggered by:** ${{ github.event.head_commit.message }}\\n\\n---\\n*This MR was created automatically by the bidirectional sync workflow.*\",
              \"remove_source_branch\": true,
              \"squash\": false,
              \"labels\": \"sync,automated\"
            }" \
            "${{ env.GITLAB_URL }}/api/v4/projects/$PROJECT_ENCODED/merge_requests")

          HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
          BODY=$(echo "$RESPONSE" | sed '$d')

          if [ "$HTTP_CODE" -ge 200 ] && [ "$HTTP_CODE" -lt 300 ]; then
            MR_URL=$(echo "$BODY" | jq -r '.web_url')
            MR_IID=$(echo "$BODY" | jq -r '.iid')
            echo "✅ Merge Request created: $MR_URL"
            echo "MR_URL=$MR_URL" >> $GITHUB_ENV
            echo "MR_IID=$MR_IID" >> $GITHUB_ENV
          else
            echo "❌ Failed to create MR: $BODY"
            exit 1
          fi

      - name: Auto-merge if enabled
        if: ${{ vars.AUTO_MERGE_SYNC == 'true' }}
        env:
          GITLAB_API_TOKEN: ${{ secrets.GITLAB_API_TOKEN }}
        run: |
          PROJECT_ENCODED=$(echo "${{ env.GITLAB_PROJECT }}" | sed 's/\//%2F/g')

          # Accept the merge request
          curl -s \
            --request PUT \
            --header "PRIVATE-TOKEN: $GITLAB_API_TOKEN" \
            --data "merge_when_pipeline_succeeds=true" \
            "${{ env.GITLAB_URL }}/api/v4/projects/$PROJECT_ENCODED/merge_requests/$MR_IID/merge"

      - name: Update GitHub Summary
        run: |
          cat >> $GITHUB_STEP_SUMMARY << EOF
          ## 🔄 GitLab Sync Complete

          - **Merge Request:** $MR_URL
          - **Commits synced:** ${{ needs.compare-branches.outputs.commits_to_sync }}
          - **Source:** GitHub main
          - **Target:** GitLab main
          - **Status:** Pending review
          EOF
```

### 5.3 GitLab CI Configuration

```yaml
# Add to .gitlab-ci.yml
stages:
  # ... existing stages ...
  - sync

variables:
  # ... existing variables ...
  SYNC_PREFIX_GITHUB: "[github-sync]"
  SYNC_PREFIX_GITLAB: "[gitlab-sync]"
  SYNC_AUTHOR_EMAIL: "gitlab-github-sync@kejaksaan.go.id"
  GITHUB_REPO: "analisaperlengkapan/simpel2"

# Sync to GitHub job
sync-to-github:
  stage: sync
  image: alpine:latest
  variables:
    GIT_STRATEGY: clone
    GIT_DEPTH: 100
  before_script:
    - apk add --no-cache git curl jq openssh-client
  script:
    - |
      # Check if this is a sync commit (from GitHub)
      if echo "$CI_COMMIT_MESSAGE" | grep -q "$SYNC_PREFIX_GITHUB"; then
        echo "⏭️ Skipping: This is a sync commit from GitHub"
        exit 0
      fi

      # Configure Git
      git config --global user.email "$SYNC_AUTHOR_EMAIL"
      git config --global user.name "GitLab-GitHub Sync Bot"

      # Add GitHub remote
      git remote add github "https://x-access-token:${GITHUB_TOKEN}@github.com/${GITHUB_REPO}.git" || true
      git fetch github main --depth=100

      # Compare branches
      GITLAB_SHA=$(git rev-parse HEAD)
      GITHUB_SHA=$(git rev-parse github/main 2>/dev/null || echo "")

      if [ -z "$GITHUB_SHA" ]; then
        echo "⚠️ Could not fetch GitHub main branch"
        exit 0
      fi

      if [ "$GITLAB_SHA" = "$GITHUB_SHA" ]; then
        echo "✅ Branches are in sync"
        exit 0
      fi

      # Check if GitLab is ahead
      if git merge-base --is-ancestor github/main HEAD; then
        COMMITS=$(git log github/main..HEAD --oneline | wc -l)
        echo "📤 GitLab is ahead by $COMMITS commits"

        # Create sync branch
        SYNC_BRANCH="sync/from-gitlab-${CI_PIPELINE_ID}"
        git checkout -b "$SYNC_BRANCH"

        # Amend with sync marker
        git commit --amend -m "${SYNC_PREFIX_GITLAB} ${CI_COMMIT_TITLE}"

        # Push to GitHub
        git push github "$SYNC_BRANCH" --force

        # Create Pull Request via GitHub API
        RESPONSE=$(curl -s -w "\n%{http_code}" \
          -X POST \
          -H "Authorization: token $GITHUB_TOKEN" \
          -H "Accept: application/vnd.github.v3+json" \
          -d "{
            \"title\": \"${SYNC_PREFIX_GITLAB} Sync from GitLab main\",
            \"body\": \"## Automated Sync from GitLab\\n\\n**Source:** GitLab main branch\\n**Commits:** ${COMMITS}\\n**Pipeline:** ${CI_PIPELINE_URL}\\n\\n---\\n*This PR was created automatically by the bidirectional sync workflow.*\",
            \"head\": \"$SYNC_BRANCH\",
            \"base\": \"main\"
          }" \
          "https://api.github.com/repos/${GITHUB_REPO}/pulls")

        HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
        BODY=$(echo "$RESPONSE" | sed '$d')

        if [ "$HTTP_CODE" -ge 200 ] && [ "$HTTP_CODE" -lt 300 ]; then
          PR_URL=$(echo "$BODY" | jq -r '.html_url')
          echo "✅ Pull Request created: $PR_URL"
        else
          echo "❌ Failed to create PR: $BODY"
          exit 1
        fi
      else
        echo "⚠️ Branches have diverged - manual intervention may be needed"
      fi
  rules:
    - if: '$CI_COMMIT_BRANCH == "main"'
      when: on_success
    - when: never
  allow_failure: true
```

---

## 6. Required Secrets Configuration

### 6.1 GitHub Secrets

| Secret Name         | Description                                                                                | Required Permissions      |
| ------------------- | ------------------------------------------------------------------------------------------ | ------------------------- |
| `GITLAB_MIRROR_URL` | GitLab repo URL with auth: `https://oauth2:TOKEN@gitlab.kejaksaan.go.id/irhas/simpel2.git` | N/A                       |
| `GITLAB_API_TOKEN`  | GitLab Project Access Token or PAT                                                         | `api`, `write_repository` |

### 6.2 GitLab CI Variables

| Variable Name  | Description                            | Required Permissions          |
| -------------- | -------------------------------------- | ----------------------------- |
| `GITHUB_TOKEN` | GitHub Fine-grained PAT or Classic PAT | `repo`, `pull_requests:write` |

### 6.3 Token Creation Instructions

**GitLab Project Access Token:**

1. Go to Project → Settings → Access Tokens
2. Create token with:
   - Role: Maintainer
   - Scopes: `api`, `write_repository`
   - Expiration: Set appropriate date

**GitHub Fine-grained PAT:**

1. Go to Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Create token with:
   - Repository access: `analisaperlengkapan/simpel2`
   - Permissions:
     - Contents: Read and write
     - Pull requests: Read and write
     - Metadata: Read

---

## 7. Handling Edge Cases

### 7.1 Diverged Branches

When both GitHub and GitLab have unique commits:

```yaml
- name: Handle diverged branches
  run: |
    GITHUB_UNIQUE=$(git log gitlab/main..HEAD --oneline | wc -l)
    GITLAB_UNIQUE=$(git log HEAD..gitlab/main --oneline | wc -l)

    if [ "$GITHUB_UNIQUE" -gt 0 ] && [ "$GITLAB_UNIQUE" -gt 0 ]; then
      echo "⚠️ DIVERGED: GitHub has $GITHUB_UNIQUE unique, GitLab has $GITLAB_UNIQUE unique"

      # Option 1: Create issue for manual resolution
      # Option 2: Attempt automatic merge
      # Option 3: Use rebase strategy

      # Notify maintainers
      curl -X POST "$SLACK_WEBHOOK" \
        -d '{"text":"⚠️ GitHub/GitLab branches have diverged. Manual intervention required."}'
    fi
```

### 7.2 Merge Conflicts

```yaml
- name: Check for conflicts
  run: |
    # Attempt merge locally first
    git checkout -b test-merge
    if ! git merge gitlab/main --no-commit --no-ff 2>/dev/null; then
      echo "❌ Merge conflicts detected"
      git merge --abort

      # Create issue instead of MR
      # Or: Create MR with conflict warning
    fi
```

### 7.3 Large File Handling

```yaml
- name: Check for LFS files
  run: |
    # Ensure LFS is properly configured on both sides
    git lfs install
    git lfs fetch --all
    git lfs push --all gitlab
```

### 7.4 Protected Branches

If branches are protected, ensure:

1. Bot account has bypass permissions
2. Or use merge queue / auto-merge features
3. Or require manual approval

---

## 8. Monitoring and Alerts

### 8.1 Sync Status Dashboard

Create a scheduled workflow to check sync status:

```yaml
name: Sync Status Check
on:
  schedule:
    - cron: "0 */6 * * *" # Every 6 hours
  workflow_dispatch:

jobs:
  check-sync-status:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check sync status
        run: |
          git remote add gitlab "${{ secrets.GITLAB_MIRROR_URL }}" || true
          git fetch gitlab main

          GITHUB_SHA=$(git rev-parse HEAD)
          GITLAB_SHA=$(git rev-parse gitlab/main)

          if [ "$GITHUB_SHA" = "$GITLAB_SHA" ]; then
            echo "✅ Repositories are in sync"
          else
            echo "⚠️ Repositories are out of sync"
            # Send alert
          fi
```

### 8.2 Failure Notifications

```yaml
- name: Notify on failure
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    payload: |
      {
        "text": "❌ GitHub-GitLab sync failed",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "*Sync Failure Alert*\n\nWorkflow: ${{ github.workflow }}\nRun: ${{ github.run_id }}"
            }
          }
        ]
      }
```

---

## 9. Best Practices Summary

### ✅ DO

1. **Use PR/MR approach** - Never push directly to protected branches
2. **Implement multiple loop detection** - Commit prefix + author + branch name
3. **Use dedicated bot accounts** - Clear audit trail
4. **Set appropriate timeouts** - Prevent hanging jobs
5. **Implement retry logic** - Handle transient failures
6. **Log all operations** - Debugging and compliance
7. **Use concurrency controls** - Prevent race conditions
8. **Test in staging first** - Before production deployment

### ❌ DON'T

1. **Use `--mirror` for bidirectional sync** - Overwrites history
2. **Sync without comparison** - Creates unnecessary MRs/PRs
3. **Trust commit messages alone** - Can be manipulated
4. **Skip conflict detection** - Leads to failed merges
5. **Ignore LFS/submodules** - Causes sync failures
6. **Use short-lived tokens** - Requires frequent rotation

---

## 10. Implementation Roadmap

### Phase 1: Foundation (Week 1)

- [ ] Create dedicated bot accounts
- [ ] Generate and store API tokens
- [ ] Implement GitHub → GitLab sync with MR
- [ ] Test loop prevention

### Phase 2: Bidirectional (Week 2)

- [ ] Implement GitLab → GitHub sync with PR
- [ ] Add conflict detection
- [ ] Implement divergence handling
- [ ] Add monitoring dashboard

### Phase 3: Automation (Week 3)

- [ ] Enable auto-merge for clean syncs
- [ ] Add Slack/Teams notifications
- [ ] Create runbook for manual interventions
- [ ] Document recovery procedures

### Phase 4: Optimization (Week 4)

- [ ] Performance tuning
- [ ] Add metrics collection
- [ ] Implement scheduled sync checks
- [ ] Security audit of workflows

---

## 11. Conclusion

Bidirectional GitHub-GitLab synchronization is **feasible** using the PR/MR approach. The key challenges are:

1. **Loop prevention** - Solved with multi-layer detection (commit prefix, author, branch name)
2. **API integration** - Both platforms have robust APIs for creating MRs/PRs
3. **Conflict handling** - Requires careful design and monitoring
4. **Token management** - Use project access tokens with minimal required permissions

The recommended approach uses GitHub Actions as the primary automation platform (due to better self-hosted runner support) with GitLab CI/CD as a secondary trigger for GitLab-originated changes.

---

## Appendix A: Quick Reference

### API Endpoints

| Action     | Platform | Endpoint                                               |
| ---------- | -------- | ------------------------------------------------------ |
| Create MR  | GitLab   | `POST /api/v4/projects/:id/merge_requests`             |
| Create PR  | GitHub   | `POST /repos/:owner/:repo/pulls`                       |
| Get branch | GitLab   | `GET /api/v4/projects/:id/repository/branches/:branch` |
| Get ref    | GitHub   | `GET /repos/:owner/:repo/git/ref/heads/:branch`        |
| Accept MR  | GitLab   | `PUT /api/v4/projects/:id/merge_requests/:iid/merge`   |
| Merge PR   | GitHub   | `PUT /repos/:owner/:repo/pulls/:number/merge`          |

### Sync Markers

| Marker               | Meaning                                         |
| -------------------- | ----------------------------------------------- |
| `[github-sync]`      | Commit originated from GitHub, synced to GitLab |
| `[gitlab-sync]`      | Commit originated from GitLab, synced to GitHub |
| `sync/from-github-*` | Branch created for GitHub → GitLab sync         |
| `sync/from-gitlab-*` | Branch created for GitLab → GitHub sync         |

---

_Document generated: 2024-12-01_
_Last updated: 2024-12-01_
_Version: 1.0.0_
