#!/usr/bin/env node
// @ts-check
/**
 * Route-coverage gate (F-E2E — "komprehensif, tanpa terkecuali").
 *
 * Every typed *navigable* route declared in each frontend's `routes.rs` must be
 * visited by at least one Playwright spec. This is the mechanical enforcement of
 * "semua halaman diuji": you cannot add a page without also adding an e2e that
 * opens it (or explicitly recording it as known, tracked debt).
 *
 * Pure static analysis — parses Rust source + greps spec text. No FE build, no
 * browser, no running stack; safe to run anywhere Node is present (incl. the
 * build host that OOMs on `trunk build`).
 *
 * Route set  = `pub const NAME: &str = "/<base>/...";` in <fe>/src/routes.rs,
 *              value starts with the FE `base`, minus:
 *                - `*_LEGACY` consts (deprecated aliases),
 *                - values whose suffix is in `excludeSuffix` (e.g. `/login`),
 *                - the bare base / root redirect.
 * Covered    = a normalized route path that appears as a substring in any
 *              `*.spec.ts` under the FE `specsDir`.
 * Debt       = `allowUncovered`: routes knowingly not-yet-covered. Shrinks to
 *              `[]` per FE as domains land; `blocking:true` then requires it empty.
 *
 * FAILS (exit 1) when, for any FE:
 *   - a route is uncovered AND not in `allowUncovered`  (new/forgotten page), or
 *   - `allowUncovered` has a STALE entry (now covered, or not a real route), or
 *   - `blocking:true` and `allowUncovered` is non-empty  (debt must be zero).
 *
 * Usage:  node tests/e2e/route-coverage.mjs [--json] [--fe perlengkapan|portal]
 */
import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

const args = process.argv.slice(2);
const asJson = args.includes('--json');
const feFilter = args.includes('--fe') ? args[args.indexOf('--fe') + 1] : null;
const configPath = args.includes('--config')
  ? args[args.indexOf('--config') + 1]
  : join(REPO, 'tests/e2e/route-coverage.config.json');
const CONFIG = JSON.parse(readFileSync(configPath, 'utf8'));

/** Normalize a path for comparison: drop query/hash, strip trailing slash. */
function norm(p) {
  let s = p.split('?')[0].split('#')[0].trim();
  if (s.length > 1 && s.endsWith('/')) s = s.slice(0, -1);
  return s;
}

/** Extract navigable route path values from a routes.rs, filtered to `base`. */
function parseRoutes(routesRsPath, base, excludeSuffix) {
  const src = readFileSync(routesRsPath, 'utf8');
  const re = /pub const (\w+)\s*:\s*&str\s*=\s*"([^"]+)"/g;
  const routes = new Map(); // normalizedPath -> constName
  let m;
  while ((m = re.exec(src)) !== null) {
    const [, name, value] = m;
    if (name.endsWith('_LEGACY')) continue;
    if (!value.startsWith(base)) continue;
    const p = norm(value);
    if (p === norm(base)) continue; // bare base / root redirect
    if (excludeSuffix.some((suf) => p.endsWith(suf))) continue;
    if (!routes.has(p)) routes.set(p, name);
  }
  return routes;
}

/** Recursively read all *.spec.ts text under a dir (skipping node_modules etc.). */
function specText(specsDir) {
  const SKIP = new Set(['node_modules', 'playwright-report', 'test-results', 'results']);
  let text = '';
  const walk = (dir) => {
    for (const entry of readdirSync(dir)) {
      if (SKIP.has(entry)) continue;
      const full = join(dir, entry);
      const st = statSync(full);
      if (st.isDirectory()) walk(full);
      else if (entry.endsWith('.spec.ts')) text += '\n' + readFileSync(full, 'utf8');
    }
  };
  if (existsSync(specsDir)) walk(specsDir);
  return text;
}

const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * The set of route paths a spec bundle actually visits. Specs build paths from
 * file-local string consts (e.g. `const BASE = '/perlengkapan/simpel/v2'` then
 * `` `${BASE}/dashboard` ``), so we (1) resolve `${IDENT}` from string consts,
 * then (2) extract `base`-prefixed path tokens as WHOLE tokens and match them
 * exactly (so `/dashboard` never counts `/dashboard/search`, and vice-versa).
 */
function visitedRoutes(text, base) {
  // 1. Collect `const NAME = '<path-ish string>'` and substitute `${NAME}`.
  const constRe = /(?:const|let|var)\s+(\w+)\s*=\s*['"`]([^'"`]*)['"`]/g;
  const consts = new Map();
  let c;
  while ((c = constRe.exec(text)) !== null) {
    if (c[2].startsWith('/')) consts.set(c[1], c[2]);
  }
  let resolved = text;
  for (const [name, val] of consts) {
    resolved = resolved.split('${' + name + '}').join(val);
  }
  // 2. Extract base-prefixed path tokens (regardless of surrounding origin/quotes).
  const tokenRe = new RegExp(escapeRe(base) + '[A-Za-z0-9/_-]*', 'g');
  const visited = new Set();
  let t;
  while ((t = tokenRe.exec(resolved)) !== null) visited.add(norm(t[0]));
  return visited;
}

let failed = false;
const report = [];

for (const fe of CONFIG.frontends) {
  if (feFilter && fe.name !== feFilter) continue;
  const routes = parseRoutes(join(REPO, fe.routesRs), fe.base, fe.excludeSuffix || []);
  const visited = visitedRoutes(specText(join(REPO, fe.specsDir)), fe.base);
  const allow = new Set((fe.allowUncovered || []).map(norm));

  const covered = [];
  const uncovered = [];
  for (const [p] of routes) {
    if (visited.has(p)) covered.push(p);
    else uncovered.push(p);
  }
  const unexpected = uncovered.filter((p) => !allow.has(p)).sort();
  const debt = uncovered.filter((p) => allow.has(p)).sort();
  // Stale allow: listed as debt but actually covered, or not a real route.
  const routeSet = new Set([...routes.keys()]);
  const staleAllow = [...allow].filter((p) => !routeSet.has(p) || covered.includes(p)).sort();

  const feFail =
    unexpected.length > 0 ||
    staleAllow.length > 0 ||
    (fe.blocking && debt.length > 0);
  if (feFail) failed = true;

  report.push({
    name: fe.name,
    blocking: !!fe.blocking,
    total: routes.size,
    covered: covered.length,
    uncoveredDebt: debt,
    uncoveredUnexpected: unexpected,
    staleAllow,
    pass: !feFail,
  });
}

if (asJson) {
  console.log(JSON.stringify(report, null, 2));
} else {
  for (const r of report) {
    const pct = r.total ? Math.round((r.covered / r.total) * 100) : 100;
    console.log(
      `\n■ ${r.name}  ${r.covered}/${r.total} routes covered (${pct}%)  ` +
        `[${r.blocking ? 'BLOCKING' : 'advisory'}]  ${r.pass ? 'PASS' : 'FAIL'}`,
    );
    if (r.uncoveredUnexpected.length)
      console.log(
        `  ✗ UNCOVERED (not in allowlist — add an e2e or record as debt):\n` +
          r.uncoveredUnexpected.map((p) => `      ${p}`).join('\n'),
      );
    if (r.staleAllow.length)
      console.log(
        `  ✗ STALE allowUncovered (now covered / not a route — remove it):\n` +
          r.staleAllow.map((p) => `      ${p}`).join('\n'),
      );
    if (r.uncoveredDebt.length)
      console.log(
        `  … tracked debt (allowUncovered, ${r.uncoveredDebt.length}):\n` +
          r.uncoveredDebt.map((p) => `      ${p}`).join('\n'),
      );
    if (r.blocking && r.uncoveredDebt.length)
      console.log(`  ✗ BLOCKING requires zero debt — allowUncovered must be empty.`);
  }
}

if (!asJson) console.log(`\nRoute-coverage: ${failed ? 'FAIL' : 'PASS'}`);
process.exit(failed ? 1 : 0);
