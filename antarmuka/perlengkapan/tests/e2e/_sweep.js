// Exhaustive visual sweep: every static route from routes.rs, every role that
// may reach it. Routes are DERIVED from the router (see _routes.txt, generated
// by grepping `path!(...)` out of lib.rs) rather than hand-listed, so a page
// added tomorrow is swept tomorrow — the same reason the route-coverage gate
// walks the router instead of a maintained list.
//
// Parameterised routes (`:id`) are skipped here: a screenshot of a detail page
// needs a real id, which belongs with the workflow e2e that creates one.
const { chromium } = require('@playwright/test');
const fs = require('fs');
const path = require('path');
const LOCAL = 'http://127.0.0.1:8099';
const BASE = '/perlengkapan/simpel/v2';
const OUT = process.env.OUT_DIR;
const PASS = '199203142014031001';
// Derived from the seed, not hand-listed — the same mistake the route list
// made. Three of the SEVEN seeded users were being swept, so the admin's
// `/admin/*` pages, the satker validator's and the satker approver's views had
// never been photographed while the run reported "every role".
const USERS = (() => {
  const sql = fs.readFileSync(
    path.join(__dirname, '../../../../tests/fixtures/e2e/seed-multisatker.sql'), 'utf8');
  const seen = new Map();
  // `'20000000000000000N', '…@kejaksaan.go.id', …, 'E2E Operator Jakpus', …`
  for (const m of sql.matchAll(/'(2\d{17})'[^\n]*?'(E2E [^']+)'/g)) {
    const slug = m[2].replace(/^E2E /, '').toLowerCase()
      .replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    if (!seen.has(m[1])) seen.set(m[1], slug);
  }
  if (!seen.size) throw new Error('derived ZERO users from the seed — the fixture shape changed');
  return [...seen].map(([nip, slug]) => [slug, nip]);
})();

// Derived on every run from the router itself (`_routes.py` walks the
// ParentRoute nesting), so a route added tomorrow is swept tomorrow and a
// route that moves under a new parent is swept at its real URL.
const routes = require('child_process')
  .execSync(`python3 ${__dirname}/_routes.py`, { encoding: 'utf8' })
  .split('\n').map(r => r.trim())
  .filter(r => r && !r.includes(':') && r !== '/login');

(async () => {
  const browser = await chromium.launch();
  const findings = [];
  const skipped = [];
  for (const [role, nip] of USERS) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    const page = await ctx.newPage();
    const res = await ctx.request.post(`${LOCAL}/api/v1/auth/login`, { data: { username: nip, password: PASS } });
    if (!res.ok()) {
      // A role that cannot log in is a hole in the sweep, not a note in the
      // log. `pegawai-bandung` is seeded in the CI fixture but not on staging,
      // so deriving the user list from that fixture over-promises against this
      // target — the run must say so rather than quietly cover 7 of 8.
      findings.push(`${role} -> LOGIN-GAGAL ${res.status()} — peran ini TIDAK tersapu`);
      skipped.push(role);
      await ctx.close();
      continue;
    }
    const tok = (await res.json()).access_token;
    await page.addInitScript(t => {
      localStorage.setItem('access_token', t);
      localStorage.setItem('auth_token', t);
      localStorage.setItem('token', t);
    }, tok);

    for (const r of routes) {
      const errs = [];
      page.removeAllListeners('console');
      page.on('console', m => { if (m.type() === 'error') errs.push(m.text().slice(0, 120)); });
      const slug = (r === '/' ? 'root' : r.replace(/^\//, '').replace(/\//g, '_'));
      try {
        await page.goto(`${LOCAL}${BASE}${r}`, { waitUntil: 'domcontentloaded', timeout: 20000 });
        // Wait for the page's own traffic to settle, not a guessed constant.
        // The fixed 4500ms was shorter than `/bank-aset/dashboard` takes
        // against real staging data (measured 4.75s over 624 533 rows), so the
        // dashboard was screenshotted mid-skeleton — and a screenshot of a
        // loading placeholder proves nothing about the page.
        await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        await page.waitForTimeout(800);
        await page.screenshot({ path: `${OUT}/${role}__${slug}.png`, fullPage: true });
        // Scan the CONTENT, not the shell. The first widened run flagged 99
        // of 99 pages on `MODUL UTAMA` / `MANAJEMEN PERLENGKAPAN` from the
        // sidebar and footer — identical on every page, so anything the shell
        // contributes is noise by construction, and a detector that fires
        // everywhere is as useless as one that fires nowhere.
        const main = page.locator('main, [role="main"]').first();
        const scope = await main.count() ? main : page.locator('body');
        const body = await scope.innerText();
        // `innerText` returns text AFTER `text-transform: uppercase`, so every
        // styled table header ("NAMA BARANG", "STATUS") looked like a raw enum
        // and the capital-word rule flagged 87 pages. `textContent` gives the
        // author's own casing, which is what the rule is actually asking about.
        const source = await scope.evaluate(el => el.textContent || '');
        // Signals worth a human look, from defects this dashboard actually had.
        const flags = [];
        // Every pattern below was earned by a defect these screenshots actually
        // contained on 2026-08-29 — not by imagination.
        if (/[A-Z]{3,}_[A-Z]{3,}/.test(body)) flags.push('enum-mentah');
        // The rule above needs an underscore, so a single-word SCREAMING state
        // slipped past it: the kebutuhan list showed a bare `DRAFT` in every
        // row and the sweep called the page clean. Allow real acronyms.
        const ACRONYMS = new Set(['BMN','SIMPEL','NIP','SK','PDF','XLSX','XLS','CSV','QR',
                                  'RI','RKBMN','URGENT','HIGH','ID','NUP','API','UI','WIB',
                                  'SIMAN','KPKNL','BAST','TI','HR','NO','PT','CV']);
        const shouty = (source.match(/\b[A-Z]{3,}\b/g) || []).filter(w => !ACRONYMS.has(w));
        if (shouty.length) flags.push(`kata-kapital:${[...new Set(shouty)].slice(0,4).join(',')}`);
        // Leptos source rendering as page text — a split `>=` ends the tag early
        // and the rest of the attributes become a text node. Eleven components
        // shipped this; the button also silently stops working.
        if (/on:click|move \|_\||\.get\(\)|prop:disabled/.test(body)) flags.push('kode-bocor');
        // Raw storage formats reaching the reader.
        if (/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(\+00:00|Z)/.test(body)) flags.push('waktu-mentah-utc');
        if (/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/.test(body)) flags.push('uuid-di-layar');
        // Internal module slugs used as labels ("penghapusan_bmn: berpindah ...").
        if (/\b[a-z]+_[a-z]+(_[a-z]+)?\b/.test(body)) flags.push('slug-modul');
        if (/\bNaN\b|\bundefined\b|\bnull\b|\bUnknown\b/.test(body)) flags.push('nilai-kosong');
        if (/Tahun Anggaran 20(1|2[0-5])\b/.test(body)) flags.push('tahun-basi');
        // A 404 is a statement about the ROUTE, not about the page's content.
        // Reported as "halaman-nyaris-kosong" it read as a thin page; 24 of the
        // 99 screenshots were of the not-found page because the route list had
        // dropped the `/admin` prefix off seven nested routes, and nothing said
        // so. See `_routes.py`.
        // Match the app's own not-found copy, not a bare "404": an asset count
        // of 404 on a real page would otherwise be reported as a dead route.
        // Third variant of the same mistake: a page that deliberately shows
        // little is not a thin page. A 404 is a statement about the ROUTE and a
        // 403 about PERMISSION; both were being filed as observations about
        // content. All 21 `/admin/*` findings were the RoleGate denying three
        // non-admin users correctly.
        if (/Halaman Tidak Ditemukan/.test(body)) flags.push('RUTE-404');
        else if (/Akses Ditolak/.test(body)) flags.push('AKSES-DITOLAK');
        else if (body.trim().length < 200) flags.push('halaman-nyaris-kosong');
        if (errs.length) flags.push(`console:${errs.length}`);
        if (flags.length) findings.push(`${role} ${r} -> ${flags.join(', ')}`);
      } catch (e) {
        findings.push(`${role} ${r} -> GAGAL: ${String(e).slice(0, 80)}`);
      }
    }
    await ctx.close();
  }
  await browser.close();
  // Count what was CAPTURED. The previous line multiplied routes by roles and
  // printed 256 while 224 files existed, because one role never logged in —
  // a summary claiming coverage the run did not have.
  const swept = USERS.length - skipped.length;
  console.log(`\n${routes.length} rute x ${swept} peran tersapu = ${routes.length * swept} screenshot`);
  if (skipped.length) {
    console.log(`PERAN TIDAK TERSAPU (${skipped.length}): ${skipped.join(', ')}`);
  }
  console.log(findings.length ? `\nPERLU DIPERIKSA (${findings.length}):` : '\nTidak ada penanda otomatis.');
  findings.forEach(f => console.log('  ' + f));
})();
