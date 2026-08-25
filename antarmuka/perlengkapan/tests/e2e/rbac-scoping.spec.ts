/**
 * RBAC data-scoping — server-side enforcement (#33 / F5-C, validates #66 / #565 /
 * #566 against the REAL stack).
 *
 * The plan's Layer-2 ("API integrasi"): RBAC must be ENFORCED server-side, not
 * merely hidden in the FE ("Bukti penegakan di BE — bukan sekadar tombol
 * disembunyikan di FE"). So this suite talks to the perlengkapan BACKEND API
 * directly with a real per-role JWT and asserts the tiered `bank_aset`
 * visibility the multi-satker seed was built to make observable.
 *
 * Two independent layers so a failure points at the right component:
 *   1. authenc mints a correct per-role token (decode the JWT, check claims).
 *   2. the backend enforces the scope (exact row counts + per-row isolation +
 *      object-level fail-closed + unauthenticated rejection).
 *
 * Pure API (no browser): uses Playwright's `request` fixture against
 * AUTHENC_URL (login) + PERLENGKAPAN_API_URL (layanan-perlengkapan). Requires
 * the e2e stack up AND the multi-satker fixture loaded
 * (tests/fixtures/e2e/seed-multisatker.sql).
 *
 * Every expectation is DERIVED from the environment under test — no row counts
 * and no satker codes are written down here. That is what lets one suite certify
 * both the empty CI stack and staging's real 624k-row SIMAN snapshot; see the
 * fixture-table comment in helpers/real-auth.ts for what the hard-coded form
 * did when it was first pointed at staging.
 */
import { test, expect } from '@playwright/test';
import {
  TEST_USERS,
  userByKey,
  credsFor,
  apiLogin,
  bankAsetList,
  bankAsetListJson,
  bankAsetDetail,
  adminMasterList,
  decodeJwtIdentity,
  deriveScope,
  scopedTotalFor,
} from './helpers/real-auth';

test.describe('Perlengkapan RBAC data-scoping (bank_aset)', () => {
  // ── Layer 1: authenc mints role + satker into each per-role JWT ───────────
  for (const user of TEST_USERS) {
    test(`authenc mints role+satker into the JWT — ${user.key}`, async ({ request }) => {
      const { accessToken } = await apiLogin(request, credsFor(user));
      const identity = decodeJwtIdentity(accessToken);
      expect(identity.satkerCode, `satker_code claim for ${user.key}`).toBe(user.satkerCode);
      expect(
        identity.realmRoles,
        `realm_access.roles for ${user.key} should include "${user.role}" (got ${JSON.stringify(identity.realmRoles)})`,
      ).toContain(user.role);
    });
  }

  // ── Layer 2: the backend enforces the scope ──────────────────────────────
  //
  // A satker-bound role must see exactly ONE satker. This is the leak detector:
  // a second satker in the result is a counter-example no matter how much data
  // the environment holds, and it needs no allowlist to recognise one.
  for (const user of TEST_USERS.filter((u) => u.tier === 'satker')) {
    test(`sees exactly one satker — ${user.key} (${user.role})`, async ({ request }) => {
      const { accessToken } = await apiLogin(request, credsFor(user));
      const scope = await deriveScope(request, accessToken);

      expect(scope.total, `${user.key} should see some assets`).toBeGreaterThan(0);
      expect(
        scope.satkerNames,
        `${user.key} (${user.role}) is satker-bound but saw ${scope.satkerNames.length} satkers`,
      ).toHaveLength(1);
    });
  }

  // The cross-path check: what a satker-bound user sees IMPLICITLY must equal
  // what a national user sees when filtering EXPLICITLY to that same satker.
  // Two independent code paths (`AsetScope::from_claims` vs the `satker` query
  // filter) agreeing on an exact total is strong evidence the scope is right —
  // and it holds whatever the underlying row count is.
  for (const user of TEST_USERS.filter((u) => u.tier === 'satker')) {
    test(`implicit scope equals explicit national filter — ${user.key}`, async ({ request }) => {
      const pusat = userByKey('validator_pusat');
      const tok = (await apiLogin(request, credsFor(user))).accessToken;
      const tokPusat = (await apiLogin(request, credsFor(pusat))).accessToken;

      const scope = await deriveScope(request, tok);
      const satkerName = scope.satkerNames[0];
      expect(satkerName, `${user.key} must resolve to a satker`).toBeTruthy();

      const national = await scopedTotalFor(request, tokPusat, satkerName);
      expect(
        scope.total,
        `${user.key} sees ${scope.total} rows for "${satkerName}" but a national ` +
          `user filtering to that satker sees ${national}`,
      ).toBe(national);
    });
  }

  // A filter must INTERSECT the caller's scope, never replace it. Asking for
  // someone else's satker returns nothing rather than widening access.
  test('a satker-bound user cannot widen scope via the satker filter', async ({ request }) => {
    const opA = userByKey('operator_a');
    const opB = userByKey('operator_b');
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokB = (await apiLogin(request, credsFor(opB))).accessToken;

    const foreignName = (await deriveScope(request, tokB)).satkerNames[0];
    expect(foreignName, 'operator_b must resolve to a satker').toBeTruthy();

    const widened = await scopedTotalFor(request, tokA, foreignName);
    expect(
      widened,
      `operator_a asked for "${foreignName}" and got ${widened} rows — the ` +
        `satker filter replaced the scope instead of intersecting it`,
    ).toBe(0);
  });

  // Tier ordering: a wilayah user must see at least its member satkers, and a
  // national user at least the wilayah. Exact totals via the explicit filter,
  // so this is not a sampled comparison.
  test('scope widens monotonically satker → wilayah → nasional', async ({ request }) => {
    const opA = userByKey('operator_a');
    const wil = userByKey('validator_wilayah');
    const pusat = userByKey('validator_pusat');
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokW = (await apiLogin(request, credsFor(wil))).accessToken;
    const tokP = (await apiLogin(request, credsFor(pusat))).accessToken;

    const scopeA = await deriveScope(request, tokA);
    const scopeW = await deriveScope(request, tokW);
    const scopeP = await deriveScope(request, tokP);
    const satkerA = scopeA.satkerNames[0];

    expect(scopeW.total, 'wilayah must be at least as wide as its satker').toBeGreaterThanOrEqual(scopeA.total);
    expect(scopeP.total, 'nasional must be at least as wide as wilayah').toBeGreaterThanOrEqual(scopeW.total);

    // Containment, exactly: the wilayah user sees operator_a's satker in full.
    expect(
      await scopedTotalFor(request, tokW, satkerA),
      `validator_wilayah must see all of "${satkerA}" (operator_a sees ${scopeA.total})`,
    ).toBe(scopeA.total);
  });

  // Satker-internal chain (#96): same satker ⇒ identical scope, whatever its size.
  for (const user of TEST_USERS.filter((u) => u.sameScopeAs)) {
    test(`${user.key} has the same scope as ${user.sameScopeAs}`, async ({ request }) => {
      const peer = userByKey(user.sameScopeAs!);
      const tok = (await apiLogin(request, credsFor(user))).accessToken;
      const tokPeer = (await apiLogin(request, credsFor(peer))).accessToken;

      const scope = await deriveScope(request, tok);
      const peerScope = await deriveScope(request, tokPeer);

      expect(scope.total, `${user.key} vs ${peer.key} total`).toBe(peerScope.total);
      expect(scope.satkerNames, `${user.key} vs ${peer.key} satkers`).toEqual(peerScope.satkerNames);
    });
  }

  // ── Cross-satker isolation: two operators see mutually disjoint data ──────
  test('operators in different satkers are mutually isolated', async ({ request }) => {
    const opA = userByKey('operator_a');
    const opB = userByKey('operator_b');
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokB = (await apiLogin(request, credsFor(opB))).accessToken;

    const kdA = new Set((await deriveScope(request, tokA)).kdCodes);
    const kdB = new Set((await deriveScope(request, tokB)).kdCodes);

    expect(kdA.size, 'operator_a should see at least one satker').toBeGreaterThan(0);
    expect(kdB.size, 'operator_b should see at least one satker').toBeGreaterThan(0);
    for (const kd of kdA) {
      expect(kdB.has(kd), `operator_b leaked operator_a's satker ${kd}`).toBeFalsy();
    }
  });

  // ── Object-level scope: detail fails closed across satkers, open within ───
  test('bank-aset detail fails closed across satkers (404), open within (200)', async ({
    request,
  }) => {
    const opA = userByKey('operator_a');
    const opB = userByKey('operator_b');
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokB = (await apiLogin(request, credsFor(opB))).accessToken;

    // Each operator's OWN list is by definition in-scope for them and — proven
    // by the isolation test above — out of scope for the other. Deriving the
    // two ids this way needs no national listing and no hard-coded codes.
    const own = (await bankAsetListJson(request, tokA, { per_page: 1 })).data[0];
    const foreign = (await bankAsetListJson(request, tokB, { per_page: 1 })).data[0];
    expect(own, 'operator_a must own at least one asset').toBeTruthy();
    expect(foreign, 'operator_b must own at least one asset').toBeTruthy();

    const ownResp = await bankAsetDetail(request, tokA, own!.id);
    expect(ownResp.status(), 'operator_a must read its own asset').toBe(200);

    const foreignResp = await bankAsetDetail(request, tokA, foreign!.id);
    expect(
      [403, 404],
      `operator_a must not read a foreign asset (fail closed); got ${foreignResp.status()}`,
    ).toContain(foreignResp.status());
  });

  // ── No anonymous access ───────────────────────────────────────────────────
  test('bank-aset rejects an unauthenticated request (401)', async ({ request }) => {
    const resp = await bankAsetList(request, null);
    expect(resp.status(), 'no Bearer token → unauthorized').toBe(401);
  });
});

test.describe('Perlengkapan admin API authZ — server-side (cross-satker required)', () => {
  // The backend `/admin/*` gate is `require_admin` = `is_cross_satker_role`, so
  // satker-bound roles get 403 while validator_pusat (cross-satker) is allowed —
  // intentionally WIDER than the FE `/admin` UI guard (`is_admin`, which denies
  // validator_pusat). Asserting both halves keeps that asymmetry from drifting.
  for (const user of TEST_USERS.filter((u) => u.role !== 'validator_pusat')) {
    test(`${user.key} (${user.role}) is forbidden (403) at the admin API`, async ({ request }) => {
      const { accessToken } = await apiLogin(request, credsFor(user));
      const resp = await adminMasterList(request, accessToken);
      expect(resp.status(), `${user.key} must be forbidden at /admin/master`).toBe(403);
    });
  }

  test('validator_pusat (cross-satker) is allowed (200) at the admin API', async ({ request }) => {
    const pusat = TEST_USERS.find((u) => u.key === 'validator_pusat')!;
    const { accessToken } = await apiLogin(request, credsFor(pusat));
    const resp = await adminMasterList(request, accessToken);
    expect(
      resp.status(),
      'validator_pusat is cross-satker → allowed at the admin API (unlike the FE UI)',
    ).toBe(200);
  });

  test('admin API rejects an unauthenticated request (401)', async ({ request }) => {
    const resp = await adminMasterList(request, null);
    expect(resp.status(), 'no Bearer token → unauthorized').toBe(401);
  });
});
