# SIMPEL v1/v2 E2E Test Suite

## Overview

This test suite validates the integration between SIMPEL v1 (Legacy PHP) and v2 (Modern Rust) using Playwright for end-to-end testing.

## Test Coverage

### 1. Portal Navigation & Version Selection (4 tests)

- ✓ Portal loads successfully
- ✓ Portal shows v1/v2 selector in perlengkapan app
- ✓ Portal submenu shows v1 and v2 options
- ✓ User can expand/collapse submenu with smooth animation

### 2. SIMPEL v1 OAuth Flow (4 tests)

- ✓ v1 redirects to Portal login when not authenticated
- ✓ v1 OAuth callback accepts valid token
- ✓ v1 creates Laravel session from JWT claims
- ✓ v1 health check endpoint responds correctly

### 3. SIMPEL v2 Navigation (1 test)

- ✓ v2 loads successfully when authenticated

### 4. Cross-Version Session Management (1 test)

- ✓ Session invalidation in v1 detected in Portal (via localStorage broadcast)

### 5. gRPC Service Integration (1 test)

- ✓ v1 connects to Authenc service on startup

### 6. Error Handling (2 tests)

- ✓ v1 handles invalid JWT token gracefully
- ✓ v1 shows appropriate error messages

### 7. Performance & Load (2 tests)

- ✓ v1 health endpoint responds within 100ms
- ✓ v1 dashboard loads within 3 seconds

### 8. Accessibility (1 test)

- ✓ v1 login form is keyboard accessible

### 9. Security (1 test)

- ✓ v1 CSRF token present in forms
- ✓ v1 sets secure session cookie with httpOnly and secure flags

## Running Tests

### Prerequisites

```bash
# Install Playwright
npm install -D @playwright/test

# Ensure services are running
docker-compose up -d

# Or run manually
cargo run --bin layasan-perlengkapan
cd monolith/simpelv1 && php artisan serve
```

### Run All Tests

```bash
npx playwright test
```

### Run Specific Test File

```bash
npx playwright test tests/e2e/simpelv1-integration.spec.ts
```

### Run Specific Test

```bash
npx playwright test -g "Portal shows v1/v2 selector"
```

### Run with Browser UI (Debug Mode)

```bash
npx playwright test --ui
```

### Run Tests in Headed Mode

```bash
npx playwright test --headed
```

## Test Results

Tests generate reports in multiple formats:

- **HTML Report**: `playwright-report/index.html`
- **JUnit XML**: `test-results/junit.xml`
- **GitHub**: Integrated with CI/CD

View HTML report:

```bash
npx playwright show-report
```

## Performance Benchmarks

Target metrics for SIMPEL v1/v2 integration:

| Metric | Target | Actual |
|--------|--------|--------|
| Health endpoint response | < 100ms | — |
| Dashboard load time | < 3s | — |
| OAuth callback | < 2s | — |
| Form submission | < 1s | — |

## Troubleshooting

### Services Not Starting

```bash
# Check if ports are in use
lsof -i :3000 :8000 :3020 :50051

# Kill processes if needed
kill -9 <PID>

# Start fresh
docker-compose down && docker-compose up
```

### Tests Timing Out

Increase test timeout in `playwright.config.ts`:

```typescript
use: {
  navigationTimeout: 30000,
  actionTimeout: 10000,
}
```

### CSRF Token Errors

Ensure CSRF middleware is enabled in Laravel:

```bash
# In monolith/simpelv1
php artisan config:cache
```

### gRPC Connection Errors

Verify gRPC services are running:

```bash
# Check service health
grpcurl -plaintext localhost:50051 list
grpcurl -plaintext localhost:50053 list
grpcurl -plaintext localhost:50052 list
```

## CI/CD Integration

### GitLab CI

Tests run in the `e2e-test` stage:

```yaml
e2e:simpelv1:
  stage: e2e-test
  script:
    - npx playwright test
```

### GitHub Actions

Tests run via `.github/workflows/e2e.yml`:

```bash
npm run test:e2e
```

## Contributing

When adding new tests:

1. Follow the existing test structure
2. Use descriptive test names
3. Add appropriate test.describe() groups
4. Include comments for complex test logic
5. Ensure tests are idempotent (can run multiple times)
6. Test both happy path and error scenarios
7. Consider accessibility (keyboard navigation, screen readers)
8. Test on multiple browsers if critical functionality

## References

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [SIMPEL Architecture](./AGENTS.md)
- [Laravel Testing](https://laravel.com/docs/testing)
