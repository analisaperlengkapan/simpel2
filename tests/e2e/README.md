# E2E Tests - Playwright

End-to-end tests for SIMPEL using Playwright.

## Overview

This directory contains comprehensive E2E tests covering:
- **API Fast Tests**: Edge cases, validation, search, batch operations
- **API Business Tests**: Complete business process workflows
- **UI E2E Tests**: Full user interface interactions
- **Portal Auth Tests**: Authentication and authorization flows

## Test Structure

```
tests/e2e/
├── playwright.config.ts          # Playwright configuration
├── package.json                  # Node.js dependencies
├── fixtures/                     # Test fixtures and helpers
├── tests/                        # Test files organized by feature
├── business-process-*.spec.ts    # Business process tests
├── edge-cases-*.spec.ts          # Edge case tests
└── search-filter-*.spec.ts       # Search and filter tests
```

## Prerequisites

### Local Development

1. **Node.js 20+**
   ```bash
   node --version  # Should be 20.x or higher
   ```

2. **PostgreSQL 15+**
   ```bash
   # Create test database
   createdb simpelv2_e2e
   ```

3. **Redis 7+**
   ```bash
   # Start Redis
   redis-server
   ```

4. **Backend Services**
   ```bash
   # Build and run layanan-perlengkapan-api
   cargo build --release --bin layanan-perlengkapan-api
   DATABASE_URL=postgres://user:pass@localhost:5432/simpelv2_e2e \
   REDIS_URL=redis://localhost:6379 \
   ./target/release/layanan-perlengkapan-api
   ```

### CI/CD Environment

The tests are automatically configured to run in:
- **GitHub Actions**: `.github/workflows/e2e-tests.yml`
- **GitLab CI**: `.gitlab-ci.yml` (e2e-test stage)

## Installation

```bash
cd tests/e2e
npm install
npx playwright install --with-deps chromium
```

## Running Tests

### Run All Tests
```bash
npx playwright test
```

### Run Specific Test Suite
```bash
# API fast tests (no browser)
npx playwright test --project=api-fast

# API business process tests
npx playwright test --project=api-e2e

# UI E2E tests (requires frontend)
npx playwright test --project=ui-e2e

# Portal authentication tests
npx playwright test --project=portal-auth
```

### Run Specific Test File
```bash
npx playwright test business-process-kebutuhan-bmn.spec.ts
```

### Debug Mode
```bash
npx playwright test --debug
```

### UI Mode (Interactive)
```bash
npx playwright test --ui
```

## Test Projects

### 1. api-fast
Pure API tests without browser rendering. Fastest execution.

**Tests:**
- `edge-cases-validation.spec.ts`
- `workflow-edge-cases.spec.ts`
- `search-filter-pagination.spec.ts`
- `batch-operations-export.spec.ts`

**Environment:**
- `API_BASE_URL`: Backend API endpoint (default: http://localhost:8093)

### 2. api-e2e
API business process tests with screenshot documentation.

**Tests:**
- `business-process-kebutuhan-bmn.spec.ts`
- `business-process-pemakaian-bmn.spec.ts`
- `business-process-penghapusan-bmn.spec.ts`
- `business-process-pakaian-dinas.spec.ts`
- `business-process-integration.spec.ts`

**Environment:**
- `API_BASE_URL`: Backend API endpoint

### 3. ui-e2e
Full UI tests requiring browser and running frontend.

**Tests:**
- `tests/auth-login-flow.spec.ts`
- `tests/dashboard-navigation.spec.ts`
- `tests/ui-kebutuhan-bmn.spec.ts`
- `tests/ui-pakaian-dinas.spec.ts`
- `tests/ui-pemakaian-bmn.spec.ts`
- `tests/ui-penghapusan-bmn.spec.ts`

**Environment:**
- `API_BASE_URL`: Backend API endpoint
- `FRONTEND_URL`: Frontend URL (default: http://localhost:8080)

### 4. portal-auth
Portal authentication and authorization tests.

**Tests:**
- `tests/portal-auth-e2e.spec.ts`

**Environment:**
- `PORTAL_URL`: Portal URL (default: http://localhost:18080)

## Environment Variables

```bash
# Backend API
export API_BASE_URL=http://localhost:8093

# Frontend
export FRONTEND_URL=http://localhost:8080

# Portal
export PORTAL_URL=http://localhost:18080

# Database
export DATABASE_URL=postgres://simpelv2_e2e:e2e_test_2024@localhost:5432/simpelv2_e2e

# Redis
export REDIS_URL=redis://localhost:6379

# CI mode
export CI=true
```

## Test Reports

### HTML Report
```bash
npx playwright show-report
```

The HTML report includes:
- Test results with pass/fail status
- Screenshots for visual verification
- Videos for failed tests
- Execution timeline
- Error details with stack traces

### JSON Report
```bash
# Generate JSON report
npx playwright test --reporter=json

# View results
cat test-results/results.json | jq
```

## Artifacts

Test artifacts are stored in:
- `test-results/`: Test execution results
- `test-results/artifacts/`: Screenshots and videos
- `playwright-report/`: HTML report

### CI/CD Artifacts

In CI/CD pipelines, artifacts are automatically uploaded:
- **GitHub Actions**: Available in workflow run artifacts
- **GitLab CI**: Available in job artifacts (7 days retention)

## Database Setup

### Migrations

Migrations are automatically run in CI/CD. For local development:

```bash
cd layanan/perlengkapan/crates/api
cargo install sqlx-cli --no-default-features --features postgres
sqlx database create
sqlx migrate run
```

### Test Data Seeding

Test data is seeded automatically in tests using fixtures. See `fixtures/` directory.

## Troubleshooting

### Tests Fail with Connection Refused

**Problem:** Backend service not running or not accessible.

**Solution:**
```bash
# Check if API is running
curl http://localhost:8093/health

# Start backend if not running
./target/release/layanan-perlengkapan-api
```

### Database Connection Errors

**Problem:** PostgreSQL not running or wrong credentials.

**Solution:**
```bash
# Check PostgreSQL status
pg_isready -h localhost -p 5432

# Verify database exists
psql -l | grep simpelv2_e2e

# Create database if missing
createdb simpelv2_e2e
```

### Playwright Browser Not Found

**Problem:** Playwright browsers not installed.

**Solution:**
```bash
npx playwright install --with-deps chromium
```

### Tests Timeout

**Problem:** Backend service slow to start or respond.

**Solution:**
- Increase timeout in `playwright.config.ts`
- Check backend logs for errors
- Verify database migrations completed

### Screenshots Not Generated

**Problem:** Screenshot configuration not enabled.

**Solution:**
- Check `playwright.config.ts` has `screenshot: 'on'`
- Verify test project configuration
- Check `test-results/artifacts/` directory

## CI/CD Integration

### GitHub Actions

Workflow: `.github/workflows/e2e-tests.yml`

**Triggers:**
- Push to `main` or `develop`
- Pull requests
- Manual workflow dispatch

**Jobs:**
1. `setup-test-environment`: Setup PostgreSQL and Redis
2. `build-backend`: Build Rust backend services
3. `playwright-api-tests`: Run API tests (api-fast, api-e2e)
4. `playwright-ui-tests`: Run UI tests (ui-e2e)
5. `test-report`: Generate summary report

**Artifacts:**
- Test results (HTML, JSON)
- Screenshots and videos
- Backend logs

### GitLab CI

Stage: `e2e-test` in `.gitlab-ci.yml`

**Jobs:**
1. `e2e:setup`: Setup test environment
2. `e2e:api-fast`: Run fast API tests
3. `e2e:api-business`: Run business process tests
4. `e2e:ui-tests`: Run UI tests (manual trigger)
5. `e2e:report`: Generate summary

**Services:**
- PostgreSQL 15
- Redis 7

**Artifacts:**
- Test results (1 week retention)
- Screenshots and videos
- API and frontend logs

## Best Practices

### Writing Tests

1. **Use descriptive test names**
   ```typescript
   test('should create kebutuhan BMN submission with valid data', async ({ request }) => {
     // Test implementation
   });
   ```

2. **Use fixtures for test data**
   ```typescript
   import { testUsers } from './fixtures/users';
   ```

3. **Clean up after tests**
   ```typescript
   test.afterEach(async ({ request }) => {
     // Cleanup test data
   });
   ```

4. **Use proper assertions**
   ```typescript
   expect(response.status()).toBe(200);
   expect(data).toHaveProperty('id');
   ```

5. **Add screenshots for visual verification**
   ```typescript
   await page.screenshot({ path: 'test-results/screenshot.png' });
   ```

### Performance

1. **Use API tests when possible** (faster than UI tests)
2. **Run tests in parallel** (configured in `playwright.config.ts`)
3. **Use `api-fast` project** for quick feedback
4. **Limit UI tests** to critical user flows

### Debugging

1. **Use `--debug` flag** for step-by-step execution
2. **Use `--ui` mode** for interactive debugging
3. **Check backend logs** in `api.log`
4. **Review screenshots** in `test-results/artifacts/`
5. **Use `page.pause()`** to pause execution

## Contributing

When adding new tests:

1. Follow existing test structure
2. Add tests to appropriate project (api-fast, api-e2e, ui-e2e)
3. Update this README if adding new test categories
4. Ensure tests pass locally before committing
5. Add appropriate fixtures and helpers

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Playwright API Reference](https://playwright.dev/docs/api/class-playwright)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [SIMPEL API Documentation](../../docs/)

## Support

For issues or questions:
- Check existing test files for examples
- Review Playwright documentation
- Check CI/CD logs for detailed error messages
- Contact the development team
