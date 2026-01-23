# CI/CD Environment Injection

This document describes how to use Secreton's CI/CD environment injection feature to securely inject secrets as environment variables into CI/CD pipelines with automatic cleanup.

## Overview

The CI/CD injection feature provides:
- **Secure Secret Injection**: Fetch secrets from Secreton and inject them as environment variables
- **Automatic Cleanup**: Secrets are automatically cleaned up after a configurable TTL
- **Session Tracking**: Track injection sessions for audit and monitoring
- **Flexible Formatting**: Support for flat and nested environment variable formats
- **Key Mapping**: Map secret keys to custom environment variable names

## API Endpoints

### Inject Secrets

**POST** `/v1/inject/env`

Inject secrets as environment variables for a CI/CD job.

**Request Body**:
```json
{
  "secrets": [
    {
      "path": "/secret/data/database/prod",
      "key": "password",
      "env_name": "DB_PASSWORD"
    },
    {
      "path": "/secret/data/api/keys"
    }
  ],
  "job_id": "ci-job-12345",
  "ttl": 3600,
  "prefix": "SECRET_",
  "format": "flat"
}
```

**Parameters**:
- `secrets` (array, required): List of secrets to inject
  - `path` (string, required): Path to the secret in Secreton
  - `key` (string, optional): Specific key within the secret (if omitted, all keys are injected)
  - `env_name` (string, optional): Custom environment variable name
- `job_id` (string, required): Unique identifier for the CI/CD job
- `ttl` (integer, optional): Time-to-live in seconds (default: 3600)
- `prefix` (string, optional): Prefix for environment variable names (default: "SECRET_")
- `format` (string, optional): Format for environment variables ("flat" or "nested", default: "flat")

**Response**:
```json
{
  "success": true,
  "data": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "env_vars": {
      "DB_PASSWORD": "secret123",
      "SECRET_API_KEY": "key456",
      "SECRET_API_SECRET": "secret789"
    },
    "expires_at": "2024-01-01T01:00:00Z",
    "cleanup_url": "/v1/inject/cleanup/550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### Cleanup Session

**DELETE** `/v1/inject/cleanup/{session_id}`

Manually cleanup an injection session before it expires.

**Response**:
```json
{
  "success": true,
  "data": null
}
```

### List Active Sessions

**GET** `/v1/inject/sessions`

List all active injection sessions.

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "job_id": "ci-job-12345",
      "created_at": "2024-01-01T00:00:00Z",
      "expires_at": "2024-01-01T01:00:00Z",
      "secret_paths": ["/secret/data/database/prod"],
      "active": true
    }
  ]
}
```

### Get Session Details

**GET** `/v1/inject/sessions/{session_id}`

Get details of a specific injection session.

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "job_id": "ci-job-12345",
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-01-01T01:00:00Z",
    "secret_paths": ["/secret/data/database/prod"],
    "active": true
  }
}
```

## Usage Examples

### GitLab CI/CD

```yaml
variables:
  SECRETON_ADDR: "https://secreton.internal:8200"
  SECRETON_TOKEN: "${CI_JOB_JWT}"

stages:
  - deploy

deploy:
  stage: deploy
  script:
    # Inject secrets
    - |
      RESPONSE=$(curl -X POST "${SECRETON_ADDR}/v1/inject/env" \
        -H "X-Secret Vault-Token: ${SECRETON_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
          "secrets": [
            {"path": "/secret/data/database/prod", "key": "password", "env_name": "DB_PASSWORD"},
            {"path": "/secret/data/api/keys"}
          ],
          "job_id": "'${CI_JOB_ID}'",
          "ttl": 3600
        }')

    # Extract session ID and environment variables
    - SESSION_ID=$(echo $RESPONSE | jq -r '.data.session_id')
    - echo $RESPONSE | jq -r '.data.env_vars | to_entries[] | "\(.key)=\(.value)"' > .env

    # Source environment variables
    - export $(cat .env | xargs)

    # Run deployment
    - ./deploy.sh

    # Cleanup
    - |
      curl -X DELETE "${SECRETON_ADDR}/v1/inject/cleanup/${SESSION_ID}" \
        -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"
```

### GitHub Actions

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Inject Secrets
        id: secrets
        run: |
          RESPONSE=$(curl -X POST "${{ secrets.SECRETON_ADDR }}/v1/inject/env" \
            -H "X-Secret Vault-Token: ${{ secrets.SECRETON_TOKEN }}" \
            -H "Content-Type: application/json" \
            -d '{
              "secrets": [
                {"path": "/secret/data/database/prod"},
                {"path": "/secret/data/api/keys"}
              ],
              "job_id": "${{ github.run_id }}",
              "ttl": 3600
            }')

          echo "session_id=$(echo $RESPONSE | jq -r '.data.session_id')" >> $GITHUB_OUTPUT
          echo $RESPONSE | jq -r '.data.env_vars | to_entries[] | "\(.key)=\(.value)"' >> $GITHUB_ENV

      - name: Deploy Application
        run: ./deploy.sh

      - name: Cleanup Secrets
        if: always()
        run: |
          curl -X DELETE "${{ secrets.SECRETON_ADDR }}/v1/inject/cleanup/${{ steps.secrets.outputs.session_id }}" \
            -H "X-Secret Vault-Token: ${{ secrets.SECRETON_TOKEN }}"
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    environment {
        SECRETON_ADDR = 'https://secreton.internal:8200'
        SECRETON_TOKEN = credentials('secreton-token')
    }

    stages {
        stage('Inject Secrets') {
            steps {
                script {
                    def response = sh(
                        script: """
                            curl -X POST "${SECRETON_ADDR}/v1/inject/env" \
                              -H "X-Secret Vault-Token: ${SECRETON_TOKEN}" \
                              -H "Content-Type: application/json" \
                              -d '{
                                "secrets": [
                                  {"path": "/secret/data/database/prod"},
                                  {"path": "/secret/data/api/keys"}
                                ],
                                "job_id": "${BUILD_ID}",
                                "ttl": 3600
                              }'
                        """,
                        returnStdout: true
                    ).trim()

                    def json = readJSON text: response
                    env.SESSION_ID = json.data.session_id

                    json.data.env_vars.each { key, value ->
                        env[key] = value
                    }
                }
            }
        }

        stage('Deploy') {
            steps {
                sh './deploy.sh'
            }
        }
    }

    post {
        always {
            script {
                if (env.SESSION_ID) {
                    sh """
                        curl -X DELETE "${SECRETON_ADDR}/v1/inject/cleanup/${SESSION_ID}" \
                          -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"
                    """
                }
            }
        }
    }
}
```

### CircleCI

```yaml
version: 2.1

jobs:
  deploy:
    docker:
      - image: cimg/base:stable
    steps:
      - checkout

      - run:
          name: Inject Secrets
          command: |
            RESPONSE=$(curl -X POST "${SECRETON_ADDR}/v1/inject/env" \
              -H "X-Secret Vault-Token: ${SECRETON_TOKEN}" \
              -H "Content-Type: application/json" \
              -d '{
                "secrets": [
                  {"path": "/secret/data/database/prod"},
                  {"path": "/secret/data/api/keys"}
                ],
                "job_id": "'${CIRCLE_WORKFLOW_ID}'",
                "ttl": 3600
              }')

            echo $RESPONSE | jq -r '.data.session_id' > /tmp/session_id
            echo $RESPONSE | jq -r '.data.env_vars | to_entries[] | "export \(.key)=\(.value)"' >> $BASH_ENV

      - run:
          name: Deploy
          command: ./deploy.sh

      - run:
          name: Cleanup
          when: always
          command: |
            SESSION_ID=$(cat /tmp/session_id)
            curl -X DELETE "${SECRETON_ADDR}/v1/inject/cleanup/${SESSION_ID}" \
              -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"

workflows:
  deploy:
    jobs:
      - deploy
```

## Environment Variable Formats

### Flat Format (Default)

With flat format, all keys are prefixed with the specified prefix:

```json
{
  "format": "flat",
  "prefix": "SECRET_"
}
```

Result:
```
SECRET_DB_HOST=localhost
SECRET_DB_PORT=5432
SECRET_API_KEY=key123
```

### Nested Format

With nested format, the secret path is included in the variable name:

```json
{
  "format": "nested",
  "prefix": "SECRET_"
}
```

For secret at `/secret/data/database/prod`:
```
SECRET_DATABASE_PROD_DB_HOST=localhost
SECRET_DATABASE_PROD_DB_PORT=5432
```

## Security Considerations

### 1. Use Short TTLs

Set TTL to the minimum required for your job:

```json
{
  "ttl": 1800  // 30 minutes
}
```

### 2. Always Cleanup

Always cleanup sessions in a `finally` or `always` block:

```bash
trap 'curl -X DELETE "${SECRETON_ADDR}/v1/inject/cleanup/${SESSION_ID}" \
  -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"' EXIT
```

### 3. Use Job-Specific Tokens

Use CI/CD platform's native authentication when possible:

```yaml
# GitLab CI with JWT
SECRETON_TOKEN: "${CI_JOB_JWT}"

# GitHub Actions with OIDC
- uses: hashicorp/engine-action@v2
  with:
    url: https://secreton.internal:8200
    method: jwt
```

### 4. Audit Injection Sessions

Regularly review active sessions:

```bash
curl -X GET "${SECRETON_ADDR}/v1/inject/sessions" \
  -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"
```

### 5. Limit Secret Access

Use policies to restrict which secrets can be injected:

```hcl
path "secret/data/ci/*" {
  capabilities = ["read"]
}

path "secret/data/production/*" {
  capabilities = ["deny"]
}
```

## Monitoring and Troubleshooting

### Check Session Status

```bash
curl -X GET "${SECRETON_ADDR}/v1/inject/sessions/${SESSION_ID}" \
  -H "X-Secret Vault-Token: ${SECRETON_TOKEN}"
```

### List Active Sessions

```bash
curl -X GET "${SECRETON_ADDR}/v1/inject/sessions" \
  -H "X-Secret Vault-Token: ${SECRETON_TOKEN}" | jq
```

### Common Issues

#### Session Not Found

**Error**: `Session not found or expired`

**Solution**: The session may have expired. Check the TTL and ensure cleanup is called before expiration.

#### Permission Denied

**Error**: `Permission denied`

**Solution**: Verify the token has read access to the requested secret paths.

#### Invalid Secret Path

**Error**: `Secret not found`

**Solution**: Verify the secret path exists and is accessible.

## Best Practices

1. **Use Unique Job IDs**: Always use unique identifiers for `job_id` to track sessions
2. **Set Appropriate TTLs**: Match TTL to expected job duration plus buffer
3. **Implement Cleanup**: Always cleanup sessions, even on failure
4. **Use Specific Keys**: Request only the keys you need instead of entire secrets
5. **Audit Regularly**: Review injection sessions and audit logs
6. **Rotate Tokens**: Regularly rotate CI/CD authentication tokens
7. **Use Namespaces**: Isolate CI/CD secrets in dedicated namespaces

## Requirements

Validates: Requirements 5.3

## Related Documentation

- [Authentication Guide](./AUTH_GUIDE.md)
- [Policy Management](./POLICY_API.md)
- [Audit Logging](./AUDIT_GUIDE.md)
