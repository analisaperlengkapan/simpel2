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
import { test, expect, type APIRequestContext } from '@playwright/test';
import {
  PERLENGKAPAN_API_URL,
  TEST_USERS,
  type PaginatedResponse,
  userByKey,
  credsFor,
  tokenFor,
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

// ═══════════════════════════════════════════════════════════════════════════
// Object-level scoping on penghapusan BMN (#66 follow-through).
//
// The list endpoint was scoped; every BY-ID endpoint of the module was not.
// Role gating answered "may this role ever do this" and stopped there, so an
// operator holding `operator_satker` could read, edit, and drive the workflow
// of a usulan belonging to a satker that is not theirs — `update` did not even
// check the role, taking `_claims` and ignoring it entirely.
//
// The ids are DERIVED: each operator's own first row is by construction
// in-scope for them and out of scope for the other (proven by the isolation
// test above), so this needs no hard-coded UUID and works against the CI seed
// and staging alike.
//
// Every probe below is non-destructive. The PUT body is `{}` — the update is
// COALESCE-based, so even a leak would change no column value. Nothing here
// advances a workflow state, which keeps the spec re-runnable after a retry
// (the state-pollution trap that made 36 of 39 failures self-inflicted once).
// ═══════════════════════════════════════════════════════════════════════════
test.describe('Penghapusan BMN object-level scoping — server-side', () => {
  const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/penghapusan-bmn`;
  const bearer = (token: string) => ({ Authorization: `Bearer ${token}` });

  /** First usulan visible to `token`, or null when that operator has none. */
  async function firstUsulan(request: APIRequestContext, token: string) {
    const resp = await request.get(`${API}?per_page=1`, { headers: bearer(token) });
    if (!resp.ok()) throw new Error(`penghapusan list failed (${resp.status()}): ${await resp.text()}`);
    const body = (await resp.json()) as PaginatedResponse<{ id: string; satker_code?: string }>;
    return body.data[0] ?? null;
  }

  /** Both operators' ids in one place — every test below needs the same pair. */
  async function idPair(request: APIRequestContext) {
    const tokA = await tokenFor(request, 'operator_a');
    const tokB = await tokenFor(request, 'operator_b');
    const own = await firstUsulan(request, tokA);
    const foreign = await firstUsulan(request, tokB);
    expect(own, 'operator_a must own at least one usulan penghapusan').toBeTruthy();
    expect(foreign, 'operator_b must own at least one usulan penghapusan').toBeTruthy();
    expect(
      own!.id,
      'the seed must give the two operators DIFFERENT usulan, or this suite proves nothing',
    ).not.toBe(foreign!.id);
    return { tokA, ownId: own!.id, foreignId: foreign!.id };
  }

  // Positive control first: if these went red the negatives below would pass
  // for the wrong reason (everything denied is not the same as scoped).
  test('an operator reads its OWN usulan (200) on both read surfaces', async ({ request }) => {
    const { tokA, ownId } = await idPair(request);

    const detail = await request.get(`${API}/${ownId}/detail`, { headers: bearer(tokA) });
    expect(detail.status(), 'operator_a must read the detail of its own usulan').toBe(200);

    const one = await request.get(`${API}/${ownId}`, { headers: bearer(tokA) });
    expect(one.status(), 'operator_a must read its own usulan').toBe(200);
  });

  // The read surfaces. 404 rather than 403 on purpose: a 403 confirms the id
  // exists in someone else's satker, which is the existence oracle #93 closed
  // for satker detail. 403 is accepted so the assertion tests containment
  // rather than one particular status.
  const readSurfaces: Array<[string, (id: string) => string]> = [
    ['GET /{id}', (id) => `${API}/${id}`],
    ['GET /{id}/detail', (id) => `${API}/${id}/detail`],
    ['GET /{id}/verifikasi-siman', (id) => `${API}/${id}/verifikasi-siman`],
    ['GET /{id}/lampiran', (id) => `${API}/${id}/lampiran`],
    ['GET /{id}/document', (id) => `${API}/${id}/document`],
  ];

  for (const [label, url] of readSurfaces) {
    test(`${label} fails closed across satkers`, async ({ request }) => {
      const { tokA, foreignId } = await idPair(request);
      const resp = await request.get(url(foreignId), { headers: bearer(tokA) });
      expect(
        [403, 404],
        `operator_a must not read another satker's usulan via ${label}; got ${resp.status()}`,
      ).toContain(resp.status());
    });
  }

  // The write surface that had no gate at all. This is the one that mattered:
  // a PUT here rewrote another satker's usulan and returned 200.
  test('PUT /{id} fails closed across satkers', async ({ request }) => {
    const { tokA, foreignId } = await idPair(request);
    const resp = await request.put(`${API}/${foreignId}`, {
      headers: bearer(tokA),
      data: {},
    });
    expect(
      [403, 404],
      `operator_a must not edit another satker's usulan; got ${resp.status()}`,
    ).toContain(resp.status());
  });

  // Workflow moves. operator_a genuinely HOLDS operator_satker, so the role
  // gate passes and only the object scope can stop this — which is exactly the
  // hole: role-gated is not the same as scoped.
  test('POST /{id}/submit-wilayah fails closed across satkers', async ({ request }) => {
    const { tokA, foreignId } = await idPair(request);
    const resp = await request.post(`${API}/${foreignId}/submit-wilayah`, {
      headers: bearer(tokA),
      data: { catatan: 'e2e cross-satker probe' },
    });
    expect(
      [403, 404],
      `operator_a must not submit another satker's usulan; got ${resp.status()}`,
    ).toContain(resp.status());
  });

  // A validator_wilayah is scoped to its wilayah, not to "any usulan awaiting
  // a wilayah decision". Whether the two seeded satkers share a wilayah is an
  // environment fact, so derive the expectation instead of asserting a status:
  // the validator may act only on what its own list shows it.
  test('a validator_wilayah can only act on usulan its own list contains', async ({ request }) => {
    const tokW = await tokenFor(request, 'validator_wilayah');
    const tokB = await tokenFor(request, 'operator_b');

    const visible = await request.get(`${API}?per_page=200`, { headers: bearer(tokW) });
    expect(visible.status(), 'validator_wilayah must be able to list').toBe(200);
    const ids = new Set(
      ((await visible.json()) as PaginatedResponse<{ id: string }>).data.map((r) => r.id),
    );

    const foreign = await firstUsulan(request, tokB);
    expect(foreign, 'operator_b must own at least one usulan penghapusan').toBeTruthy();

    const resp = await request.get(`${API}/${foreign!.id}`, { headers: bearer(tokW) });
    if (ids.has(foreign!.id)) {
      expect(
        resp.status(),
        'the usulan IS in the validator wilayah list, so reading it must succeed',
      ).toBe(200);
    } else {
      expect(
        [403, 404],
        `the usulan is NOT in the validator wilayah list, so reading it must fail closed; got ${resp.status()}`,
      ).toContain(resp.status());
    }
  });

  test('penghapusan rejects an unauthenticated by-id request (401)', async ({ request }) => {
    const tokB = await tokenFor(request, 'operator_b');
    const foreign = await firstUsulan(request, tokB);
    const resp = await request.get(`${API}/${foreign!.id}`);
    expect(resp.status(), 'no Bearer token → unauthorized').toBe(401);
  });
});

/**
 * Pakaian dinas — employee surfaces, object-level scoping, server-side.
 *
 * These endpoints return personal data (NIP, name, phone, gender, rank) and
 * uniform measurements for a satker the CALLER names in the path, and accept
 * profile writes for an employee the caller names by NIP. Every one of them
 * took an unused `_claims`, and the gap was measured against deployed staging
 * rather than inferred: operator_a (0200010) read operator_b's (0200020)
 * roster on all three read paths, and wrote a profile onto b's employee — 200
 * each time, the row rewritten.
 *
 * Non-destructive by construction: the cross-satker write must be REFUSED, so
 * a passing run changes nothing and a retry sees the same state. The one write
 * that is expected to succeed is idempotent (same values every run).
 */
test.describe('Pakaian dinas employee scoping — server-side', () => {
  const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;
  const bearer = (token: string) => ({ Authorization: `Bearer ${token}` });

  /** The three read paths, all keyed by a satker code the caller supplies. */
  const rosterPaths = (satker: string) => [
    `${API}/pegawai-satker/${satker}`,
    `${API}/pegawai-satker/${satker}/with-sizes`,
    `${API}/pegawai-satker/${satker}/roster`,
  ];

  test('an operator reads its own satker roster', async ({ request }) => {
    const tok = await tokenFor(request, 'operator_a');
    const own = userByKey('operator_a').satkerCode;
    for (const url of rosterPaths(own)) {
      const resp = await request.get(url, { headers: bearer(tok) });
      expect(resp.status(), `own roster must stay readable: ${url}`).toBe(200);
    }
  });

  test('an operator cannot read another satker roster', async ({ request }) => {
    const tok = await tokenFor(request, 'operator_a');
    const foreign = userByKey('operator_b').satkerCode;
    expect(foreign, 'the two operators must sit in DIFFERENT satkers').not.toBe(
      userByKey('operator_a').satkerCode,
    );

    for (const url of rosterPaths(foreign)) {
      const resp = await request.get(url, { headers: bearer(tok) });
      // 404, not 403: a 403 confirms the satker exists and has staff, which is
      // the cross-tenant existence oracle #93 closed.
      expect(
        resp.status(),
        `cross-satker roster must fail closed: ${url} -> ${await resp.text()}`,
      ).toBe(404);
    }
  });

  test('the roster endpoints reject an unauthenticated request (401)', async ({ request }) => {
    for (const url of rosterPaths(userByKey('operator_a').satkerCode)) {
      const resp = await request.get(url);
      expect(resp.status(), `no Bearer token → unauthorized: ${url}`).toBe(401);
    }
  });

  test('an operator cannot write a profile onto another satker employee', async ({ request }) => {
    const tokA = await tokenFor(request, 'operator_a');
    const tokB = await tokenFor(request, 'operator_b');
    const foreignSatker = userByKey('operator_b').satkerCode;

    // Take the target NIP from b's OWN roster, so the test names a real
    // employee of the other satker rather than a guess.
    const rosterB = await request.get(`${API}/pegawai-satker/${foreignSatker}`, {
      headers: bearer(tokB),
    });
    expect(rosterB.status(), 'operator_b must be able to read its own roster').toBe(200);
    const staff = ((await rosterB.json()) as { data: Array<{ nip: string }> }).data;
    expect(staff.length, 'operator_b satker must have at least one employee seeded').toBeGreaterThan(0);
    const targetNip = staff[0].nip;

    const resp = await request.post(`${API}/pegawai-profile`, {
      headers: bearer(tokA),
      data: {
        nip: targetNip,
        nama: 'DITULIS OLEH OPERATOR SATKER LAIN',
        // The body used to decide which satker the row belonged to. Claiming
        // the victim's satker is exactly what made the staging probe land.
        kode_satker: foreignSatker,
        ukuran_baju: 'XXL',
        with_hijab: false,
      },
    });
    expect(
      resp.status(),
      `cross-satker profile write must be refused: ${await resp.text()}`,
    ).toBe(404);

    // Refused must mean nothing was written. Read it back as the satker that
    // OWNS the record, so a leak would be visible rather than merely absent.
    const after = await request.get(`${API}/pegawai-satker/${foreignSatker}/with-sizes`, {
      headers: bearer(tokB),
    });
    expect(after.status()).toBe(200);
    const rows = ((await after.json()) as {
      data: Array<{ pegawai: { nip: string }; existing_sizes: { nama?: string } | null }>;
    }).data;
    const victim = rows.find((r) => r.pegawai.nip === targetNip);
    expect(victim, 'the target employee must still be in the roster').toBeTruthy();
    expect(
      victim!.existing_sizes?.nama ?? '',
      'a refused write must leave no trace on the record',
    ).not.toContain('OPERATOR SATKER LAIN');
  });

  test('a profile write takes its satker from the source of truth, not the body', async ({
    request,
  }) => {
    const tok = await tokenFor(request, 'operator_a');
    const own = userByKey('operator_a').satkerCode;
    const foreign = userByKey('operator_b').satkerCode;

    const roster = await request.get(`${API}/pegawai-satker/${own}`, { headers: bearer(tok) });
    expect(roster.status()).toBe(200);
    const staff = ((await roster.json()) as { data: Array<{ nip: string }> }).data;
    expect(staff.length, 'operator_a satker must have at least one employee seeded').toBeGreaterThan(0);

    // In scope for this employee, but lying about which satker they belong to.
    // Idempotent: same values every run, so retries are safe.
    const resp = await request.post(`${API}/pegawai-profile`, {
      headers: bearer(tok),
      data: {
        nip: staff[0].nip,
        kode_satker: foreign,
        ukuran_baju: 'L',
        with_hijab: false,
      },
    });
    expect(resp.status(), await resp.text()).toBe(200);
    const saved = (await resp.json()) as { data: { kode_satker: string } };
    expect(
      saved.data.kode_satker,
      'the satker written must come from MySIMKARI, not from the request body',
    ).toBe(own);
  });
});

/**
 * Pakaian dinas — a campaign's per-satker surfaces, server-side.
 *
 * The campaign itself is nationwide and pusat-authored; a satker's RESPONSE to
 * it is not. Measured against deployed staging (rc29), operator_a (0200010)
 * saw operator_b's (0200020) participation row on the campaign's satker list,
 * read it by id, read its approval trail — names and NIPs of the validators
 * who acted — and got the NATIONAL employee list back from the daftar report.
 *
 * Read-only: nothing here mutates, so retries see the same state.
 */
test.describe('Pakaian dinas campaign scoping — server-side', () => {
  const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pakaian-dinas`;
  const bearer = (token: string) => ({ Authorization: `Bearer ${token}` });

  /** A campaign both operators can see, plus its participation rows as pusat sees them. */
  async function campaignWithTwoParticipants(request: APIRequestContext) {
    const tokP = await tokenFor(request, 'validator_pusat');
    const list = await request.get(`${API}/pengajuan?per_page=50`, { headers: bearer(tokP) });
    expect(list.status(), 'pusat must be able to list campaigns').toBe(200);
    const campaigns = ((await list.json()) as PaginatedResponse<{ id: string }>).data;

    for (const c of campaigns) {
      const resp = await request.get(`${API}/pengajuan/${c.id}/satker?per_page=50`, {
        headers: bearer(tokP),
      });
      if (!resp.ok()) continue;
      const rows = ((await resp.json()) as PaginatedResponse<{ id: string; satker_id: string }>)
        .data;
      const own = rows.find((r) => r.satker_id === userByKey('operator_a').satkerCode);
      const foreign = rows.find((r) => r.satker_id === userByKey('operator_b').satkerCode);
      if (own && foreign) return { campaignId: c.id, own, foreign };
    }
    return null;
  }

  test('the participant list shows a satker only its own row', async ({ request }) => {
    const found = await campaignWithTwoParticipants(request);
    expect(
      found,
      'the seed must give ONE campaign participation rows for BOTH operators, or this suite proves nothing',
    ).toBeTruthy();

    const tokA = await tokenFor(request, 'operator_a');
    const resp = await request.get(`${API}/pengajuan/${found!.campaignId}/satker?per_page=50`, {
      headers: bearer(tokA),
    });
    expect(resp.status()).toBe(200);
    const body = (await resp.json()) as PaginatedResponse<{ satker_id: string }>;
    const others = body.data.filter((r) => r.satker_id !== userByKey('operator_a').satkerCode);
    expect(others, `operator_a must see only its own participation row`).toEqual([]);
    // The total is scoped too — a scoped page beside a national total still
    // says how many participants are being withheld.
    expect(body.total).toBe(body.data.length);
  });

  test('a participation row and its approval trail of another satker are closed', async ({
    request,
  }) => {
    const found = await campaignWithTwoParticipants(request);
    expect(found).toBeTruthy();
    const tokA = await tokenFor(request, 'operator_a');

    // Positive control: the caller's own row is readable.
    const own = await request.get(`${API}/satker/${found!.own.id}`, { headers: bearer(tokA) });
    expect(own.status(), `own participation row: ${await own.text()}`).toBe(200);

    const foreign = await request.get(`${API}/satker/${found!.foreign.id}`, {
      headers: bearer(tokA),
    });
    expect(
      foreign.status(),
      `another satker's participation row: ${await foreign.text()}`,
    ).toBe(404);

    // The trail names who acted, with their NIPs. Out of scope it is empty
    // rather than 404 — indistinguishable from "nothing has happened yet",
    // which is the point: no oracle either way.
    const trail = await request.get(`${API}/satker/${found!.foreign.id}/aktivitas`, {
      headers: bearer(tokA),
    });
    expect(trail.status()).toBe(200);
    expect(((await trail.json()) as { data: unknown[] }).data).toEqual([]);
  });

  test('the daftar report is scoped and a client filter can only narrow', async ({ request }) => {
    const found = await campaignWithTwoParticipants(request);
    expect(found).toBeTruthy();
    const tokA = await tokenFor(request, 'operator_a');
    const ownSatker = userByKey('operator_a').satkerCode;
    const foreignSatker = userByKey('operator_b').satkerCode;
    const base = `${API}/laporan/daftar-pegawai?pengajuan_id=${found!.campaignId}&page=1&per_page=100`;

    const mine = await request.get(base, { headers: bearer(tokA) });
    expect(mine.status(), await mine.text()).toBe(200);
    const rows = ((await mine.json()) as PaginatedResponse<{ satker_nama?: string }>).data;
    // Derived, not hard-coded: whatever the seed contains, none of it may come
    // from the other operator's satker.
    const namesResp = await request.get(`${API}/pegawai-satker/${ownSatker}`, {
      headers: bearer(tokA),
    });
    expect(namesResp.status()).toBe(200);

    const foreignRoster = await request.get(`${API}/pegawai-satker/${foreignSatker}`, {
      headers: bearer(await tokenFor(request, 'operator_b')),
    });
    const foreignNips = new Set(
      ((await foreignRoster.json()) as { data: Array<{ nip: string }> }).data.map((p) => p.nip),
    );
    const leaked = rows.filter((r) => foreignNips.has((r as { nip?: string }).nip ?? ''));
    expect(leaked, 'the national employee list used to come back here').toEqual([]);

    // Naming the other satker narrows to nothing rather than widening to them.
    const narrowed = await request.get(`${base}&satker_id=${foreignSatker}`, {
      headers: bearer(tokA),
    });
    expect(narrowed.status()).toBe(200);
    expect(((await narrowed.json()) as PaginatedResponse<unknown>).data).toEqual([]);
  });

  test('only pusat may author a campaign', async ({ request }) => {
    const tokA = await tokenFor(request, 'operator_a');
    const resp = await request.post(`${API}/pengajuan`, {
      headers: bearer(tokA),
      data: {
        nama: 'E2E RBAC probe — must be refused',
        tahun: 2026,
        tgl_mulai: '2026-01-01',
        tgl_selesai: '2026-12-31',
        pilihan_satker: 'semua',
        spesifikasi_ids: ['00000000-0000-0000-0000-000000000000'],
      },
    });
    // 403 before any business validation — the point is that authorization
    // decides, not that the payload happens to be incomplete.
    expect(resp.status(), `a satker operator must not author a campaign: ${await resp.text()}`).toBe(
      403,
    );
  });
});

/**
 * Kebutuhan BMN — per-satker surfaces, server-side.
 *
 * #93 scoped ONE endpoint on the `satker/{id}` path — the detail — and its
 * siblings kept reading the row unscoped. Measured against deployed staging
 * (rc29), operator_a (0200010) against another satker's participation row:
 *
 * ```text
 * GET .../satker/{foreign}            -> 404   (#93, working)
 * GET .../satker/{foreign}/aktivitas  -> 200, the trail, with each
 *                                        validator's user id and NIP
 * GET .../satker/{foreign}/analisis   -> 200, their whole feasibility case
 * GET .../satker/{foreign}/laporan/download -> 200, a 4 973-byte PDF
 * GET .../pengajuan/{id}/satker       -> 200, every participant, including
 *                                        one in another wilayah
 * ```
 *
 * Read-only, so retries see the same state.
 */
test.describe('Kebutuhan BMN satker scoping — server-side', () => {
  const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/kebutuhan-bmn`;
  const bearer = (token: string) => ({ Authorization: `Bearer ${token}` });

  /** A campaign whose participants include both operators' satkers. */
  async function campaignWithBothOperators(request: APIRequestContext) {
    const tokP = await tokenFor(request, 'validator_pusat');
    const list = await request.get(`${API}/pengajuan?per_page=50`, { headers: bearer(tokP) });
    expect(list.status(), 'pusat must be able to list kebutuhan campaigns').toBe(200);
    const campaigns = ((await list.json()) as PaginatedResponse<{ id: string }>).data;

    for (const c of campaigns) {
      const resp = await request.get(`${API}/pengajuan/${c.id}/satker`, { headers: bearer(tokP) });
      if (!resp.ok()) continue;
      const rows = ((await resp.json()) as { data: Array<{ id: string; satker_id: string }> }).data;
      const own = rows.find((r) => r.satker_id === userByKey('operator_a').satkerCode);
      const foreign = rows.find((r) => r.satker_id === userByKey('operator_b').satkerCode);
      if (own && foreign) return { campaignId: c.id, own, foreign, all: rows };
    }
    return null;
  }

  test('the participant list shows a satker only its own row', async ({ request }) => {
    const found = await campaignWithBothOperators(request);
    expect(
      found,
      'the seed must give ONE kebutuhan campaign participation rows for BOTH operators',
    ).toBeTruthy();

    const tokA = await tokenFor(request, 'operator_a');
    const resp = await request.get(`${API}/pengajuan/${found!.campaignId}/satker`, {
      headers: bearer(tokA),
    });
    expect(resp.status()).toBe(200);
    const rows = ((await resp.json()) as { data: Array<{ satker_id: string }> }).data;
    expect(
      rows.filter((r) => r.satker_id !== userByKey('operator_a').satkerCode),
      'an operator must see only its own participation row',
    ).toEqual([]);
    // The positive half: it must still see its OWN row, or this proves nothing.
    expect(rows.length).toBe(1);
  });

  test('the approval trail of another satker is closed', async ({ request }) => {
    const found = await campaignWithBothOperators(request);
    expect(found).toBeTruthy();
    const tokA = await tokenFor(request, 'operator_a');
    const tokB = await tokenFor(request, 'operator_b');

    // Establish that the foreign trail is non-empty for the satker that OWNS
    // it — otherwise an empty answer below would prove nothing.
    const ownersView = await request.get(`${API}/satker/${found!.foreign.id}/aktivitas`, {
      headers: bearer(tokB),
    });
    expect(ownersView.status()).toBe(200);
    const ownerRows = ((await ownersView.json()) as { data: unknown[] }).data;

    const resp = await request.get(`${API}/satker/${found!.foreign.id}/aktivitas`, {
      headers: bearer(tokA),
    });
    expect(resp.status()).toBe(200);
    const leaked = ((await resp.json()) as { data: unknown[] }).data;
    expect(leaked, 'the trail names every validator who acted, with their NIP').toEqual([]);
    expect(
      ownerRows.length,
      'the owning satker must see a non-empty trail, or the assertion above is vacuous',
    ).toBeGreaterThan(0);
  });

  test('the feasibility analysis and its PDF are closed across satkers', async ({ request }) => {
    const found = await campaignWithBothOperators(request);
    expect(found).toBeTruthy();
    const tokA = await tokenFor(request, 'operator_a');

    for (const suffix of ['analisis', 'laporan/preview', 'laporan/download']) {
      const own = await request.get(`${API}/satker/${found!.own.id}/${suffix}`, {
        headers: bearer(tokA),
      });
      expect(own.status(), `own ${suffix}: ${await own.text()}`).toBe(200);

      const foreign = await request.get(`${API}/satker/${found!.foreign.id}/${suffix}`, {
        headers: bearer(tokA),
      });
      // 404, not 403 — see #93.
      expect(foreign.status(), `another satker's ${suffix} was served`).toBe(404);
    }
  });
});

/**
 * Pemakaian BMN — the surfaces reached BY PERMIT ID, server-side.
 *
 * The LIST has been scoped since #66 and is asserted above. The detail behind
 * it was not. Measured against deployed staging (rc29), operator_a (0200010)
 * against operator_b's permit (0200020):
 *
 * ```text
 * GET /pemakaian-bmn/{foreign}             -> 200, the whole record
 * GET /pemakaian-bmn/{foreign}/sk-izin.pdf -> 200, application/pdf, 5 057 bytes
 * ```
 *
 * A permit names a person, an asset and a period — the fields the stakeholder
 * asked to be limited per role. Read-only, so retries see the same state.
 */
test.describe('Pemakaian BMN permit scoping — server-side', () => {
  const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pemakaian-bmn`;
  const bearer = (token: string) => ({ Authorization: `Bearer ${token}` });

  /** One permit each operator owns, taken from their OWN scoped list. */
  async function permitPair(request: APIRequestContext) {
    const tokA = await tokenFor(request, 'operator_a');
    const tokB = await tokenFor(request, 'operator_b');

    const mine = await request.get(`${API}?per_page=50`, { headers: bearer(tokA) });
    expect(mine.status(), 'operator_a must be able to list permits').toBe(200);
    const own = ((await mine.json()) as PaginatedResponse<{ id: string }>).data[0];

    const theirs = await request.get(`${API}?per_page=50`, { headers: bearer(tokB) });
    expect(theirs.status(), 'operator_b must be able to list permits').toBe(200);
    const foreign = ((await theirs.json()) as PaginatedResponse<{ id: string }>).data[0];

    expect(own, 'operator_a must own at least one permit').toBeTruthy();
    expect(foreign, 'operator_b must own at least one permit').toBeTruthy();
    expect(
      own!.id,
      'the seed must give the two operators DIFFERENT permits, or this suite proves nothing',
    ).not.toBe(foreign!.id);
    return { tokA, ownId: own!.id, foreignId: foreign!.id };
  }

  test('a permit detail is closed across satkers', async ({ request }) => {
    const { tokA, ownId, foreignId } = await permitPair(request);

    const own = await request.get(`${API}/${ownId}`, { headers: bearer(tokA) });
    expect(own.status(), `own permit: ${await own.text()}`).toBe(200);

    const foreign = await request.get(`${API}/${foreignId}`, { headers: bearer(tokA) });
    // 404, not 403: 403 would confirm the permit exists under another satker.
    expect(foreign.status(), `another satker's permit: ${await foreign.text()}`).toBe(404);
    expect(
      await foreign.text(),
      'the holder should not be named in the refusal',
    ).not.toContain('pegawai_nama');
  });

  test('the decision letter is closed across satkers', async ({ request }) => {
    const { tokA, ownId, foreignId } = await permitPair(request);

    const own = await request.get(`${API}/${ownId}/sk-izin.pdf`, { headers: bearer(tokA) });
    expect(own.status(), `own SK: ${own.status()}`).toBe(200);
    expect(own.headers()['content-type']).toContain('application/pdf');

    const foreign = await request.get(`${API}/${foreignId}/sk-izin.pdf`, {
      headers: bearer(tokA),
    });
    expect(foreign.status(), "another satker's decision letter was served").toBe(404);
  });

  test('the permit surfaces reject an unauthenticated request (401)', async ({ request }) => {
    const { ownId } = await permitPair(request);
    for (const url of [`${API}/${ownId}`, `${API}/${ownId}/sk-izin.pdf`]) {
      const resp = await request.get(url);
      expect(resp.status(), `no Bearer token → unauthorized: ${url}`).toBe(401);
    }
  });
});
