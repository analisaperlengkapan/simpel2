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
import type { APIRequestContext, BrowserContext } from "@playwright/test";

/** authenc REST base — the e2e overlay publishes it on host :18088. */
export const AUTHENC_URL = process.env.AUTHENC_URL || "http://localhost:18088";

/** Perlengkapan FE base — compose serves it on :3001 (host). */
export const PERLENGKAPAN_URL = process.env.PERLENGKAPAN_URL || process.env.BASE_URL || "http://localhost:3001";

/**
 * Perlengkapan BACKEND (layanan-perlengkapan) base — the axum API. RBAC
 * data-scoping is enforced server-side (`bank_aset::AsetScope::from_claims`), so
 * the scoping suite asserts against the API directly rather than the WASM UI
 * ("Bukti penegakan di BE — bukan sekadar tombol disembunyikan di FE", F5-C).
 * Compose publishes it on host :3020; on the CI compose network it is reachable
 * by service name (`http://layanan-perlengkapan:3020`).
 */
export const PERLENGKAPAN_API_URL = process.env.PERLENGKAPAN_API_URL || "http://localhost:3020";

/** localStorage key Perlengkapan reads the JWT from (features/auth.rs AUTH_TOKEN_KEY). */
export const AUTH_TOKEN_KEY = "auth_token";
export const REFRESH_TOKEN_KEY = "refresh_token";

export interface SeedCredentials {
  username: string;
  password: string;
}

/** The single seed user (has all four perlengkapan roles incl. admin). */
export const SEED_USER: SeedCredentials = {
  username: "199203142014031001",
  password: "199203142014031001",
};

export interface LoginTokens {
  accessToken: string;
  refreshToken: string;
}

/**
 * Log in via the real authenc REST API and return the JWT pair.
 * Throws if authenc is unreachable or the response lacks an access token.
 */
export async function apiLogin(request: APIRequestContext, creds: SeedCredentials = SEED_USER): Promise<LoginTokens> {
  const resp = await request.post(`${AUTHENC_URL}/api/v1/auth/login`, {
    data: { username: creds.username, password: creds.password },
    headers: { "Content-Type": "application/json" },
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
  return { accessToken: body.access_token, refreshToken: body.refresh_token ?? "" };
}

/**
 * Seed a BrowserContext with a real authenticated session: log in via API, then
 * write the JWT into Perlengkapan's localStorage on its own origin (storageState
 * captures localStorage, so derived projects start authenticated).
 */
export async function seedRealAuth(context: BrowserContext, creds: SeedCredentials = SEED_USER): Promise<LoginTokens> {
  const tokens = await apiLogin(context.request, creds);
  const page = await context.newPage();
  // Must be on the target origin before touching its localStorage.
  await page.goto(PERLENGKAPAN_URL, { waitUntil: "domcontentloaded" });
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

// ───────────────────────────────────────────────────────────────────────────
// Per-role RBAC data-scoping fixtures (#33 / F5-C).
//
// These mirror EXACTLY the synthetic users + assets in
// `tests/fixtures/e2e/seed-multisatker.sql` (loaded by the `e2e-seed` service
// onto a FRESH dbsimpelv2). Because the CI e2e stack starts empty and only this
// seed populates `integrasi.siman_aset`, the expected counts below are exact.
// (On staging — real snapshot, many more rows — only the per-row *isolation*
// invariant holds, not these absolute counts.)
//
// Seed asset distribution (5 total):
//   satker 0200010 (Jakpus, DKI, kdsatker_keu 006019999010001KD) → 2
//   satker 0200020 (Jaksel, DKI, kdsatker_keu 006019999020001KD) → 2
//   satker 0300010 (Bandung, JABAR, kdsatker_keu 006018888010001KD) → 1
// Wilayah tier groups by substring(kdsatker_keu, 6, 4): DKI = '9999', JABAR = '8888'.
// ───────────────────────────────────────────────────────────────────────────

/** A seeded single-role user plus its seed-derived expected `bank_aset` scope. */
export interface ScopedTestUser {
  /** Short stable id — used in storageState filenames + test titles. */
  key: string;
  /** Login username (NIP-shaped); password equals the username (seed convention). */
  username: string;
  /** The role carried in the issued JWT (`Claims.role`). */
  role: string;
  /** Caller MySIMKARI `kode_satker` (`Claims.satker_code`). */
  satkerCode: string;
  /** Exact number of `integrasi.siman_aset` rows visible to this user on the seed. */
  expectedAsetCount: number;
  /** SIMAN `kdsatker_keu` codes this user is allowed to see (per-row isolation). */
  allowedKdsatkerKeu: string[];
}

const KD_JAKPUS = "006019999010001KD";
const KD_JAKSEL = "006019999020001KD";
const KD_BANDUNG = "006018888010001KD";

/** All four per-role scoping users from the multi-satker seed. */
export const TEST_USERS: ScopedTestUser[] = [
  {
    key: "operator_a",
    username: "200000000000000001",
    role: "operator_satker",
    satkerCode: "0200010",
    expectedAsetCount: 2,
    allowedKdsatkerKeu: [KD_JAKPUS],
  },
  {
    key: "operator_b",
    username: "200000000000000002",
    role: "operator_satker",
    satkerCode: "0200020",
    expectedAsetCount: 2,
    allowedKdsatkerKeu: [KD_JAKSEL],
  },
  {
    key: "validator_wilayah",
    username: "200000000000000003",
    role: "validator_wilayah",
    satkerCode: "0200010",
    expectedAsetCount: 4,
    allowedKdsatkerKeu: [KD_JAKPUS, KD_JAKSEL],
  },
  {
    key: "validator_pusat",
    username: "200000000000000004",
    role: "validator_pusat",
    satkerCode: "0100000",
    expectedAsetCount: 5,
    allowedKdsatkerKeu: [KD_JAKPUS, KD_JAKSEL, KD_BANDUNG],
  },
  // Satker-internal approval chain (#96). Both sit in 0200010 with operator_a:
  // the Pemakaian chain is satker-internal, so validator and approver are the
  // operator's own colleagues and see exactly the same 2 assets.
  {
    key: "validator_satker",
    username: "200000000000000006",
    role: "validator_satker",
    satkerCode: "0200010",
    expectedAsetCount: 2,
    allowedKdsatkerKeu: [KD_JAKPUS],
  },
  {
    key: "approver_satker",
    username: "200000000000000007",
    role: "approver_satker",
    satkerCode: "0200010",
    expectedAsetCount: 2,
    allowedKdsatkerKeu: [KD_JAKPUS],
  },
];

/**
 * Credentials for a scoped test user. ALL multi-satker users share the base
 * seed password — the fixture reuses the one known Argon2id hash (verifying
 * exactly "199203142014031001") for every user, so `password == username`
 * 401s for everyone except the base seed user (proven on the first real run
 * of this suite, CI run 28557595484).
 */
export function credsFor(user: ScopedTestUser): SeedCredentials {
  return { username: user.username, password: SEED_USER.password };
}

/**
 * storageState file for a given role key (written by `auth.setup.ts`, reused by
 * authenticated specs via `test.use({ storageState })`). The all-role seed user
 * (admin-capable) lives under the key `admin`; per-role users under their `key`.
 */
export function storageStatePath(key: string): string {
  return `results/.auth/${key}.json`;
}

/** Shape of `PaginatedResponse<T>` returned by `lib_perlengkapan::response`. */
export interface PaginatedResponse<T> {
  success: boolean;
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
  message: string;
}

/** The subset of `BankAsetItem` fields the scoping suite asserts on. */
export interface BankAsetItem {
  id: string;
  nup: string;
  /** API name for the FE `satker` display field (DB `nama_satker`). */
  satker: string | null;
  /** The SIMAN `kdsatker_keu` value — the API serializes it as `kode_satker`. */
  kode_satker: string | null;
}

/**
 * Call `GET /api/v1/perlengkapan/bank-aset` on the BACKEND with a Bearer token.
 * Returns the raw Playwright response so callers can assert status codes too;
 * use {@link bankAsetListJson} when you only want the parsed body.
 */
export async function bankAsetList(
  request: APIRequestContext,
  token: string | null,
  query: Record<string, string | number> = {},
) {
  const params = new URLSearchParams(Object.entries({ per_page: 200, ...query }).map(([k, v]) => [k, String(v)]));
  const headers: Record<string, string> = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  return request.get(`${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/bank-aset?${params.toString()}`, { headers });
}

/** As {@link bankAsetList} but parses and returns the `PaginatedResponse` body. */
export async function bankAsetListJson(
  request: APIRequestContext,
  token: string,
  query: Record<string, string | number> = {},
): Promise<PaginatedResponse<BankAsetItem>> {
  const resp = await bankAsetList(request, token, query);
  if (!resp.ok()) {
    throw new Error(`bank-aset list failed (${resp.status()}): ${await resp.text()}`);
  }
  return resp.json();
}

/** Call `GET /api/v1/perlengkapan/bank-aset/{id}` with a Bearer token. */
export async function bankAsetDetail(request: APIRequestContext, token: string, id: string) {
  return request.get(`${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/bank-aset/${id}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
}

/**
 * Call `GET /api/v1/perlengkapan/admin/master` with an optional Bearer token.
 * The backend `/admin/*` endpoints enforce `require_admin` = `is_cross_satker_role`
 * server-side: cross-satker roles (admin/pusat/validator_pusat) → 200; satker-bound
 * roles → 403; no token → 401. NOTE this differs from the FE `/admin/*` UI guard,
 * which uses the narrower `is_admin()` (so validator_pusat is denied the UI but
 * allowed the API).
 */
export async function adminMasterList(request: APIRequestContext, token: string | null) {
  const headers: Record<string, string> = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  return request.get(`${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/admin/master`, {
    headers,
  });
}

/** Decoded JWT identity claims relevant to RBAC scoping. */
export interface JwtIdentity {
  /** `custom.satker_code` (or legacy `satker`) — the caller MySIMKARI code. */
  satkerCode?: string;
  /** `custom.realm_access.roles` — the realm roles authenc minted into the token. */
  realmRoles: string[];
}

/**
 * Decode (NOT verify) a JWT payload and pull out the scoping-relevant claims, so
 * the suite can assert authenc minted a correct per-role token independently of
 * whether the backend then enforces it. Signature verification is the backend's
 * job (authenc gRPC `ValidateToken`); here we only inspect the claims.
 */
export function decodeJwtIdentity(token: string): JwtIdentity {
  const part = token.split(".")[1];
  if (!part) throw new Error("not a JWT (missing payload segment)");
  const json = Buffer.from(part.replace(/-/g, "+").replace(/_/g, "/"), "base64").toString("utf8");
  const claims = JSON.parse(json) as Record<string, unknown>;
  const satkerCode = (claims.satker_code as string | undefined) ?? (claims.satker as string | undefined);
  const realmAccess = claims.realm_access as { roles?: unknown } | undefined;
  const realmRoles = Array.isArray(realmAccess?.roles)
    ? (realmAccess!.roles as unknown[]).filter((r): r is string => typeof r === "string")
    : [];
  return { satkerCode, realmRoles };
}
