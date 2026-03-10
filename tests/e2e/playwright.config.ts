import { defineConfig, devices } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: '.',
  timeout: 60000,
  expect: {
    timeout: 10000,
  },
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : 4,
  reporter: [
    ['html', { outputFolder: path.join(__dirname, 'test-results', 'html-report') }],
    ['list'],
  ],
  outputDir: path.join(__dirname, 'test-results', 'artifacts'),
  use: {
    baseURL: process.env.API_BASE_URL || 'http://localhost:8093',
    trace: 'on-first-retry',
    extraHTTPHeaders: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
  },
  projects: [
    // Pure API tests - no browser, fastest execution
    {
      name: 'api-fast',
      use: {},
      testMatch: [
        'edge-cases-validation.spec.ts',
        'workflow-edge-cases.spec.ts',
        'search-filter-pagination.spec.ts',
        'batch-operations-export.spec.ts',
      ],
    },
    // API business-process tests with screenshot documentation
    {
      name: 'api-e2e',
      use: {
        ...devices['Desktop Chrome'],
        screenshot: 'on',
      },
      testMatch: [
        'business-process-kebutuhan-bmn.spec.ts',
        'business-process-pemakaian-bmn.spec.ts',
        'business-process-penghapusan-bmn.spec.ts',
        'business-process-pakaian-dinas.spec.ts',
        'business-process-integration.spec.ts',
      ],
    },
    // Legacy UI-based tests (require browser + running frontend)
    {
      name: 'legacy',
      use: { ...devices['Desktop Chrome'] },
      testMatch: [
        'tests/perlengkapan.spec.ts',
      ],
    },
    // Full UI E2E tests (require browser + running perlengkapan WASM frontend)
    {
      name: 'ui-e2e',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: process.env.FRONTEND_URL || 'http://localhost:8080',
        screenshot: 'on',
      },
      testMatch: [
        'tests/auth-login-flow.spec.ts',
        'tests/dashboard-navigation.spec.ts',
        'tests/ui-kebutuhan-bmn.spec.ts',
        'tests/ui-pakaian-dinas.spec.ts',
        'tests/ui-pemakaian-bmn.spec.ts',
        'tests/ui-penghapusan-bmn.spec.ts',
        'tests/ui-bank-aset-pengelolaan.spec.ts',
        'tests/ui-admin-role-management.spec.ts',
      ],
    },
    // Portal authentication E2E tests (real authenc backend, captcha solving)
    {
      name: 'portal-auth',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: process.env.PORTAL_URL || 'http://localhost:18080',
        screenshot: 'on',
        video: 'on-first-retry',
      },
      testMatch: [
        'tests/portal-auth-e2e.spec.ts',
      ],
    },
    // Portal screenshot E2E (all pages, staging gateway)
    {
      name: 'portal-screenshots',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: process.env.PORTAL_URL || 'http://10.1.7.121',
        screenshot: 'on',
        video: 'on-first-retry',
        viewport: { width: 1920, height: 1080 },
      },
      testMatch: [
        'tests/portal-screenshots-e2e.spec.ts',
      ],
    },
    // Integration tests: layanan-integrasi ↔ authenc (gRPC + REST API)
    {
      name: 'integrasi',
      use: {
        baseURL: process.env.AUTHENC_URL || 'http://localhost:18088',
      },
      testMatch: [
        'tests/integrasi-authenc-e2e.spec.ts',
      ],
    },
  ],
});
