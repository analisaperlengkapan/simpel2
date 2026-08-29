// Targeted after-shots for named routes, one role. Used to honour the
// before/after obligation without re-running the whole 256-shot sweep.
const { chromium } = require('@playwright/test');
const LOCAL = 'http://127.0.0.1:8099';
const BASE = '/perlengkapan/simpel/v2';
const OUT = process.env.OUT_DIR;
const ROUTES = (process.env.ROUTES_CSV || '').split(',').filter(Boolean);
const NIP = process.env.NIP || '200000000000000004';

(async () => {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await ctx.newPage();
  const res = await ctx.request.post(`${LOCAL}/api/v1/auth/login`,
    { data: { username: NIP, password: '199203142014031001' } });
  if (!res.ok()) { console.log(`login ${res.status()}`); process.exit(1); }
  const tok = (await res.json()).access_token;
  await page.addInitScript(t => {
    localStorage.setItem('access_token', t);
    localStorage.setItem('auth_token', t);
    localStorage.setItem('token', t);
  }, tok);
  for (const r of ROUTES) {
    const slug = r.replace(/^\//, '').replace(/\//g, '_');
    await page.goto(`${LOCAL}${BASE}${r}`, { waitUntil: 'domcontentloaded', timeout: 20000 });
    await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
    await page.waitForTimeout(800);
    await page.screenshot({ path: `${OUT}/${slug}.png`, fullPage: true });
    console.log(`shot ${r}`);
  }
  await browser.close();
})();
