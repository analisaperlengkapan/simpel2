/**
 * Workflow-driver helpers shared by the e2e specs that push a record through a
 * real multi-role approval chain.
 *
 * Why `clickAction` exists: these specs used to `click()` a workflow button and
 * then immediately `page.reload()`. The reload tears down the in-flight fetch,
 * the browser sends RST_STREAM, and nginx logs the POST as `499` (client closed
 * request) — the test cancelled the very mutation it was about to assert on.
 * Whether the backend had finished by then came down to how loaded the CI host
 * was, which is what made the resulting failures look like environment flake.
 *
 * Observed for real on 2026-08-24 in one run: `approver-satker-action` and
 * `keputusan-pusat` both logged 499, and the pemakaian permit was left APPROVED
 * instead of ACTIVE because the auto-activation that follows the approval
 * commit never got to run.
 *
 * The service side is being made cancel-safe separately (a dropped request must
 * not leave torn state); this helper fixes the test's own half of it, so a spec
 * asserts on a request that actually completed.
 */
import { expect, type Locator, type Page, type Response } from "@playwright/test";

/**
 * Click a workflow action button and wait for the mutation it fires to come
 * back, so callers can reload/assert against settled state.
 *
 * `urlFragment` matches against the request URL; pass the endpoint suffix
 * (e.g. `"approver-satker-action"`). Only non-GET responses are considered, so
 * a page's background reads cannot satisfy the wait by accident.
 */
export async function clickAction(
  page: Page,
  button: Locator,
  urlFragment: string,
  opts: { timeout?: number } = {},
): Promise<Response> {
  const timeout = opts.timeout ?? 30000;
  // Arm the waiter BEFORE the click: the response can land before an
  // afterwards-registered listener is attached.
  const pending = page.waitForResponse(
    (r) => r.url().includes(urlFragment) && r.request().method() !== "GET",
    { timeout },
  );
  await button.click();
  const response = await pending;
  expect(
    response.ok(),
    `${response.request().method()} ${urlFragment} -> ${response.status()}`,
  ).toBeTruthy();
  return response;
}
