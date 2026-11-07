/**
 * The admin Audit Log page must show what the system audits.
 *
 * The page used to call `/admin/audit`, which read `workflow_transitions` JOIN
 * `workflow_instances` — two tables no code path ever INSERTs into — so it said
 * "Menampilkan 0 entri" in every environment while `perlengkapan.audit_log`
 * held every real event. Nothing failed: guards-rbac and nav-access only ask who
 * may OPEN the page (and the former even claimed the page "renders real data"),
 * so a page that was empty by construction stayed green.
 *
 * This spec files its own audited event through the API, then asserts the page
 * lists it, that every dropdown option is an option that really selects rows
 * (the old lists named `bank_aset`, `login`, ... that nothing ever logs), and
 * that the date filter filters (a bare `YYYY-MM-DD` was once ignored server-side
 * while the page claimed a range was applied).
 *
 * Self-contained: it creates its own ticket, so it does not depend on the order
 * other specs ran in.
 */
import { test, expect, type Page } from '@playwright/test';
import { storageStatePath, tokenFor, PERLENGKAPAN_API_URL } from './helpers/real-auth';

const BASE = '/perlengkapan/simpel/v2';
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;
const PAGE = `${BASE}/admin/audit`;

const total = (page: Page) => page.getByTestId('audit-total');
const rows = (page: Page) => page.getByTestId('audit-row');

/** Option values of a dropdown, minus the "Semua" (all) entry. */
async function optionsOf(page: Page, label: string): Promise<string[]> {
  const values = await page
    .getByLabel(label, { exact: true })
    .locator('option')
    .evaluateAll((os) => os.map((o) => (o as HTMLOptionElement).value));
  return values.filter((v) => v !== '');
}

/** Today as the BROWSER's calendar sees it — the zone the page builds day bounds in. */
async function browserDate(page: Page, plusDays = 0): Promise<string> {
  return page.evaluate((d) => {
    const t = new Date();
    t.setDate(t.getDate() + d);
    const p = (n: number) => String(n).padStart(2, '0');
    return `${t.getFullYear()}-${p(t.getMonth() + 1)}-${p(t.getDate())}`;
  }, plusDays);
}

test.describe('Admin audit log shows the real trail', () => {
  test.use({ storageState: storageStatePath('admin') });

  let subject: string;
  let ticketId: string;

  test.beforeAll(async ({ request }) => {
    // Any audited action will do; a helpdesk ticket writes module=bantuan,
    // resource_type=tiket, action=create, message="subject=<subject>".
    subject = `E2E audit ${Date.now()}`;
    const token = await tokenFor(request, 'operator_b');
    const created = await request.post(`${API}/bantuan/tiket`, {
      headers: { Authorization: `Bearer ${token}` },
      data: { subject, description: 'Peristiwa untuk uji halaman audit.', priority: 'low' },
    });
    expect(created.ok(), await created.text()).toBeTruthy();
    ticketId = (await created.json()).data.id as string;
  });

  test('an audited event is listed, with who/what/which record', async ({ page }) => {
    await page.goto(PAGE, { waitUntil: 'domcontentloaded' });
    await expect(page.getByText('Aktivitas').first()).toBeVisible({ timeout: 20000 });

    // The headline defect: the count was 0 whatever had happened.
    await expect(total(page)).toHaveText(/^[1-9]\d*$/, { timeout: 20000 });

    // Find OUR event by its unique subject (free-text search = message ILIKE).
    await page.getByPlaceholder(/ringkasan/i).fill(subject);
    await page.getByRole('button', { name: 'Terapkan' }).click();
    const row = page.locator(`[data-testid="audit-row"][data-resource-id="${ticketId}"]`);
    await expect(row).toBeVisible({ timeout: 20000 });
    await expect(row).toHaveAttribute('data-module', 'bantuan');
    await expect(row).toHaveAttribute('data-entity', 'tiket');
    await expect(row).toHaveAttribute('data-action', 'create');
    await expect(row).toContainText('tiket.create');
    await expect(row).toContainText(subject);
    await expect(row).toContainText('Berhasil');
  });

  test('dropdowns offer exactly what the trail holds, and each option selects rows', async ({
    page,
  }) => {
    await page.goto(PAGE, { waitUntil: 'domcontentloaded' });
    await expect(total(page)).toHaveText(/^[1-9]\d*$/, { timeout: 20000 });

    // The vocabulary comes from the data: what we just wrote is offered …
    await expect.poll(() => optionsOf(page, 'Modul'), { timeout: 20000 }).toContain('bantuan');
    expect(await optionsOf(page, 'Jenis Entitas')).toContain('tiket');
    expect(await optionsOf(page, 'Aksi')).toContain('create');
    // … and the values the old hand-written lists invented are not.
    for (const [label, invented] of [
      ['Jenis Entitas', ['bank_aset', 'master_data', 'user', 'workflow']],
      ['Aksi', ['login', 'logout', 'delete']],
    ] as const) {
      const offered = await optionsOf(page, label);
      for (const v of invented) {
        expect(offered, `${label} must not offer "${v}", nothing is logged under it`).not.toContain(v);
      }
    }

    // Each option, once chosen, narrows the table to rows that carry that value —
    // and to AT LEAST one row. `poll` waits out the previous answer, so stale rows
    // from the earlier filter read as "mixed" until the new answer lands, and an
    // option that matches nothing stays "empty" and fails.
    for (const [label, attr] of [
      ['Modul', 'data-module'],
      ['Jenis Entitas', 'data-entity'],
      ['Aksi', 'data-action'],
    ] as const) {
      for (const value of await optionsOf(page, label)) {
        await page.getByLabel(label, { exact: true }).selectOption(value);
        await expect
          .poll(
            async () => {
              const seen = await rows(page).evaluateAll(
                (els, a) => els.map((e) => e.getAttribute(a)),
                attr,
              );
              if (seen.length === 0) return 'empty';
              return seen.every((s) => s === value) ? 'ok' : 'mixed';
            },
            { message: `${label}=${value} must list rows that all carry it`, timeout: 20000 },
          )
          .toBe('ok');
      }
      await page.getByLabel(label, { exact: true }).selectOption('');
    }
  });

  test('the date filter filters: a future day is empty, today holds our event', async ({
    page,
  }) => {
    await page.goto(PAGE, { waitUntil: 'domcontentloaded' });
    await expect(total(page)).toHaveText(/^[1-9]\d*$/, { timeout: 20000 });

    const today = await browserDate(page);
    const tomorrow = await browserDate(page, 1);

    // From tomorrow on: nothing has happened yet. (A bare date the server ignores
    // would leave the full list standing here.)
    await page.getByLabel('Dari tanggal').fill(tomorrow);
    await page.getByRole('button', { name: 'Terapkan' }).click();
    await expect(total(page)).toHaveText('0', { timeout: 20000 });
    await expect(page.getByText('Tidak ada aktivitas')).toBeVisible();

    // Today only: our event is inside the whole-day window.
    await page.getByLabel('Dari tanggal').fill(today);
    await page.getByLabel('Sampai tanggal').fill(today);
    await page.getByPlaceholder(/ringkasan/i).fill(subject);
    await page.getByRole('button', { name: 'Terapkan' }).click();
    await expect(
      page.locator(`[data-testid="audit-row"][data-resource-id="${ticketId}"]`),
    ).toBeVisible({ timeout: 20000 });

    // Reset puts everything back.
    await page.getByRole('button', { name: 'Reset' }).click();
    await expect(total(page)).toHaveText(/^[1-9]\d*$/, { timeout: 20000 });
  });
});
