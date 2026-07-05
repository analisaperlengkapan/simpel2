import { defineConfig, devices } from '@playwright/test';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: '.',
  testMatch: ['**/*.spec.ts'],
  testIgnore: [
    '**/node_modules/**',
    '**/playwright-report/**',
    '**/test-results/**',
    '**/results/**',
  ],
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['junit', { outputFile: 'results/junit.xml' }],
    ['list'],
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.FRONTEND_URL || process.env.BASE_URL || 'http://localhost:8080',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',

    /* Video on failure */
    video: 'retain-on-failure',

    /* Maximum time each action can take */
    actionTimeout: 10000,

    /* Maximum time for navigation */
    navigationTimeout: 30000,
  },

  /* Configure projects by test category.
   *
   * Only `portal-chromium` is the CI gate (e2e-portal job): the genuine portal
   * FE real-auth login flow (captcha via debug endpoint, /api/** proxied to
   * authenc by page.route) — needs only portal + authenc. The other specs need
   * MORE than the portal stack and are split into their own non-gating projects
   * so the gate's stack matches what it runs:
   *   - portal-cross-app  (auth-login-flow): REAL cross-app SSO (portal login
   *     → perlengkapan dashboard via the same-origin JWT). Runs in the
   *     e2e-portal-cross-app job against the single-origin `cross-app-ingress`
   *     (both FEs + /api on one origin, mirroring the prod Istio VS); baseURL
   *     is that ingress (BASE_URL env).
   *   - portal-integration (integrasi-authenc-e2e): gRPC against
   *     layanan-integrasi → needs that service up (backend integration).
   *   - portal-screenshots: visual artifacts, not assertions.
   */
  projects: [
    {
      name: 'portal-chromium',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/portal-auth-e2e.spec.ts'],
    },
    {
      name: 'portal-cross-app',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/auth-login-flow.spec.ts'],
    },
    {
      name: 'portal-integration',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/integrasi-authenc-e2e.spec.ts'],
    },
    {
      name: 'portal-screenshots',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/portal-screenshots-e2e.spec.ts'],
    },
  ],

  /* Run your local dev server before starting the tests */
  // webServer: {
  //   command: 'npm run start',
  //   url: 'http://127.0.0.1:8080',
  //   reuseExistingServer: !process.env.CI,
  // },

  /* Global timeout for each test */
  timeout: 60000,

  /* Expect timeout */
  expect: {
    timeout: 5000,
  },

  /* Output folder for test artifacts */
  outputDir: 'test-results/',
});
