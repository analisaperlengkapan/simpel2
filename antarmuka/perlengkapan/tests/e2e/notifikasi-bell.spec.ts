/**
 * Notifikasi — the bell in the header.
 *
 * Real-auth, per-role storageState; asserts the real UI AND the backend behind it.
 * Preconditions: seed-perlengkapan-workflow.sql section 9 (operator_a's inbox):
 *   N1 "E2E Notifikasi Satu"    unread, action_url '/perlengkapan/simpel/pengelolaan/penghapusan' (full form)
 *   N2 "E2E Notifikasi Dua"     unread, action_url '/pakaian-dinas/pengajuan' (the form real producers write)
 *   N3 "E2E Notifikasi Terbaca" read,   no action_url
 *   N4 belongs to operator_b and must never appear for operator_a.
 *
 * WHAT THIS GUARDS
 *   The bell used to be a link that jumped to the inbox page: clicking it showed no list. And the
 *   inbox's "Buka" link took `action_url` as a bare href; real rows hold a path inside the module
 *   ('/pakaian-dinas/...'), so it left the app for the host root: a blank 404. The old fixture held
 *   only full paths, which is why no test ever saw it.
 *
 * Read state is NOT used as a starting point. "Tandai semua dibaca" in bankaset-analitik-workflow
 * legitimately drains operator_a's inbox, and the projects run in parallel; rows are found by
 * title, and the read-state check below is conditional on what the row was beforehand.
 */
import { test, expect, type APIRequestContext, type Page } from "@playwright/test";
import { PERLENGKAPAN_API_URL, storageStatePath, tokenFor } from "./helpers/real-auth";

const BASE = "/perlengkapan/simpel";
const API = `${PERLENGKAPAN_API_URL}/api/v1/perlengkapan`;
const N2 = "f1000000-0000-4f00-8f00-0000000c0002";

const shell = (page: Page) => page.locator("header").getByText("SIMPEL").first();
const bell = (page: Page) => page.locator("header").getByRole("button", { name: /^Notifikasi/ });
const panel = (page: Page) => page.getByRole("dialog");

const isRead = async (request: APIRequestContext, id: string): Promise<boolean | undefined> => {
  const token = await tokenFor(request, "operator_a");
  const res = await request.get(`${API}/notifikasi?limit=100&offset=0&unread_only=false`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(res.ok(), `GET /notifikasi -> ${res.status()}`).toBeTruthy();
  const rows: Array<{ id: string; read: boolean }> = (await res.json()).data ?? [];
  return rows.find((r) => r.id === id)?.read;
};

const openBell = async (page: Page) => {
  await page.goto(`${BASE}/dashboard`);
  await expect(shell(page)).toBeVisible({ timeout: 20000 });
  await bell(page).click();
  await expect(panel(page)).toBeVisible({ timeout: 20000 });
};

test.describe("Notifikasi — the bell lists notifications", () => {
  test.use({ storageState: storageStatePath("operator_a") });

  test("clicking the bell opens a list of this user's notifications, and it stays open", async ({
    page,
  }) => {
    await openBell(page);
    for (const title of ["E2E Notifikasi Satu", "E2E Notifikasi Dua", "E2E Notifikasi Terbaca"]) {
      await expect(panel(page).getByText(title)).toBeVisible({ timeout: 20000 });
    }
    // Per-user isolation, same as the inbox page.
    await expect(panel(page).getByText("E2E Notifikasi Operator B")).toHaveCount(0);
    await expect(bell(page)).toHaveAttribute("aria-expanded", "true");

    // Anchored to the bell, not stranded in the page corner. `Popover` once never positioned its
    // panel (the element it was given to position was an empty copy), so every popover sat at
    // `top: 0; left: 0`, over the header and the sidebar.
    const anchor = (await bell(page).boundingBox())!;
    const box = (await panel(page).boundingBox())!;
    expect(box.y, "the panel starts below the bell, not at the top of the page").toBeGreaterThanOrEqual(
      anchor.y + anchor.height,
    );
    expect(
      Math.abs(box.x + box.width - (anchor.x + anchor.width)),
      "the panel's right edge lines up with the bell's",
    ).toBeLessThanOrEqual(40);

    // The bug this replaces: the panel opened and was closed again by the same click, a
    // millisecond later. A moment on, it must still be there.
    await page.waitForTimeout(500);
    await expect(panel(page)).toBeVisible();
  });

  test("Escape, a click outside, and the bell itself close the panel", async ({ page }) => {
    await openBell(page);
    await page.keyboard.press("Escape");
    await expect(panel(page)).toHaveCount(0);
    await expect(bell(page)).toHaveAttribute("aria-expanded", "false");

    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    // `header` matches every card header on the dashboard too; the app bar is the page banner.
    await page.getByRole("banner").click({ position: { x: 640, y: 28 } });
    await expect(panel(page)).toHaveCount(0);

    await bell(page).click();
    await expect(panel(page)).toBeVisible();
    await bell(page).click();
    await expect(panel(page)).toHaveCount(0);
  });

  test("a notification that carries the module-relative path opens that page, not a 404", async ({
    page,
    request,
  }) => {
    const wasUnread = (await isRead(request, N2)) === false;
    await openBell(page);

    // Only a row that was unread triggers the PATCH; wait for it only then.
    const patched = wasUnread
      ? page.waitForRequest(
          (r) => r.method() === "PATCH" && r.url().endsWith(`/notifikasi/${N2}/read`),
        )
      : null;
    await panel(page).getByRole("link", { name: /E2E Notifikasi Dua/ }).click();

    // On the app's own mount point. The bare href went to `/pakaian-dinas/pengajuan` on the host root.
    await expect(page).toHaveURL(new RegExp(`${BASE}/pakaian-dinas/pengajuan$`));
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    await expect(
      page.getByRole("heading", { name: "Pengajuan Pakaian Dinas", exact: true }),
    ).toBeVisible({
      timeout: 20000,
    });
    await expect(panel(page)).toHaveCount(0);

    if (patched) await patched;
    await expect.poll(() => isRead(request, N2), { timeout: 20000 }).toBe(true);
  });

  test("a notification that already carries the full path is not given the mount point twice", async ({
    page,
  }) => {
    await openBell(page);
    await panel(page).getByRole("link", { name: /E2E Notifikasi Satu/ }).click();
    await expect(page).toHaveURL(new RegExp(`${BASE}/pengelolaan/penghapusan$`));
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
  });

  test("a notification with no destination is a button that only marks it read", async ({
    page,
  }) => {
    await openBell(page);
    const row = panel(page).getByRole("button", { name: /E2E Notifikasi Terbaca/ });
    await expect(row).toBeVisible({ timeout: 20000 });
    await expect(panel(page).getByRole("link", { name: /E2E Notifikasi Terbaca/ })).toHaveCount(0);

    const before = page.url();
    await row.click();
    await expect(panel(page)).toHaveCount(0);
    expect(page.url(), "opening a row without a destination must not navigate").toBe(before);
  });

  test("on a short screen the way to the full inbox stays reachable", async ({ page }) => {
    // The panel is limited to what the screen leaves, and the list between its header and its footer
    // shrinks and scrolls. With a fixed list height the footer link was pushed off the bottom of a
    // phone-sized screen, and scrolling the list never brought it back.
    await page.setViewportSize({ width: 1280, height: 480 });
    await openBell(page);
    const all = panel(page).getByRole("link", { name: "Lihat semua notifikasi" });
    await expect(all).toBeInViewport();
    await all.click();
    await expect(page).toHaveURL(new RegExp(`${BASE}/notifikasi$`));
  });

  test("'Lihat semua notifikasi' leads to the inbox page", async ({ page }) => {
    await openBell(page);
    await panel(page).getByRole("link", { name: "Lihat semua notifikasi" }).click();
    await expect(page).toHaveURL(new RegExp(`${BASE}/notifikasi$`));
    await expect(page.getByText("E2E Notifikasi Satu").first()).toBeVisible({ timeout: 20000 });
  });

  test("the inbox's 'Buka' opens the same page for a module-relative path", async ({ page }) => {
    await page.goto(`${BASE}/notifikasi`);
    await expect(shell(page)).toBeVisible({ timeout: 20000 });
    const row = page.locator("li", { hasText: "E2E Notifikasi Dua" });
    await expect(row).toBeVisible({ timeout: 20000 });
    await row.getByRole("link", { name: "Buka" }).click();
    await expect(page).toHaveURL(new RegExp(`${BASE}/pakaian-dinas/pengajuan$`));
    await expect(
      page.getByRole("heading", { name: "Pengajuan Pakaian Dinas", exact: true }),
    ).toBeVisible({
      timeout: 20000,
    });
  });
});
