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
 * (tests/fixtures/e2e/seed-multisatker.sql). Counts are exact because the CI
 * stack starts from an empty dbsimpelv2 that only this seed populates.
 */
import { test, expect } from '@playwright/test';
import {
  TEST_USERS,
  credsFor,
  apiLogin,
  bankAsetList,
  bankAsetListJson,
  bankAsetDetail,
  adminMasterList,
  decodeJwtIdentity,
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

  // ── Layer 2: the backend enforces the scope (exact counts + isolation) ────
  for (const user of TEST_USERS) {
    test(`bank-aset scoped to ${user.expectedAsetCount} row(s) — ${user.key} (${user.role})`, async ({
      request,
    }) => {
      const { accessToken } = await apiLogin(request, credsFor(user));
      const body = await bankAsetListJson(request, accessToken);

      expect(body.total, `total visible to ${user.key}`).toBe(user.expectedAsetCount);
      expect(body.data.length, `rows returned to ${user.key}`).toBe(user.expectedAsetCount);

      // Per-row isolation: every visible asset belongs to an allowed satker.
      const seen = [...new Set(body.data.map((a) => a.kode_satker))].sort();
      for (const kd of seen) {
        expect(
          user.allowedKdsatkerKeu,
          `${user.key} (${user.role}) leaked an asset of kdsatker_keu=${kd}`,
        ).toContain(kd);
      }
    });
  }

  // ── Cross-satker isolation: two operators see mutually disjoint data ──────
  test('operators in different satkers are mutually isolated', async ({ request }) => {
    const opA = TEST_USERS.find((u) => u.key === 'operator_a')!;
    const opB = TEST_USERS.find((u) => u.key === 'operator_b')!;
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokB = (await apiLogin(request, credsFor(opB))).accessToken;

    const kdA = new Set((await bankAsetListJson(request, tokA)).data.map((a) => a.kode_satker));
    const kdB = new Set((await bankAsetListJson(request, tokB)).data.map((a) => a.kode_satker));

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
    const opA = TEST_USERS.find((u) => u.key === 'operator_a')!;
    const pusat = TEST_USERS.find((u) => u.key === 'validator_pusat')!;
    const tokA = (await apiLogin(request, credsFor(opA))).accessToken;
    const tokPusat = (await apiLogin(request, credsFor(pusat))).accessToken;

    // validator_pusat sees everything → use it to discover concrete asset ids.
    const all = (await bankAsetListJson(request, tokPusat)).data;
    const own = all.find((a) => opA.allowedKdsatkerKeu.includes(a.kode_satker));
    const foreign = all.find((a) => !opA.allowedKdsatkerKeu.includes(a.kode_satker));
    expect(own, 'seed must contain an operator_a-owned asset').toBeTruthy();
    expect(foreign, 'seed must contain an asset outside operator_a').toBeTruthy();

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
