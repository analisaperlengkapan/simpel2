import { defineConfig, devices } from '@playwright/test';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

/** storageState produced by auth.setup.ts (real authenc login → JWT in localStorage). */
const STORAGE_STATE = 'results/.auth/perlengkapan.json';

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

  /* Configure projects by test category */
  projects: [
    // Real-auth setup: logs the seeded user in against authenc and writes the
    // JWT into Perlengkapan localStorage, saved as storageState for reuse.
    {
      name: 'setup',
      use: { ...devices['Desktop Chrome'] },
      testMatch: '**/auth.setup.ts',
    },
    // F5-A SMOKE — real-auth merge gate (guards regression #482). Runs against
    // the real e2e stack (docker-compose.e2e.yml). Authenticated specs reuse the
    // storageState from `setup`.
    {
      name: 'perlengkapan-smoke',
      use: { ...devices['Desktop Chrome'], storageState: STORAGE_STATE },
      dependencies: ['setup'],
      testMatch: ['**/guards-smoke.spec.ts'],
    },
    // F5-C per-role RoleGate regression (#482 guard dedup, beyond the
    // single-route AuthGate smoke). Browser-driven: each test reuses a per-role
    // storageState (real JWT) and asserts the /admin/* gate (client-side
    // is_admin check). Depends on `setup` for the per-role storageState files.
    {
      name: 'perlengkapan-guards-rbac',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
      testMatch: ['**/guards-rbac.spec.ts'],
    },
    // F5-C RBAC data-scoping — server-side enforcement (#66 / #565 / #566).
    // Pure API (no browser, no storageState): each test logs the relevant
    // per-role seed user in against authenc and asserts the backend scopes
    // `bank_aset` correctly. Runs against the real e2e stack with the
    // multi-satker fixture loaded. No `setup` dependency (does its own logins).
    {
      name: 'perlengkapan-rbac',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/rbac-scoping.spec.ts'],
    },
    // F5-C UI-layer data-scoping (Layer-3): the FE bank-aset list must RENDER
    // only the in-scope rows. Browser-driven, per-role storageState; needs the
    // compose FE→BE upstream fix so the WASM's origin-relative API call resolves.
    {
      name: 'perlengkapan-ui-rbac',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup'],
      testMatch: ['**/rbac-scoping-ui.spec.ts'],
    },
    // LEGACY mock UI specs — kept for the F5-C comprehensive rewrite. They use
    // the dead mock session and are NOT part of the smoke gate; do not run them
    // against the real stack until rewritten (F5-C / task #33).
    {
      name: 'perlengkapan-chromium',
      use: { ...devices['Desktop Chrome'] },
      testMatch: [
        '**/ui-*.spec.ts',
        '**/dashboard-navigation.spec.ts'
      ],
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
