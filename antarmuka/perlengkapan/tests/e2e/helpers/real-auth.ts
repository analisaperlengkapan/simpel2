/**
 * Real-auth helper for Perlengkapan e2e (F5-A).
 *
 * Replaces the legacy MOCK (`helpers/session.ts`, which injected a
 * `mock-jwt-token` + `user_session`/`active_role` keys the app no longer reads).
 * Perlengkapan's `AuthService::load_session()` now decodes the JWT in the
 * `auth_token` localStorage key directly (see `antarmuka/perlengkapan/src/
 * features/auth.rs`), so a fake token authenticates nothing.
 *
 * This helper logs in the seeded test user against the REAL authenc REST API
 * (`POST /api/v1/auth/login`, no captcha at the API level) and returns the real
 * JWT, which is then placed in `auth_token` so both the WASM app and the backend
 * accept the session. Seed: migrations 044/047 — NIP 199203142014031001
 * (password = the NIP), roles operator_satker/validator_wilayah/validator_pusat/
 * admin.
 *
 * Stack: brought up via `docker-compose.e2e.yml` (authenc REST on :18088).
 */
import type { APIRequestContext, BrowserContext } from '@playwright/test';

/** authenc REST base — the e2e overlay publishes it on host :18088. */
export const AUTHENC_URL = process.env.AUTHENC_URL || 'http://localhost:18088';

/** Perlengkapan FE base — compose serves it on :3001 (host). */
export const PERLENGKAPAN_URL =
  process.env.PERLENGKAPAN_URL || process.env.BASE_URL || 'http://localhost:3001';

/** localStorage key Perlengkapan reads the JWT from (features/auth.rs AUTH_TOKEN_KEY). */
export const AUTH_TOKEN_KEY = 'auth_token';
export const REFRESH_TOKEN_KEY = 'refresh_token';

export interface SeedCredentials {
  username: string;
  password: string;
}

/** The single seed user (has all four perlengkapan roles incl. admin). */
export const SEED_USER: SeedCredentials = {
  username: '199203142014031001',
  password: '199203142014031001',
};

export interface LoginTokens {
  accessToken: string;
  refreshToken: string;
}

/**
 * Log in via the real authenc REST API and return the JWT pair.
 * Throws if authenc is unreachable or the response lacks an access token.
 */
export async function apiLogin(
  request: APIRequestContext,
  creds: SeedCredentials = SEED_USER,
): Promise<LoginTokens> {
  const resp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
    data: { username: creds.username, password: creds.password },
    headers: { 'Content-Type': 'application/json' },
  });
  if (!resp.ok()) {
    throw new Error(
      `authenc login failed (${resp.status()}): ${await resp.text()} — is the e2e stack up (docker-compose.e2e.yml)?`,
    );
  }
  const body = await resp.json();
  if (!body.access_token) {
    throw new Error(
      `authenc login returned no access_token (mfa_required=${body.mfa_required}, ` +
        `require_password_change=${body.require_password_change}): ${JSON.stringify(body)}`,
    );
  }
  return { accessToken: body.access_token, refreshToken: body.refresh_token ?? '' };
}

/**
 * Seed a BrowserContext with a real authenticated session: log in via API, then
 * write the JWT into Perlengkapan's localStorage on its own origin (storageState
 * captures localStorage, so derived projects start authenticated).
 */
export async function seedRealAuth(
  context: BrowserContext,
  creds: SeedCredentials = SEED_USER,
): Promise<LoginTokens> {
  const tokens = await apiLogin(context.request, creds);
  const page = await context.newPage();
  // Must be on the target origin before touching its localStorage.
  await page.goto(PERLENGKAPAN_URL, { waitUntil: 'domcontentloaded' });
  await page.evaluate(
    ({ accessKey, refreshKey, access, refresh }) => {
      localStorage.setItem(accessKey, access);
      if (refresh) localStorage.setItem(refreshKey, refresh);
    },
    {
      accessKey: AUTH_TOKEN_KEY,
      refreshKey: REFRESH_TOKEN_KEY,
      access: tokens.accessToken,
      refresh: tokens.refreshToken,
    },
  );
  await page.close();
  return tokens;
}
