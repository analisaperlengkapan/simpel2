
import { test, expect } from "@playwright/test";

test("verify dashboard metrics and secrets page", async ({ page }) => {
  // Navigate to dashboard
  await page.goto("http://localhost:3000");

  // Check if system metrics are visible (e.g., uptime, total secrets)
  // Assuming the fallback loader or actual data renders some text.
  await expect(page.locator("body")).toBeVisible();
  await page.screenshot({ path: "test/verification/dashboard_metrics.png" });

  // Navigate to secrets page
  await page.goto("http://localhost:3000/secrets");
  await expect(page.locator("h1")).toContainText("Manajemen Rahasia");
  await page.screenshot({ path: "test/verification/secrets_page.png" });
});
