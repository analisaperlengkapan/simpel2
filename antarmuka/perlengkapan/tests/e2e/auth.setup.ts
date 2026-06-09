/**
 * Playwright "setup" project (F5-A): authenticate the seeded user against the
 * REAL authenc REST API and persist the resulting JWT as storageState, so the
 * smoke project starts already-authenticated with a backend-valid session.
 *
 * Requires the e2e stack up: docker-compose.e2e.yml (authenc REST on :18088).
 */
import { test as setup } from '@playwright/test';
import { seedRealAuth } from './helpers/real-auth';

const STORAGE_STATE = 'results/.auth/perlengkapan.json';

setup('authenticate via real authenc login', async ({ browser }) => {
  const context = await browser.newContext();
  await seedRealAuth(context); // API login → inject real JWT into localStorage
  await context.storageState({ path: STORAGE_STATE });
  await context.close();
});
