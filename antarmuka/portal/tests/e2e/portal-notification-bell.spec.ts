/**
 * Portal — the notification bell.
 *
 * Real auth (UI login, real captcha), like portal-core-workflow. The notifikasi service is NOT part
 * of this stack, so the inbox is either honestly unavailable (the real state here) or, where a test
 * needs rows, answered by a stub for the list/read calls only.
 *
 * WHAT THIS GUARDS
 *   The bell never opened. Its "close when clicking outside" listener was installed from an Effect
 *   that ran on the very click that opened the panel, so the same click closed it again a
 *   millisecond later, before anything was painted. A person saw a bell with a red badge that did
 *   nothing. The first test is that regression: the panel must be open, and STILL open a moment
 *   later. Opening a row also did nothing but mark it read; it now opens the page it is about,
 *   on the Perlengkapan mount point (the rows hold a path inside the module, '/pakaian-dinas/...').
 */
import { test, expect, type Page, type Route } from '@playwright/test';
import {
  OPERATOR_USER,
  loginViaUi,
  setupAllProxies,
  skipUnlessCaptchaDebug,
} from './helpers/real-auth';

const bell = (page: Page) => page.getByRole('button', { name: /^Notifikasi/ });
const panel = (page: Page) => page.getByRole('dialog');

const PENGAJUAN = '11111111-1111-4111-8111-111111111111';
const ROWS = [
  {
    id: 'a0000000-0000-4000-8000-000000000001',
    title: 'Pakaian dinas menunggu pemeriksaan Anda',
    message: 'KEJAKSAAN NEGERI JAKARTA SELATAN mengajukan ukuran pakaian dinas.',
    priority: 'normal',
    category: 'info',
    // What real producers write: a path inside the module, without the mount point.
    action_url: `/pakaian-dinas/pengajuan/${PENGAJUAN}/satker`,
    read: false,
    created_at: '2026-11-28T04:25:27.248854+00:00',
  },
  {
    id: 'a0000000-0000-4000-8000-000000000002',
    title: 'Pemberitahuan tanpa tujuan',
    message: 'Baris ini tidak membawa tautan.',
    priority: 'normal',
    category: 'info',
    action_url: null,
    read: false,
    created_at: '2026-11-28T04:20:00.000000+00:00',
  },
];

/** Answer the inbox list and the read calls; record the reads. The rest of /api keeps its real proxy. */
async function stubInbox(page: Page, rows: unknown[] = ROWS): Promise<string[]> {
  const read: string[] = [];
  await page.route(/\/api\/v1\/perlengkapan\/notifikasi\?/, async (route: Route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true, data: rows, message: 'ok' }),
    });
  });
  await page.route(/\/api\/v1\/perlengkapan\/notifikasi\/[^/]+\/read$/, async (route: Route) => {
    read.push(route.request().url().split('/').slice(-2)[0]);
    await route.fulfill({ status: 200, contentType: 'application/json', body: '{"success":true}' });
  });
  // The Perlengkapan app is not part of this stack either: a navigation to its mount point only
  // needs to be observed, not served.
  await page.route('**/perlengkapan/simpel/**', async (route: Route) => {
    if (route.request().resourceType() === 'document') {
      await route.fulfill({
        status: 200,
        contentType: 'text/html',
        body: '<!doctype html><title>perlengkapan</title><p>perlengkapan</p>',
      });
    } else {
      await route.fallback();
    }
  });
  return read;
}

test.describe('Portal — notification bell', () => {
  let getCaptchaAnswer: () => string | null;

  test.beforeEach(async ({ page }) => {
    ({ getCaptchaAnswer } = await setupAllProxies(page));
    await skipUnlessCaptchaDebug(page.request);
  });

  test('clicking the bell opens the panel, and it stays open', async ({ page }) => {
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await expect(page).toHaveURL(/\/portal\/dashboard/);

    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    await expect(panel(page).getByRole('heading', { name: 'Notifikasi' })).toBeVisible();
    await expect(bell(page)).toHaveAttribute('aria-expanded', 'true');

    // Anchored to the bell, not stranded in the page corner (`Popover` once never positioned its panel).
    const anchor = (await bell(page).boundingBox())!;
    const box = (await panel(page).boundingBox())!;
    expect(box.y, 'the panel starts below the bell, not at the top of the page').toBeGreaterThanOrEqual(
      anchor.y + anchor.height,
    );
    expect(
      Math.abs(box.x + box.width - (anchor.x + anchor.width)),
      "the panel's right edge lines up with the bell's",
    ).toBeLessThanOrEqual(40);

    // The regression: opened and closed again by the same click, a millisecond later.
    await page.waitForTimeout(500);
    await expect(panel(page)).toBeVisible();

    // Not a fabricated list: this stack has no notifikasi service, so the panel says so.
    await expect(panel(page).getByText(/Notifikasi tidak tersedia|Tidak ada notifikasi/)).toBeVisible({
      timeout: 15000,
    });
  });

  test('Escape, a click outside, and the bell itself close the panel', async ({ page }) => {
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(panel(page)).toHaveCount(0);
    await expect(bell(page)).toHaveAttribute('aria-expanded', 'false');

    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    // An empty spot of the sidebar: outside the panel and its bell, and not a control.
    await page.mouse.click(5, 650);
    await expect(panel(page)).toHaveCount(0);

    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    await bell(page).click();
    await expect(panel(page)).toHaveCount(0);
  });

  test('the panel lists the inbox rows and the badge counts the unread ones', async ({ page }) => {
    await stubInbox(page);
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);

    await expect(bell(page)).toHaveAttribute('aria-label', 'Notifikasi, 2 belum dibaca', {
      timeout: 15000,
    });
    await bell(page).click();
    for (const row of ROWS) {
      await expect(panel(page).getByText(row.title)).toBeVisible();
    }
  });

  test('opening a row with a destination reads it and goes to that page on the Perlengkapan mount point', async ({
    page,
  }) => {
    const read = await stubInbox(page);
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await expect(bell(page)).toHaveAttribute('aria-label', /belum dibaca/, { timeout: 15000 });

    await bell(page).click();
    await panel(page).getByRole('link', { name: /Pakaian dinas menunggu pemeriksaan Anda/ }).click();

    // The host root `/pakaian-dinas/...` is served by nothing: this is where the bare path used to lead.
    await expect(page).toHaveURL(
      new RegExp(`/perlengkapan/simpel/pakaian-dinas/pengajuan/${PENGAJUAN}/satker$`),
    );
    expect(read, 'the row is marked read on the server').toContain(ROWS[0].id);
  });

  test('a row without a destination only marks it read', async ({ page }) => {
    const read = await stubInbox(page);
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await expect(bell(page)).toHaveAttribute('aria-label', 'Notifikasi, 2 belum dibaca', {
      timeout: 15000,
    });

    await bell(page).click();
    const before = page.url();
    await panel(page).getByRole('button', { name: /Pemberitahuan tanpa tujuan/ }).click();

    await expect(panel(page)).toHaveCount(0);
    expect(page.url(), 'a row without a destination must not navigate').toBe(before);
    await expect.poll(() => read).toContain(ROWS[1].id);
    await expect(bell(page)).toHaveAttribute('aria-label', 'Notifikasi, 1 belum dibaca');
  });

  test('on a short screen the way to the full list stays reachable', async ({ page }) => {
    // A full panel (more rows than fit) on a phone-sized screen: the list between the header and the
    // footer must shrink and scroll, not push the footer link off the bottom.
    const many = Array.from({ length: 12 }, (_, i) => ({
      ...ROWS[0],
      id: `b0000000-0000-4000-8000-${String(i).padStart(12, '0')}`,
      title: `Notifikasi nomor ${i + 1}`,
    }));
    await stubInbox(page, many);
    await page.setViewportSize({ width: 1280, height: 480 });
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await expect(bell(page)).toHaveAttribute('aria-label', 'Notifikasi, 12 belum dibaca', {
      timeout: 15000,
    });

    await bell(page).click();
    const all = panel(page).getByRole('link', { name: 'Lihat semua notifikasi' });
    await expect(all).toBeInViewport();
    await all.click();
    await expect(page).toHaveURL(/\/portal\/notifications$/);
  });

  test("the full list page offers 'Buka' on the Perlengkapan mount point", async ({ page }) => {
    await stubInbox(page);
    await loginViaUi(page, OPERATOR_USER, getCaptchaAnswer);
    await bell(page).click();
    await panel(page).getByRole('link', { name: 'Lihat semua notifikasi' }).click();
    await expect(page).toHaveURL(/\/portal\/notifications$/);

    await expect(page.getByRole('link', { name: 'Buka' })).toHaveAttribute(
      'href',
      `/perlengkapan/simpel/pakaian-dinas/pengajuan/${PENGAJUAN}/satker`,
      { timeout: 15000 },
    );
    // Only the row that points somewhere gets one.
    await expect(page.getByRole('link', { name: 'Buka' })).toHaveCount(1);
  });
});
