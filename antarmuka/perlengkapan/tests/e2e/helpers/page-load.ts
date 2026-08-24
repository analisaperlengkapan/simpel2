import { expect, type Page } from "@playwright/test";

/**
 * Statuses that mean the page issued a request it had no business issuing.
 *
 * `400`/`422` — the FE built a request the BE rejects: a missing required
 * param, a wrong enum value, a type the extractor cannot parse. Always an
 * FE↔BE contract bug.
 * `401` — every caller of {@link reachable} runs with an authenticated
 * `storageState`, so a rejected token is a bug, not a scenario.
 * `5xx` — never acceptable.
 *
 * `403` and `404` are deliberately NOT here: both are load-bearing signals
 * elsewhere in this suite (guards-rbac asserts 403; pages legitimately probe
 * for optional resources that may not exist yet), so flagging them from a
 * generic page-load check would fight the specs that assert them directly.
 */
function isBrokenStatus(status: number): boolean {
  return status === 400 || status === 401 || status === 422 || status >= 500;
}

const shell = (page: Page) => page.locator("header").getByText("SIMPEL").first();

/**
 * Navigate to `path` and assert the page is actually WORKING — not merely that
 * it mounted.
 *
 * The earlier form of this helper asserted two things: the authenticated shell
 * renders, and the URL is not the login page. Both hold for a page whose every
 * data request comes back 400 — the Leptos shell mounts, each resource lands in
 * its `Err` arm, and the user sees a page full of error cards. Three such
 * defects sat behind a green `laporan page is reachable` (a required
 * `pengajuan_id` the FE sent only when set, on two endpoints, plus an export
 * that sent the tab id `pegawai` where the BE matches `daftar`).
 *
 * So the check is derived from the traffic the page itself generates rather
 * than from a hand-maintained list of endpoints per page: whatever a page
 * calls, it has to succeed. A page added to this helper tomorrow gets the same
 * coverage without anyone remembering to extend anything — the failure mode
 * catalogued in `project_gate_scope_must_be_derived`.
 */
export async function reachable(page: Page, path: string): Promise<void> {
  const failures: string[] = [];
  const onResponse = (response: import("@playwright/test").Response) => {
    if (!response.url().includes("/api/")) return;
    if (!isBrokenStatus(response.status())) return;
    const u = new URL(response.url());
    failures.push(`${response.request().method()} ${u.pathname}${u.search} -> ${response.status()}`);
  };

  page.on("response", onResponse);
  try {
    await page.goto(path, { waitUntil: "domcontentloaded" });
    await expect(shell(page), `shell mounts on ${path}`).toBeVisible({ timeout: 20000 });
    expect(/login/i.test(page.url()), `must not redirect to login (${page.url()})`).toBeFalsy();
    // Give the page's own XHRs time to land. A page that never goes idle (poll
    // loops, a websocket that keeps HTTP alive) must not fail the test on that
    // account — we still assert on every response captured up to the deadline.
    await page.waitForLoadState("networkidle", { timeout: 10000 }).catch(() => {});
  } finally {
    page.off("response", onResponse);
  }

  expect(failures, `API calls failed while loading ${path}`).toEqual([]);
}
