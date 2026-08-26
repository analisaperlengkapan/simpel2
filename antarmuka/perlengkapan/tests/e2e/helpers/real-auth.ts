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
 * Seconds of remaining lifetime below which a cached token is thrown away.
 * authenc issues 900s tokens; a test that starts with 60s left can outlive it
 * mid-request, so refresh well before the edge.
 */
const TOKEN_MIN_REMAINING_SECONDS = 120;

/** Per-worker token cache, keyed by username. Playwright workers are separate
 *  processes, so this is per-worker rather than global — which is fine: the
 *  point is to stop logging in once per TEST. */
const tokenCache = new Map<string, { tokens: LoginTokens; expEpoch: number }>();

/** `exp` out of a JWT payload, in epoch seconds; 0 if it cannot be read. */
function jwtExpEpoch(token: string): number {
  const part = token.split(".")[1];
  if (!part) return 0;
  try {
    const json = Buffer.from(part.replace(/-/g, "+").replace(/_/g, "/"), "base64").toString("utf8");
    const exp = JSON.parse(json).exp;
    return typeof exp === "number" ? exp : 0;
  } catch {
    return 0;
  }
}

/**
 * Log in via the real authenc REST API and return the JWT pair.
 * Throws if authenc is unreachable or the response lacks an access token.
 *
 * CACHED per worker, per user, until the token nears expiry. That is not a
 * micro-optimisation — it is what makes this suite runnable outside CI:
 *
 *   - `docker-compose.e2e.yml` sets `AUTHENC_RATE_LIMIT_ENABLED: "false"`.
 *     The Helm chart has no such knob, so against staging (and production)
 *     the limiter is ON. There are ~45 direct `apiLogin` call sites across
 *     seven specs; at four workers they tripped it, and the failure surfaced
 *     as `429 Rate limit exceeded` — with a message blaming a stack that was
 *     in fact perfectly healthy.
 *   - Running serially to dodge the limiter took 9.7 minutes, which is longer
 *     than authenc's 900-second token lifetime, so the tail of the run failed
 *     with `401 Token validation failed` instead. Both directions failed for
 *     harness reasons, and neither said so.
 *
 * Caching removes the rate-limit pressure, and the expiry check means a long
 * run refreshes instead of dying. See memory `project_staging_shape_blindness`.
 */
export async function apiLogin(request: APIRequestContext, creds: SeedCredentials = SEED_USER): Promise<LoginTokens> {
  const nowEpoch = Math.floor(Date.now() / 1000);
  const cached = tokenCache.get(creds.username);
  if (cached && cached.expEpoch - nowEpoch > TOKEN_MIN_REMAINING_SECONDS) {
    return cached.tokens;
  }

  const tokens = await apiLoginUncached(request, creds);
  const expEpoch = jwtExpEpoch(tokens.accessToken);
  // A token whose `exp` we cannot read is used once and never cached — better
  // an extra login than handing every later test a token of unknown lifetime.
  if (expEpoch > nowEpoch) {
    tokenCache.set(creds.username, { tokens, expEpoch });
  }
  return tokens;
}

/** The actual round trip. Use `apiLogin` unless a test is specifically
 *  exercising the login endpoint itself and must not be served from cache. */
export async function apiLoginUncached(
  request: APIRequestContext,
  creds: SeedCredentials = SEED_USER,
): Promise<LoginTokens> {
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
// These mirror the synthetic users in `tests/fixtures/e2e/seed-multisatker.sql`.
// Only facts that are TRUE OF THE USER live here — login, role, satker code and
// the tier the backend must scope that role to. What each user can *see* is
// deliberately NOT recorded: it is derived at runtime from the environment.
//
// The earlier form of this table hard-coded `expectedAsetCount` (2/2/4/5) and an
// `allowedKdsatkerKeu` allowlist of three synthetic codes. Both are properties of
// a FRESH, EMPTY CI database that only this seed populates — so the suite could
// only ever run against CI. Pointed at staging (real SIMAN snapshot: operator_a
// 1 681 rows, validator_pusat 624 533) every count assertion failed, and the
// isolation check failed *backwards*: it flagged `006010100005037000KD` — JakPus's
// own real assets, correctly scoped — as a leak, because the code was not in a
// list written before that data existed.
//
// That is the failure mode catalogued in `project_gate_scope_must_be_derived`:
// the gate was right, its scope was a hand-written list that did not grow with
// reality. So the expectations are now DERIVED from the environment under test
// (see `deriveScope`), which makes the same suite valid on CI and on staging —
// the environment the release plan requires it to certify.
// ───────────────────────────────────────────────────────────────────────────

/**
 * The tier the backend must scope a role to. This IS a property of the role
 * (`bank_aset::AsetScope::from_claims`), so it belongs in the fixture table;
 * the row counts each tier yields do not.
 */
export type ScopeTier = "satker" | "wilayah" | "nasional";

/** A seeded single-role user. Scope expectations are derived, not stored. */
export interface ScopedTestUser {
  /** Short stable id — used in storageState filenames + test titles. */
  key: string;
  /** Login username (NIP-shaped); password equals the username (seed convention). */
  username: string;
  /** The role carried in the issued JWT (`Claims.role`). */
  role: string;
  /** Caller MySIMKARI `kode_satker` (`Claims.satker_code`). */
  satkerCode: string;
  /** Tier the backend must scope this role to. */
  tier: ScopeTier;
  /**
   * Key of another user whose visible set this user's must EQUAL. Set for the
   * satker-internal approval chain (#96): validator/approver sit in the
   * operator's own satker, so identical scope is the invariant — in any
   * environment, whatever the row count happens to be.
   */
  sameScopeAs?: string;
}

/** All per-role scoping users from the multi-satker seed. */
export const TEST_USERS: ScopedTestUser[] = [
  {
    key: "operator_a",
    username: "200000000000000001",
    role: "operator_satker",
    satkerCode: "0200010",
    tier: "satker",
  },
  {
    key: "operator_b",
    username: "200000000000000002",
    role: "operator_satker",
    satkerCode: "0200020",
    tier: "satker",
  },
  {
    key: "validator_wilayah",
    username: "200000000000000003",
    role: "validator_wilayah",
    satkerCode: "0200010",
    tier: "wilayah",
  },
  {
    key: "validator_pusat",
    username: "200000000000000004",
    role: "validator_pusat",
    satkerCode: "0100000",
    tier: "nasional",
  },
  // Satker-internal approval chain (#96): both sit in 0200010 with operator_a.
  {
    key: "validator_satker",
    username: "200000000000000006",
    role: "validator_satker",
    satkerCode: "0200010",
    tier: "satker",
    sameScopeAs: "operator_a",
  },
  {
    key: "approver_satker",
    username: "200000000000000007",
    role: "approver_satker",
    satkerCode: "0200010",
    tier: "satker",
    sameScopeAs: "operator_a",
  },
];

/** Look up a seeded user by key, failing loudly rather than returning undefined. */
export function userByKey(key: string): ScopedTestUser {
  const user = TEST_USERS.find((u) => u.key === key);
  if (!user) {
    throw new Error(`unknown seeded test user: ${key} (known: ${TEST_USERS.map((u) => u.key).join(", ")})`);
  }
  return user;
}

/**
 * Access token for a seeded test user key, for direct API assertions.
 *
 * Handles the `admin` key, which the scoped `TEST_USERS` table deliberately
 * does NOT contain: the admin identity is the all-role [`SEED_USER`], not a
 * satker-scoped user. `storageStatePath("admin")` already resolves that way for
 * browser contexts (auth.setup writes the seed user under that key), so a
 * `tokenFor("admin")` that threw was an inconsistency between the browser path
 * and the API path — and it threw "unknown seeded test user: admin" the first
 * time admin-bantuan-workflow.spec.ts ever executed.
 *
 * Lives here rather than being copy-pasted per spec: two specs had their own
 * identical local copies, so the bug had to be found and fixed twice.
 */
export async function tokenFor(
  request: APIRequestContext,
  userKey: string,
): Promise<string> {
  if (userKey === "admin") {
    const { accessToken } = await apiLogin(request, SEED_USER);
    return accessToken;
  }
  const { accessToken } = await apiLogin(request, credsFor(userByKey(userKey)));
  return accessToken;
}

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

/**
 * What a token can actually SEE, measured rather than assumed.
 *
 * `total` is the backend's own count for the whole scope (not the page), so it
 * is exact regardless of how many rows a page returns. `satkerNames`/`kdCodes`
 * are distinct values over the sampled page — enough to prove a leak (one
 * foreign row is a counter-example) but NOT enough to prove containment, since
 * a page is a sample. Containment is therefore asserted via `scopedTotalFor`,
 * which compares exact totals instead of sampled sets.
 */
export interface DerivedScope {
  total: number;
  satkerNames: string[];
  kdCodes: string[];
}

/** Measure a token's visible scope. `perPage` bounds the sample, not the count. */
export async function deriveScope(
  request: APIRequestContext,
  token: string,
  query: Record<string, string | number> = {},
  perPage = 200,
): Promise<DerivedScope> {
  const body = await bankAsetListJson(request, token, { per_page: perPage, ...query });
  const names = new Set<string>();
  const kds = new Set<string>();
  for (const row of body.data) {
    if (row.satker) names.add(row.satker);
    if (row.kode_satker) kds.add(row.kode_satker);
  }
  return { total: body.total, satkerNames: [...names].sort(), kdCodes: [...kds].sort() };
}

/**
 * Exact number of rows a token sees when the list is filtered to one satker.
 *
 * The cross-check this enables is the point of the whole suite: a national user
 * filtering EXPLICITLY to satker X must get the same count a satker-bound user
 * of X gets IMPLICITLY. Those are two different code paths — a user-supplied
 * `satker` filter versus `AsetScope::from_claims` — so their agreeing is real
 * evidence, and it needs no knowledge of how many assets X happens to own.
 */
export async function scopedTotalFor(
  request: APIRequestContext,
  token: string,
  satkerName: string,
): Promise<number> {
  const body = await bankAsetListJson(request, token, { per_page: 1, satker: satkerName });
  return body.total;
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

/** One dropdown choice from `GET /bank-aset/filter-options` (value + row count). */
export interface FilterOption {
  value: string;
  count: number;
}

/** Scoped filter-dropdown values, as the FE loads them on mount. */
export interface BankAsetFilterOptions {
  jenis: FilterOption[];
  kategori: FilterOption[];
  kondisi: FilterOption[];
  satker: FilterOption[];
}

/**
 * Fetch the filter dropdown options the FE populates its selects from.
 *
 * These are RBAC-scoped server-side (`repository::filter_options` applies the
 * same `AsetScope::push_condition` predicate as the list query), so the satker
 * dropdown is an EXACT, page-size-independent statement of which satkers a user
 * may see — unlike the rendered table, which is only ever page 1 of N. That is
 * what makes it usable as a scope probe against a 624 533-row staging snapshot
 * where nothing seeded is guaranteed to appear on the first page.
 */
export async function filterOptions(
  request: APIRequestContext,
  token: string,
): Promise<BankAsetFilterOptions> {
  const resp = await request.get(`${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/bank-aset/filter-options`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!resp.ok()) {
    throw new Error(`bank-aset filter-options failed (${resp.status()}): ${await resp.text()}`);
  }
  const body = await resp.json();
  return body.data ?? body;
}

/** Distinct satker names a token may see, derived from its scoped dropdown. */
export async function allowedSatkerNames(request: APIRequestContext, token: string): Promise<string[]> {
  const opts = await filterOptions(request, token);
  return opts.satker.map((o) => o.value).sort();
}

/**
 * Render an integer the way the FE does (`format_thousands`): Indonesian
 * thousands separator `.`, so 1681 → "1.681". Lets a UI assertion compare
 * against a number measured from the API instead of a hard-coded literal.
 */
export function formatThousands(n: number): string {
  return Math.abs(n)
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, ".")
    .replace(/^/, n < 0 ? "-" : "");
}
