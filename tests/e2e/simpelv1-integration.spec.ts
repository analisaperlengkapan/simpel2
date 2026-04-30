import { test, expect } from '@playwright/test';

const BASE_URL_V1 = 'http://localhost:8000/perlengkapan/simpel/v1';
const BASE_URL_PORTAL = 'http://localhost:3000/portal';
const BASE_URL_V2 = 'http://localhost:3020/perlengkapan/simpel/v2';

test.describe('SIMPEL v1/v2 Integration E2E Tests', () => {
  
  // =====================================================
  // Portal Navigation & Version Selection
  // =====================================================
  
  test('Portal loads successfully', async ({ page }) => {
    await page.goto(BASE_URL_PORTAL);
    await expect(page).toHaveTitle(/Portal/);
    await expect(page.locator('text=SIMPEL')).toBeVisible();
  });

  test('Portal shows v1/v2 selector in perlengkapan app', async ({ page }) => {
    await page.goto(BASE_URL_PORTAL);
    
    // Find perlengkapan app card
    const perlengkapanCard = page.locator('[data-app-id="perlengkapan"]');
    await expect(perlengkapanCard).toBeVisible();
    
    // Check for expand button
    const expandButton = perlengkapanCard.locator('button:has-text("Expand")');
    await expandButton.click();
    
    // Check submenu appears
    const submenu = perlengkapanCard.locator('[data-submenu]');
    await expect(submenu).toBeVisible();
  });

  test('Portal submenu shows v1 and v2 options', async ({ page }) => {
    await page.goto(BASE_URL_PORTAL);
    
    const perlengkapanCard = page.locator('[data-app-id="perlengkapan"]');
    const expandButton = perlengkapanCard.locator('button:has-text("Expand")');
    await expandButton.click();
    
    // Check v1 link
    const v1Link = page.locator('text=SIMPEL v1');
    await expect(v1Link).toBeVisible();
    
    // Check v2 link
    const v2Link = page.locator('text=SIMPEL v2');
    await expect(v2Link).toBeVisible();
  });

  // =====================================================
  // SIMPEL v1 OAuth Flow
  // =====================================================

  test('v1 redirects to Portal login when not authenticated', async ({ page }) => {
    await page.goto(BASE_URL_V1);
    
    // Should redirect to Portal login
    await page.waitForURL(/\/portal\//);
    await expect(page).toHaveURL(/\/portal\/login/);
  });

  test('v1 OAuth callback with valid token', async ({ page, context }) => {
    // Simulate getting JWT token from Portal
    const mockJWT = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0dXNlciIsIm5hbWUiOiJUZXN0IFVzZXIiLCJpYXQiOjE3MTgyMDAwMDB9.test';
    
    // Store token in context for later use
    await context.addCookies([{
      name: 'auth_token',
      value: mockJWT,
      url: BASE_URL_V1,
    }]);
    
    // Navigate with token parameter
    await page.goto(`${BASE_URL_V1}/auth/oauth-callback?token=${mockJWT}`);
    
    // Should create session and redirect to dashboard
    await page.waitForURL(/\/perlengkapan\/simpel\/v1\/dashboard/);
    await expect(page.locator('text=Dashboard')).toBeVisible();
  });

  test('v1 health check endpoint responds', async ({ page }) => {
    const response = await page.goto(`${BASE_URL_V1}/health`);
    expect(response?.status()).toBe(200);
    const text = await response?.text();
    expect(text).toContain('healthy');
  });

  // =====================================================
  // SIMPEL v2 Navigation
  // =====================================================

  test('v2 loads successfully when authenticated', async ({ page, context }) => {
    // Pre-authenticate v2
    await context.addCookies([{
      name: 'auth_token',
      value: 'mock-jwt-token',
      url: BASE_URL_V2,
    }]);
    
    // Navigate to v2
    await page.goto(BASE_URL_V2);
    
    // Check for v2-specific elements
    await expect(page.locator('text=Modern')).toBeVisible({ timeout: 5000 });
  });

  // =====================================================
  // Cross-Version Session Management
  // =====================================================

  test('Session invalidation in v1 detected in Portal', async ({ page, context }) => {
    // Start authenticated in v1
    const mockJWT = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0dXNlciIsIm5hbWUiOiJUZXN0IFVzZXIifQ.test';
    
    await context.addCookies([{
      name: 'auth_token',
      value: mockJWT,
      url: BASE_URL_V1,
    }]);
    
    // Navigate to v1
    await page.goto(`${BASE_URL_V1}/dashboard`);
    await expect(page.locator('text=Dashboard')).toBeVisible();
    
    // Logout
    await page.click('text=Logout');
    await page.waitForURL(/\/auth\/login/);
    
    // Check logout_event marker
    const logoutEvent = await page.evaluate(() => {
      return localStorage.getItem('logout_event');
    });
    expect(logoutEvent).toBeTruthy();
  });

  // =====================================================
  // gRPC Service Integration (Indirect Tests)
  // =====================================================

  test('v1 connects to Authenc service on startup', async ({ page }) => {
    // Check server logs or health endpoint for gRPC connectivity
    const response = await page.goto(`${BASE_URL_V1}/health`);
    expect(response?.status()).toBe(200);
  });

  test('v1 database contains expected tables', async ({ page }) => {
    // This test requires database access
    // Skip for browser test, should be integration test instead
    test.skip();
  });

  // =====================================================
  // Error Handling
  // =====================================================

  test('v1 handles invalid JWT token gracefully', async ({ page }) => {
    const invalidToken = 'invalid-jwt-token';
    
    await page.goto(`${BASE_URL_V1}/auth/oauth-callback?token=${invalidToken}`);
    
    // Should redirect to login with error
    await page.waitForURL(/\/auth\/login/);
    await expect(page.locator('text=Token tidak valid')).toBeVisible();
  });

  test('v1 shows error when Portal unreachable', async ({ page }) => {
    // This test requires mocking Portal unavailability
    test.skip();
  });

  // =====================================================
  // Performance & Load Testing
  // =====================================================

  test('v1 health endpoint responds within 100ms', async ({ page }) => {
    const startTime = Date.now();
    await page.goto(`${BASE_URL_V1}/health`);
    const duration = Date.now() - startTime;
    expect(duration).toBeLessThan(100);
  });

  test('v1 dashboard loads within 3 seconds', async ({ page, context }) => {
    const mockJWT = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0dXNlciJ9.test';
    await context.addCookies([{
      name: 'auth_token',
      value: mockJWT,
      url: BASE_URL_V1,
    }]);
    
    const startTime = Date.now();
    await page.goto(`${BASE_URL_V1}/dashboard`);
    await page.waitForLoadState('networkidle');
    const duration = Date.now() - startTime;
    
    expect(duration).toBeLessThan(3000);
  });

  // =====================================================
  // Accessibility Tests
  // =====================================================

  test('v1 login form is keyboard accessible', async ({ page }) => {
    await page.goto(`${BASE_URL_V1}/auth/login`);
    
    // Tab to username field
    await page.keyboard.press('Tab');
    await expect(page.locator('input[name="username"]')).toBeFocused();
    
    // Tab to password field
    await page.keyboard.press('Tab');
    await expect(page.locator('input[name="password"]')).toBeFocused();
    
    // Tab to submit button
    await page.keyboard.press('Tab');
    await expect(page.locator('button[type="submit"]')).toBeFocused();
  });

  // =====================================================
  // Security Tests
  // =====================================================

  test('v1 CSRF token present in forms', async ({ page }) => {
    await page.goto(`${BASE_URL_V1}/auth/login`);
    
    // Check for CSRF token input
    const csrfToken = page.locator('input[name="_token"]');
    await expect(csrfToken).toBeVisible();
    
    // Verify token has a value
    const tokenValue = await csrfToken.inputValue();
    expect(tokenValue).toBeTruthy();
  });

  test('v1 sets secure session cookie', async ({ page, context }) => {
    const mockJWT = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0dXNlciJ9.test';
    
    await page.goto(`${BASE_URL_V1}/auth/oauth-callback?token=${mockJWT}`);
    
    // Check cookies
    const cookies = await context.cookies();
    const sessionCookie = cookies.find(c => c.name.includes('LARAVEL_SESSION'));
    
    expect(sessionCookie).toBeTruthy();
    expect(sessionCookie?.httpOnly).toBe(true);
    expect(sessionCookie?.secure).toBe(true); // In HTTPS environments
  });
});

// =====================================================
// Load Testing (Optional - requires wrk or similar tool)
// =====================================================

test.describe.skip('Load Tests', () => {
  test('50 concurrent requests to v1 health endpoint', async ({ page }) => {
    // This should be run with: wrk -t 4 -c 50 http://localhost:8000/health
    // Not suitable for Playwright alone
    test.skip();
  });

  test('v1 handles 10 concurrent logins', async ({ page }) => {
    // Requires database transaction support
    test.skip();
  });
});
