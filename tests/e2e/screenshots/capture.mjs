#!/usr/bin/env node
// @ts-check
/**
 * Documentation screenshot sweep — every navigable route of both frontends,
 * under the topology production actually uses.
 *
 * Run:  node tests/e2e/screenshots/capture.mjs
 * Env:  APP_ORIGIN   single-origin ingress (default http://127.0.0.1:18099)
 *       OUT_DIR      screenshot destination (default docs/assets/screenshots)
 *
 * ── Why a single origin ─────────────────────────────────────────────────────
 * In production Istio serves `/portal`, `/perlengkapan` and `/api/*` from ONE
 * host. Both things this sweep is meant to falsify depend on that: SSO (portal
 * writes `localStorage["auth_token"]`, perlengkapan reads the same-origin
 * token) and every backend call (the WASM uses origin-relative URLs). Pointing
 * the browser at the FE's own compose port would detach both and produce a
 * gallery of empty shells that reads as product breakage.
 *
 * ── Why the routes are DERIVED ──────────────────────────────────────────────
 * `tests/e2e/screenshot-routes.py` walks the ROUTER (both `lib.rs`/`app.rs`),
 * not the curated `routes.rs` consts, because the consts omit the
 * parameterised detail routes — a const-driven sweep photographs the list page
 * and never the detail page, then reports "all pages covered". A route added
 * tomorrow is swept tomorrow.
 *
 * ── Why detail ids are resolved, not invented ───────────────────────────────
 * A screenshot of `/pengelolaan/pemakaian/detail/:id` with a literal `:id` is a
 * screenshot of the 404 page. Each parameter is filled from the backend's own
 * data for the logged-in token, and a parameter that cannot be resolved FAILS
 * the run — an unresolved id silently degrades a page shot into a 404 shot.
 */

import { chromium } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, '..', '..', '..');

const APP_ORIGIN = process.env.APP_ORIGIN || 'http://127.0.0.1:18099';
const AUTHENC_API = process.env.AUTHENC_API || `${APP_ORIGIN}/api/v1/auth`;
const API = `${APP_ORIGIN}/api/v1/perlengkapan`;
const IAM_API = `${APP_ORIGIN}/api/v1/iam`;
const OUT = process.env.OUT_DIR || join(REPO, 'docs', 'assets', 'screenshots');

/** Admin user (all roles; `require_password_change=false`). See seed-multisatker.sql. */
const ADMIN = { username: '200000000000000005', password: '199203142014031001' };
/** Satker-scoped operator — used for the negative (guard) shots. */
const OPERATOR = { username: '200000000000000001', password: '199203142014031001' };

/** Viewport wide enough for the two-column admin layouts. */
const VIEWPORT = { width: 1600, height: 1000 };

function routes() {
  const raw = execFileSync('python3', [join(REPO, 'tests', 'e2e', 'screenshot-routes.py')], {
    encoding: 'utf8',
    maxBuffer: 8 * 1024 * 1024,
  });
  return JSON.parse(raw);
}

/**
 * Plain API login. Captcha is only demanded after repeated failures
 * (`captcha_threshold`), so a fresh correct login needs none — which is why
 * this works without the `captcha-debug` build feature the form-driven specs
 * require.
 */
async function login(request, user) {
  const resp = await request.post(`${AUTHENC_API}/login`, {
    data: { username: user.username, password: user.password },
    headers: { 'Content-Type': 'application/json' },
  });
  if (!resp.ok()) {
    throw new Error(
      `login failed (${resp.status()}) for ${user.username}: ${await resp.text()} — ` +
        'is the stack up? (docker compose -f docker-compose.yml -f docker-compose.e2e.yml ' +
        '-f docker-compose.screenshots.yml up -d)',
    );
  }
  const body = await resp.json();
  if (!body.access_token) {
    throw new Error(`login returned no access_token: ${JSON.stringify(body)}`);
  }
  return body;
}

/** GET a perlengkapan API path with a bearer token. */
async function apiGet(request, token, path, query = {}) {
  const qs = new URLSearchParams(Object.entries(query).map(([k, v]) => [k, String(v)]));
  const url = `${API}${path}${qs.toString() ? `?${qs}` : ''}`;
  const resp = await request.get(url, { headers: { Authorization: `Bearer ${token}` } });
  if (!resp.ok()) throw new Error(`GET ${path} -> ${resp.status()}: ${await resp.text()}`);
  return resp.json();
}

/** GET an authenc IAM path with a bearer token. */
async function iamGet(request, token, path, query = {}) {
  const qs = new URLSearchParams(Object.entries(query).map(([k, v]) => [k, String(v)]));
  const url = `${IAM_API}${path}${qs.toString() ? `?${qs}` : ''}`;
  const resp = await request.get(url, { headers: { Authorization: `Bearer ${token}` } });
  if (!resp.ok()) throw new Error(`GET iam/${path} -> ${resp.status()}: ${await resp.text()}`);
  return resp.json();
}

/**
 * Resolution of the parameterised routes.
 *
 * Every id comes from the backend's own data for the admin token. A parameter
 * that cannot be resolved is recorded and FAILS the run: capturing
 * `/…/detail/:id` verbatim yields the 404 page, and filing that as a page
 * screenshot is exactly the silent degradation this sweep exists to prevent.
 *
 * Known seed UUIDs (from `tests/fixtures/e2e/seed-perlengkapan-workflow.sql`)
 * are the fallback for rows the API does not expose as a flat list — they are
 * documented fixtures, not invented values.
 */
async function resolveIds(request, token) {
  const unresolved = [];
  const ids = {};

  /** try a resolver, recording failure against the routes that need it. */
  const resolve = async (key, neededBy, fn) => {
    try {
      const v = await fn();
      if (!v) throw new Error('resolver returned nothing');
      ids[key] = v;
    } catch (e) {
      ids[key] = undefined;
      unresolved.push(`${neededBy} — ${e.message}`);
    }
  };

  const firstId = (body) => {
    const rows = body?.data ?? body;
    if (!Array.isArray(rows) || !rows.length) throw new Error('no rows');
    return rows[0].id;
  };

  await resolve('bankAsetId', '/bank-aset/daftar/:id', async () =>
    firstId(await apiGet(request, token, '/bank-aset', { per_page: 1 })),
  );

  await resolve('pemakaianId', '/pengelolaan/pemakaian/detail/:id', async () =>
    firstId(await apiGet(request, token, '/pemakaian-bmn', { per_page: 1 })),
  );

  await resolve('penghapusanId', '/pengelolaan/penghapusan/detail/:id', async () =>
    firstId(await apiGet(request, token, '/penghapusan-bmn', { per_page: 1 })),
  );

  await resolve('kebutuhanId', '/kebutuhan-bmn/:id/edit + /detail/:id', async () =>
    firstId(await apiGet(request, token, '/kebutuhan-bmn/pengajuan', { per_page: 1 })),
  );

  // Per-satker kebutuhan row. The seed documents these UUIDs so a spec can
  // deep-link without a list click; ask the API first, fall back to the seed.
  await resolve('kebutuhanSatkerId', '/kebutuhan-bmn/satker/:satker_id', async () => {
    const body = await apiGet(request, token, '/kebutuhan-bmn/pengajuan/c1000000-0000-4c00-8c00-000000000001/satker');
    const rows = body?.data ?? body;
    if (Array.isArray(rows) && rows.length) return rows[0].id;
    return 'c1000000-0000-4c00-8c00-0000000a0001';
  });

  await resolve('pakaianJenisId', '/pakaian-dinas/jenis/:id/spesifikasi', async () =>
    firstId(await apiGet(request, token, '/pakaian-dinas/jenis', { per_page: 1 })),
  );

  await resolve('pakaianPengajuanId', '/pakaian-dinas/pengajuan/:pengajuan_id/satker', async () =>
    firstId(await apiGet(request, token, '/pakaian-dinas/pengajuan', { per_page: 1 })),
  );

  await resolve(
    'pakaianSatkerCode',
    '/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi',
    async () => {
      const body = await apiGet(
        request,
        token,
        `/pakaian-dinas/pengajuan/${ids.pakaianPengajuanId}/satker`,
      );
      const rows = body?.data ?? body;
      if (Array.isArray(rows) && rows.length) {
        return rows[0].satker_id ?? rows[0].satker_code ?? rows[0].kode_satker;
      }
      return undefined;
    },
  );

  await resolve('portalUserId', '/portal/admin/users/:id', async () =>
    firstId(await iamGet(request, token, '/users', { per_page: 1 })),
  );

  await resolve('portalClientId', '/portal/admin/clients/:id', async () =>
    firstId(await iamGet(request, token, '/clients', { per_page: 1 })),
  );

  return { ids, unresolved };
}

/** Which resolver fills the `:param` in a given route. */
const PARAM_RESOLVERS = [
  ['/bank-aset/daftar/:id', 'bankAsetId'],
  ['/kebutuhan-bmn/:id/edit', 'kebutuhanId'],
  ['/kebutuhan-bmn/detail/:id', 'kebutuhanId'],
  ['/pakaian-dinas/jenis/:id/spesifikasi', 'pakaianJenisId'],
  ['/pengelolaan/pemakaian/detail/:id', 'pemakaianId'],
  ['/pengelolaan/penghapusan/detail/:id', 'penghapusanId'],
  ['/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi', 'pakaianIsiComposite'],
  ['/pakaian-dinas/pengajuan/:pengajuan_id/satker', 'pakaianPengajuanId'],
  ['/kebutuhan-bmn/satker/:satker_id', 'kebutuhanSatkerId'],
  ['/admin/users/:id', 'portalUserId'],
  ['/admin/clients/:id', 'portalClientId'],
];

function fillParams(route, ids, unresolved) {
  if (!route.includes(':')) return route;
  if (route.includes('/pakaian-dinas/pengajuan/:pengajuan_id/satker/:satker_code/isi')) {
    if (!ids.pakaianPengajuanId || !ids.pakaianSatkerCode) {
      if (!unresolved.some((u) => u.startsWith(route))) unresolved.push(route);
      return null;
    }
    return route
      .replace(':pengajuan_id', ids.pakaianPengajuanId)
      .replace(':satker_code', ids.pakaianSatkerCode);
  }
  for (const [pattern, key] of PARAM_RESOLVERS) {
    if (route.includes(pattern)) {
      if (!ids[key]) {
        if (!unresolved.some((u) => u.startsWith(pattern))) unresolved.push(pattern);
        return null;
      }
      return route.replace(/:\w+/g, ids[key]);
    }
  }
  if (!unresolved.includes(route)) unresolved.push(route);
  return null;
}

const slug = (p) => p.replace(/^\/+/, '').replace(/[/]+/g, '__').replace(/[^a-zA-Z0-9_-]/g, '');

/**
 * Build the `user_session` blob exactly as each app builds it on boot.
 *
 * Writing only `auth_token` is NOT enough, and the reason is a real divergence
 * between the two frontends — not a detail of this script:
 *
 *   • perlengkapan (`features/auth.rs::load_session`) reads `auth_token` and
 *     derives the session from the JWT.
 *   • portal (`AuthService::load_session`) reads ONLY the `user_session` blob
 *     and has no token fallback — `app.rs` initialises the guard signal with
 *     `signal(AuthService::load_session())`.
 *
 * So injecting just the token left all 22 authenticated portal routes showing
 * the login page: the guard saw `None`, rendered `<LoginPage>` inline, and the
 * sweep photographed the login form 22 times while reporting full coverage.
 *
 * The mapping below mirrors `AuthService::decode_jwt_claims` field for field
 * (including `UserRole`'s serde representation and `get_primary_role`'s
 * precedence), so the injected session is the one the app would have built —
 * not a hand-rolled approximation that drifts as the struct grows.
 */
function buildUserSession(tokens, claims) {
  const roles = claims.realm_access?.roles ?? [];
  const role = roles.includes('admin')
    ? 'Admin'
    : roles.includes('supervisor')
      ? 'Supervisor'
      : roles.includes('guest')
        ? 'Guest'
        : 'User';
  const username = claims.preferred_username ?? claims.sub;
  const fallbackName = username.charAt(0).toUpperCase() + username.slice(1);
  const satkerCode = claims.satker_code ?? null;
  return {
    id: claims.sub,
    username,
    role,
    name: claims.name ?? fallbackName,
    email: claims.email ?? `${username}@kejaksaan.go.id`,
    avatar: null,
    nip: claims.nip ?? null,
    jabatan: claims.jabatan ?? null,
    satker_code: satkerCode,
    satuan_kerja: satkerCode ?? '',
    captcha_validated: true,
    mfa_enabled: claims.mfa_enabled ?? false,
    mfa_setup_required: claims.mfa_setup_required ?? false,
    require_password_change: claims.require_password_change ?? false,
    created_at: new Date().toISOString(),
    access_token: tokens.access_token,
    refresh_token: tokens.refresh_token ?? null,
    expires_at: claims.exp ?? null,
    permissions: roles,
  };
}

/** Decode a JWT payload without verifying (the backend already did). */
function decodeClaims(token) {
  return JSON.parse(Buffer.from(token.split('.')[1], 'base64url').toString('utf8'));
}

/**
 * Seed both storage keys BEFORE any app code runs.
 *
 * `addInitScript` runs on every navigation, ahead of the bundle, so the guard's
 * initial `load_session()` already sees a session. Setting the keys after
 * `goto` would race the WASM boot and only work by luck of a reload.
 */
async function seedSession(context, tokens) {
  const session = JSON.stringify(buildUserSession(tokens, decodeClaims(tokens.access_token)));
  await context.addInitScript(
    ([access, refresh, sess]) => {
      localStorage.setItem('auth_token', access);
      localStorage.setItem('refresh_token', refresh);
      localStorage.setItem('user_session', sess);
    },
    [tokens.access_token, tokens.refresh_token ?? '', session],
  );
}

/**
 * Capture one route and collect the signals a human reviewer needs before
 * trusting the image. The flag vocabulary is the one the perlengkapan visual
 * sweep already paid for (`_sweep.js`): each entry corresponds to a defect that
 * actually shipped, so none of these is speculative.
 *
 * `opts.expectDenied` marks a shot that is SUPPOSED to be refused: the guard
 * pages exist to show the RBAC gate in the app's own words, so a 403 body is
 * the assertion passing, not a defect. Those signals move to `expected` — and
 * the absence of the denial is itself promoted to a flag, since a guard page
 * that renders normally means the gate did not hold.
 */
async function shoot(page, url, file, findings, opts = {}) {
  const expectDenied = opts.expectDenied === true;
  let consoleErrors = [];
  let pageErrors = [];
  const onConsole = (m) => {
    if (m.type() === 'error') consoleErrors.push(m.text().slice(0, 200));
  };
  const onPageError = (e) => pageErrors.push(String(e).slice(0, 300));
  page.on('console', onConsole);
  page.on('pageerror', onPageError);

  /** Resolves true once the boot overlay is gone (i.e. WASM mounted). */
  const mounted = () =>
    page
      .waitForFunction(
        () => {
          const el = document.getElementById('loading-screen');
          if (!el) return true;
          const s = getComputedStyle(el);
          return s.display === 'none' || s.opacity === '0' || el.style.display === 'none';
        },
        { timeout: 15000 },
      )
      .then(() => true)
      .catch(() => false);

  try {
    // Two attempts. The WASM bundle is fetched as a multi-megabyte stream, and a
    // single dropped/aborted chunk surfaces as `WebAssembly compilation aborted:
    // Network error` with the boot overlay still covering the page — a
    // transient transport failure that a reload clears, not a product defect.
    // Without this the sweep files a screenshot of the splash screen as page
    // coverage and the report reads as "the page panics". A genuine boot bug
    // fails BOTH attempts and is still reported.
    let ok = false;
    for (let attempt = 0; attempt < 2 && !ok; attempt += 1) {
      consoleErrors = [];
      pageErrors = [];
      await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
      // Wait for the page's own traffic to settle rather than a guessed constant:
      // the dashboard genuinely takes seconds over a real asset table, and a
      // screenshot of a loading skeleton proves nothing about the page.
      await page.waitForLoadState('networkidle', { timeout: 30000 }).catch(() => {});
      await page.waitForTimeout(900);
      ok = await mounted();
    }

    await page.screenshot({ path: file, fullPage: true });

    // Layout sanity, measured from the live DOM rather than judged by eye on
    // 70 images. A horizontal scrollbar on a desktop viewport means something
    // is wider than the page — a table that did not shrink, an absolutely
    // positioned panel, a long unbroken token. The screenshot still "has
    // content", so `check.py` cannot see it, and a reviewer scrolling a gallery
    // of 70 will not notice the one image that is 40px too wide.
    const layout = await page
      .evaluate(() => {
        const de = document.documentElement;
        const overflowing = [];
        for (const el of document.querySelectorAll('body *')) {
          const r = el.getBoundingClientRect();
          // Ignore zero-size and off-screen-by-design nodes (menus, tooltips
          // parked at -9999px, the skip-link pattern).
          if (r.width === 0 || r.height === 0) continue;
          if (r.left < -4) continue;
          if (r.right > window.innerWidth + 4) {
            overflowing.push(
              `${el.tagName.toLowerCase()}${el.className && typeof el.className === 'string' ? '.' + el.className.trim().split(/\s+/).slice(0, 2).join('.') : ''}@${Math.round(r.right)}`,
            );
          }
        }
        return {
          scrollWidth: de.scrollWidth,
          innerWidth: window.innerWidth,
          overflowing: overflowing.slice(0, 4),
        };
      })
      .catch(() => ({ scrollWidth: 0, innerWidth: 0, overflowing: [] }));
    if (layout.scrollWidth > layout.innerWidth + 2) {
      flags.push(`meluber-horizontal:${layout.scrollWidth}>${layout.innerWidth}`);
      if (layout.overflowing.length) flags.push(`elemen-luar:${layout.overflowing.join(',')}`);
    }

    // Read the CONTENT, not the shell: the sidebar/footer are identical on
    // every page, so a detector firing on shell text fires everywhere and is as
    // useless as one that fires nowhere.
    const main = page.locator('main, [role="main"]').first();
    const scope = (await main.count()) ? main : page.locator('body');
    let text = '';
    try {
      text = await scope.innerText({ timeout: 5000 });
    } catch {
      text = '';
    }
    // `innerText` applies `text-transform: uppercase`; `textContent` gives the
    // author's own casing, which is what "is this a raw enum?" asks about.
    //
    // Text nodes are collected INDIVIDUALLY and joined with a separator rather
    // than reading one flat `textContent`. Adjacent elements concatenate with no
    // boundary, so two cells reading "BMN" and "BMN" become the single token
    // "BMNBMN" — which the shouty-word rule below then reports as a defect. That
    // is a property of how the DOM was serialised, not of what the operator
    // sees, and it produced false positives on every table page (a JOIN over
    // text nodes restores the boundary the markup already had).
    let source = '';
    try {
      source = await scope.evaluate((el) => {
        const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
        const parts = [];
        let node = walker.nextNode();
        while (node) {
          const t = (node.textContent || '').trim();
          if (t) parts.push(t);
          node = walker.nextNode();
        }
        return parts.join('\n');
      });
    } catch {
      source = '';
    }

    const flags = [];
    if (/Halaman Tidak Ditemukan|404 Not Found/i.test(text)) flags.push('RUTE-404');
    if (/Akses Ditolak/.test(text)) flags.push('AKSES-DITOLAK');
    // An AUTHENTICATED route that renders the login page means the request was
    // rejected (or the seeded session was not picked up) — the page is real,
    // tidy, and completely wrong, so nothing else here would catch it. This is
    // exactly how the first sweep filed 22 shots of the login form as portal
    // page coverage.
    const isLoginRoute = /\/login\b/.test(new URL(url).pathname);
    if (!isLoginRoute && /Masuk ke Sistem|Masuk ke Portal|Masuk via Portal|Silakan login/i.test(text)) {
      flags.push('LOGIN-DITAMPILKAN');
    }
    // Leptos source leaking into the page: a split `>=` ends the attribute
    // early and the rest of the markup becomes a text node.
    if (/on:click|move \|_\||prop:disabled|\.get\(\)/.test(text)) flags.push('kode-bocor');
    if (/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(\+00:00|Z)/.test(text)) flags.push('waktu-mentah-utc');
    // A UUID on screen is a defect only when it is LEFT UNLABELLED — that is,
    // when it is standing in for something the operator should have been shown
    // by name (the audit table rendered the actor as raw `user_id`). An admin
    // detail page that prints "ID Internal" above its own identifier is not a
    // leak; the value IS the subject of that field. Judging by proximity to a
    // declaring label separates the two, where a bare "any UUID anywhere" rule
    // cannot and so fires on the pages that render it correctly.
    {
      const LABELS = /(ID Internal|Internal ID|ID Pengguna|ID Klien|ID Sesi)/;
      const uuidRe = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g;
      let unlabelled = false;
      for (const m of text.matchAll(uuidRe)) {
        const before = text.slice(Math.max(0, m.index - 48), m.index);
        if (!LABELS.test(before)) {
          unlabelled = true;
          break;
        }
      }
      if (unlabelled) flags.push('uuid-di-layar');
    }
    if (/\bNaN\b|\bundefined\b|\bUnknown\b/.test(text)) flags.push('nilai-kosong');
    if (/[A-Z]{3,}_[A-Z]{3,}/.test(text)) flags.push('enum-mentah');
    if (/Tahun Anggaran 20(1|2[0-5])\b/.test(text)) flags.push('tahun-basi');
    {
      // "Is a raw enum on screen?", asked precisely.
      //
      // The previous form counted ANY ALL-CAPS word and flagged the page once the
      // count exceeded three. In this app that rule cannot answer its own
      // question: the backends store statuses/priorities as lowercase Indonesian
      // (`disetujui`, `draft`) or as SCREAMING_SNAKE codes, so a leaking enum
      // surfaces as `lower_snake` or `SCREAMING_SNAKE` — never as a bare
      // capitalised word. Meanwhile the SoT data genuinely IS uppercase
      // (`KEJAKSAAN NEGERI JAKARTA`, `DKI`, `TINGGI`), and rewriting it for the
      // screen would misreport the source system. So the old rule fired on every
      // table page and stayed silent on the defect it was written for.
      //
      // What a leak actually looks like is a wire token that this codebase's own
      // vocabulary defines, appearing verbatim. Match THAT set instead.
      const WIRE_VALUES = new Set([
        'PENDING', 'APPROVED', 'REJECTED', 'DRAFT', 'ACTIVE', 'INACTIVE', 'EXPIRED',
        'REVOKED', 'CANCELLED', 'COMPLETED', 'FAILED', 'SUCCESS', 'ERROR', 'UNKNOWN',
        'ENABLED', 'DISABLED', 'LOCKED', 'SUSPENDED', 'DELETED', 'ARCHIVED',
        'SUBMITTED', 'ANALYSIS', 'REVISION', 'VALIDATED',
      ]);
      const leaked = (source.match(/\b[A-Z]{3,}\b/g) || []).filter((w) => WIRE_VALUES.has(w));
      if (leaked.length) flags.push(`enum-mentah:${[...new Set(leaked)].slice(0, 4).join(',')}`);
    }
    if (pageErrors.length) {
      // The WASM bundle arrives as one streamed response. If a chunk is aborted
      // the runtime reports `WebAssembly compilation aborted: Network error`,
      // and the retry loop above reloads. When that reload SUCCEEDED the page
      // is mounted and correct — the only surviving trace is the first
      // attempt's error, still sitting in the buffer. Reporting it would call a
      // working page broken because a download was interrupted once; the page's
      // own content contradicts it. So a transport abort is a defect only when
      // the app never mounted (`ok` false); any other page error is always one.
      const transportAbort = (e) =>
        /WebAssembly compilation aborted|Response body loading was aborted|Failed to fetch|NetworkError/i.test(e);
      const realErrors = ok ? pageErrors.filter((e) => !transportAbort(e)) : pageErrors;
      if (realErrors.length) flags.push(`panic:${realErrors.length}`);
    }
    if (consoleErrors.length) flags.push(`console-error:${consoleErrors.length}`);
    if (text.trim().length < 120) flags.push('halaman-nyaris-kosong');

    // Split expected-vs-defect. For a guard shot the denial IS the point, so
    // "the page was refused" and "the page is nearly empty (it is a 403 card)"
    // are assertions that held. Everything else — a panic, a raw enum, a blank
    // body — stays a defect even on a guard page, because those are never what
    // a denial looks like.
    const EXPECTED_ON_GUARD = new Set(['AKSES-DITOLAK', 'halaman-nyaris-kosong']);
    let expected = [];
    let defectFlags = flags;
    if (expectDenied) {
      expected = flags.filter((f) => EXPECTED_ON_GUARD.has(f));
      defectFlags = flags.filter(
        (f) => !EXPECTED_ON_GUARD.has(f) && f !== 'RUTE-404' && f !== 'LOGIN-DITAMPILKAN',
      );
      // The negative half of the assertion: a guarded route that renders its
      // real content means the gate let a non-admin through.
      if (!expected.includes('AKSES-DITOLAK')) defectFlags.push('GUARD-TIDAK-MENAHAN');
    }

    findings.push({
      file: file.split('/').pop(),
      url,
      flags: defectFlags,
      expected,
      consoleErrors,
      pageErrors,
      textPreview: text.slice(0, 500),
    });
  } catch (e) {
    findings.push({
      file: file.split('/').pop(),
      url,
      flags: [`GAGAL:${String(e).slice(0, 120)}`],
      consoleErrors,
      pageErrors,
      textPreview: '',
    });
  } finally {
    page.off('console', onConsole);
    page.off('pageerror', onPageError);
  }
}

async function main() {
  const inventory = routes();
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });

  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: VIEWPORT });
  const request = context.request;

  const adminTokens = await login(request, ADMIN);
  const { ids, unresolved } = await resolveIds(request, adminTokens.access_token);

  // Seed the session into the CONTEXT, before any navigation, so both the
  // `auth_token` key (perlengkapan) and the `user_session` blob (portal) exist
  // by the time the app's boot code reads them.
  await seedSession(context, adminTokens);

  const page = await context.newPage();
  await page.goto(`${APP_ORIGIN}/portal/`, { waitUntil: 'domcontentloaded' }).catch(() => {});

  const findings = [];
  const capturedRoutes = [];
  let captured = 0;

  for (const [fe, meta] of Object.entries(inventory)) {
    for (const route of meta.routes) {
      const path = fillParams(route, ids, unresolved);
      if (!path) continue;
      const file = join(OUT, `${fe}__${slug(route)}.png`);
      await shoot(page, `${APP_ORIGIN}${path}`, file, findings);
      capturedRoutes.push({ fe, route, path, file: file.split('/').pop() });
      captured += 1;
    }
  }

  // Public / unauthenticated views: these exist only WITHOUT a session — with a
  // token the router redirects to the dashboard and the shot documents nothing.
  const anon = await browser.newContext({ viewport: VIEWPORT });
  const anonPage = await anon.newPage();
  for (const [fe, path] of [
    ['portal', '/portal/login'],
    ['portal', '/portal/logged-out'],
    ['perlengkapan', '/perlengkapan/simpel/v2/login'],
  ]) {
    const file = join(OUT, `${fe}__anon__${slug(path)}.png`);
    await shoot(anonPage, `${APP_ORIGIN}${path}`, file, findings);
    capturedRoutes.push({ fe, route: `${path} (anon)`, path, file: file.split('/').pop() });
    captured += 1;
  }

  // Guard views: a non-admin on an admin route. Documents that the RBAC gate is
  // real, in the app's own words, instead of only asserting it in a test.
  const opTokens = await login(request, OPERATOR);
  const opCtx = await browser.newContext({ viewport: VIEWPORT });
  await seedSession(opCtx, opTokens);
  const opPage = await opCtx.newPage();
  await opPage
    .goto(`${APP_ORIGIN}/perlengkapan/simpel/v2/`, { waitUntil: 'domcontentloaded' })
    .catch(() => {});
  for (const path of ['/perlengkapan/simpel/v2/admin/roles', '/portal/admin/users']) {
    const file = join(OUT, `guard__operator__${slug(path)}.png`);
    await shoot(opPage, `${APP_ORIGIN}${path}`, file, findings, { expectDenied: true });
    capturedRoutes.push({ fe: path.startsWith('/portal') ? 'portal' : 'perlengkapan', route: `${path} (operator)`, path, file: file.split('/').pop() });
    captured += 1;
  }

  await browser.close();

  const flagged = findings.filter((f) => f.flags.length);
  writeFileSync(
    join(OUT, 'capture-report.json'),
    JSON.stringify({ app_origin: APP_ORIGIN, captured, unresolved, capturedRoutes, findings }, null, 2),
  );

  console.log(`\ncaptured ${captured} screenshots -> ${OUT}`);
  if (unresolved.length) {
    console.log(`\nUNRESOLVED ROUTES (${unresolved.length}) — NOT captured:`);
    unresolved.forEach((u) => console.log('  ' + u));
  }
  // Print only what actually carries a flag. Listing every capture under
  // "FLAGGED" (as this did) made the count meaningless — 51 clean shots were
  // reported as findings and the 13 real ones had to be dug out by hand.
  console.log(`\nFLAGGED (${flagged.length} of ${findings.length} captures):`);
  flagged.forEach((f) => console.log(`  ${f.file} -> ${f.flags.join(', ')}`));

  // A run that silently drops routes is worse than one that fails: it reports
  // coverage it does not have.
  process.exit(unresolved.length ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});