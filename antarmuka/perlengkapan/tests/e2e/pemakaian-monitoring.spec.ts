/**
 * Monitoring Pemakaian BMN — per-role data scoping against the REAL stack.
 *
 * Stakeholder requirement: pemakaian BMN harus bisa dimonitor **dengan batasan
 * tiap role** — pusat semua, wilayah hanya wilayahnya, satker hanya satkernya —
 * menampilkan satker mana, nama barangnya, NUP berapa, nama pegawai yang
 * memakai, dan jangka waktu pemakaiannya.
 *
 * Before this, the monitoring endpoints checked the caller's ROLE and nothing
 * else; the satker filter arrived as a query parameter, so an `operator_satker`
 * who simply omitted it received the national picture. Six read endpoints
 * shared that shape.
 *
 * Two layers, so a failure names the guilty component:
 *   1. the API enforces the tiers (satker ⊂ wilayah ⊆ nasional, exact isolation)
 *   2. the page renders the named columns for the rows the caller may see
 *
 * Expectations are DERIVED from the environment, not written down: no row
 * counts, no satker codes. The one thing asserted absolutely is the isolation
 * property, which needs no fixture knowledge to recognise a counter-example.
 * The suite is non-vacuous by construction — it first asserts each tier sees at
 * least one permit, so "scope returns nothing" cannot masquerade as "scope
 * works".
 */
import { test, expect, type APIRequestContext } from '@playwright/test';
import {
  PERLENGKAPAN_API_URL,
  TEST_USERS,
  apiLogin,
  credsFor,
  storageStatePath,
} from './helpers/real-auth';

const MON_API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan/pemakaian-bmn/monitoring`;
/** App base path, same convention as pengelolaan-workflow.spec.ts. */
const BASE = '/perlengkapan/simpel/v2';

const userFor = (key: string) => {
  const u = TEST_USERS.find((t) => t.key === key);
  if (!u) throw new Error(`unknown test user ${key}`);
  return u;
};

interface MonitoringRow {
  id: string;
  satker_code: string | null;
  satker_nama: string | null;
  kode_barang: string;
  nama_barang: string;
  nup: string;
  merk_tipe: string | null;
  pegawai_nip: string;
  pegawai_nama: string;
  tanggal_mulai: string;
  tanggal_selesai: string;
  durasi_hari: number;
  sisa_hari: number;
  status: string;
}

async function monitoringRows(
  request: APIRequestContext,
  userKey: string,
  query = '',
): Promise<MonitoringRow[]> {
  const { accessToken } = await apiLogin(request, credsFor(userFor(userKey)));
  const resp = await request.get(`${MON_API}/pemakaian?page=1&per_page=100${query}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  expect(resp.ok(), `GET monitoring/pemakaian as ${userKey} (${resp.status()})`).toBeTruthy();
  const body = await resp.json();
  return body.data?.data ?? [];
}

const satkersIn = (rows: MonitoringRow[]) =>
  [...new Set(rows.map((r) => r.satker_code).filter((c): c is string => !!c))].sort();

// ── Layer 1: the API enforces the tiers ───────────────────────────────────
test.describe('Monitoring pemakaian BMN — API scoping', () => {
  test('a satker-bound caller sees exactly their own satker', async ({ request }) => {
    const user = userFor('operator_a');
    const rows = await monitoringRows(request, 'operator_a');

    // Non-vacuous first: an empty result would satisfy every isolation
    // assertion below while proving nothing.
    expect(rows.length, 'operator_a should see at least one permit').toBeGreaterThan(0);
    expect(satkersIn(rows), 'operator_a is satker-bound').toEqual([user.satkerCode]);
  });

  test('the tiers nest: satker ⊂ wilayah ⊆ nasional', async ({ request }) => {
    const satker = satkersIn(await monitoringRows(request, 'operator_a'));
    const wilayah = satkersIn(await monitoringRows(request, 'validator_wilayah'));
    const pusat = satkersIn(await monitoringRows(request, 'validator_pusat'));

    expect(satker.length, 'satker tier non-empty').toBeGreaterThan(0);
    for (const code of satker) {
      expect(wilayah, `wilayah must contain ${code}`).toContain(code);
    }
    for (const code of wilayah) {
      expect(pusat, `pusat must contain ${code}`).toContain(code);
    }
    // The seed puts a second satker in the same wilayah precisely so this is a
    // real comparison rather than two equal sets.
    expect(
      wilayah.length,
      `wilayah tier should be wider than one satker (saw ${JSON.stringify(wilayah)})`,
    ).toBeGreaterThan(satker.length);

    // …and STRICTLY narrower than nasional. Every assertion above is satisfied
    // by a wilayah tier that leaks the whole country, since a superset relation
    // holds trivially when the two sets are equal. The seed therefore puts one
    // ACTIVE permit under a DIFFERENT Kejati (Bandung, 0300010) with no user of
    // its own, purely so this comparison has something to exclude.
    const OTHER_KEJATI = '0300010';
    expect(
      pusat,
      `nasional tier must include the out-of-wilayah satker ${OTHER_KEJATI}; ` +
        'without it this test proves nothing about exclusion',
    ).toContain(OTHER_KEJATI);
    expect(
      wilayah,
      `wilayah tier leaked ${OTHER_KEJATI}, which sits under another Kejati ` +
        `(saw ${JSON.stringify(wilayah)})`,
    ).not.toContain(OTHER_KEJATI);
  });

  test('two satker-bound callers never see each other', async ({ request }) => {
    const a = await monitoringRows(request, 'operator_a');
    const b = await monitoringRows(request, 'operator_b');
    expect(a.length).toBeGreaterThan(0);
    expect(b.length).toBeGreaterThan(0);

    const idsB = new Set(b.map((r) => r.id));
    for (const row of a) {
      expect(idsB.has(row.id), `permit ${row.id} visible to BOTH operators`).toBeFalsy();
    }
  });

  test('a client-supplied satker_code can only narrow, never widen', async ({ request }) => {
    const other = userFor('operator_b').satkerCode;
    // This is the exact request that used to leak: the parameter was the only
    // thing deciding visibility.
    const rows = await monitoringRows(request, 'operator_a', `&satker_code=${other}`);
    expect(satkersIn(rows), `asking for ${other} must not return it`).not.toContain(other);
  });

  test('summary cards count only the caller’s own scope', async ({ request }) => {
    const read = async (key: string) => {
      const { accessToken } = await apiLogin(request, credsFor(userFor(key)));
      const resp = await request.get(`${MON_API}/summary`, {
        headers: { Authorization: `Bearer ${accessToken}` },
      });
      expect(resp.ok(), `GET monitoring/summary as ${key} (${resp.status()})`).toBeTruthy();
      return (await resp.json()).data;
    };

    const satker = await read('operator_a');
    const wilayah = await read('validator_wilayah');
    const pusat = await read('validator_pusat');

    expect(satker.sedang_dipakai, 'satker card non-vacuous').toBeGreaterThan(0);
    expect(wilayah.sedang_dipakai).toBeGreaterThanOrEqual(satker.sedang_dipakai);
    expect(pusat.sedang_dipakai).toBeGreaterThanOrEqual(wilayah.sedang_dipakai);
    // Strictly wider: the second satker's ACTIVE permit is inside the wilayah
    // and outside the satker.
    expect(wilayah.sedang_dipakai).toBeGreaterThan(satker.sedang_dipakai);
  });

  test('active-usage groups on the authoritative satker code', async ({ request }) => {
    const { accessToken } = await apiLogin(request, credsFor(userFor('operator_a')));
    const resp = await request.get(`${MON_API}/active-usage`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    expect(resp.ok(), `GET monitoring/active-usage (${resp.status()})`).toBeTruthy();
    const data = (await resp.json()).data;

    const buckets: { satker_code: string | null }[] = data.permits_by_satker ?? [];
    expect(buckets.length, 'one satker-bound caller, one bucket').toBe(1);
    expect(buckets[0].satker_code).toBe(userFor('operator_a').satkerCode);
  });
});

// ── Layer 2: the page renders the columns the stakeholder named ───────────
test.describe('Monitoring pemakaian BMN — rendered table', () => {
  test.use({ storageState: storageStatePath('operator_a') });

  test('the table shows satker, nama barang, NUP, pegawai and jangka waktu', async ({
    page,
    request,
  }) => {
    const rows = await monitoringRows(request, 'operator_a');
    expect(rows.length, 'need at least one visible permit to assert against').toBeGreaterThan(0);
    const row = rows[0];

    await page.goto(`${BASE}/pengelolaan/pemakaian/monitoring`, {
      waitUntil: 'domcontentloaded',
    });
    const table = page.getByTestId('tabel-pemakaian-bmn');
    await expect(table, 'monitoring table mounts').toBeVisible({ timeout: 30000 });

    // Structural locators, not getByText: getByText is a case-insensitive
    // SUBSTRING match, so it can pass while reading a different element and
    // send the diagnosis in the wrong direction.
    const cells = table.locator('tbody tr').first().locator('td');
    await expect(cells.nth(0), 'satker column').toContainText(row.satker_nama ?? row.satker_code!);
    await expect(cells.nth(1), 'nama barang column').toContainText(row.nama_barang);
    await expect(cells.nth(2), 'NUP column').toHaveText(row.nup);
    await expect(cells.nth(3), 'pegawai column').toContainText(row.pegawai_nama);
    await expect(cells.nth(4), 'jangka waktu column').toContainText(row.tanggal_selesai);
  });

  test('the table never renders another satker’s permit', async ({ page, request }) => {
    const foreign = await monitoringRows(request, 'operator_b');
    expect(foreign.length, 'need a foreign permit to look for').toBeGreaterThan(0);

    await page.goto(`${BASE}/pengelolaan/pemakaian/monitoring`, {
      waitUntil: 'domcontentloaded',
    });
    const table = page.getByTestId('tabel-pemakaian-bmn');
    await expect(table).toBeVisible({ timeout: 30000 });

    for (const row of foreign) {
      await expect(
        table.getByText(row.pegawai_nama, { exact: false }),
        `leaked ${row.pegawai_nama} from another satker`,
      ).toHaveCount(0);
    }
  });
});
