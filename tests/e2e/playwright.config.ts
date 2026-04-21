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

  /* Configure projects by test category */
  projects: [
    {
      name: 'api-chromium',
      use: { ...devices['Desktop Chrome'] },
      testMatch: [
        '**/business-process-*.spec.ts',
        '**/edge-cases-validation.spec.ts',
        '**/workflow-edge-cases.spec.ts',
        '**/search-filter-pagination.spec.ts',
        '**/batch-operations-export.spec.ts',
      ],
    },
    {
      name: 'ui-chromium',
      use: { ...devices['Desktop Chrome'] },
      testMatch: ['**/tests/**/*.spec.ts'],
    },

    // Uncomment to test on other browsers
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },
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
