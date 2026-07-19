//! Static asset URLs.
//!
//! Deliberately NOT in `routes.rs`. That file is the route set, and
//! `tests/e2e/route-coverage.mjs` scrapes EVERY
//! `pub const NAME: &str = "<base>/…"` out of it — file-wide, not per-module —
//! then demands an e2e that visits each one. An asset is not a navigable
//! route, so a logo constant living there made the gate ask for a spec that
//! browses to a PNG. The gate is correct and stays strict: no
//! `allowUncovered` debt, no loosened `excludeSuffix` (a `.png` suffix rule
//! could mask a genuine route later). The constant simply belongs elsewhere.

/// The app is served under `/perlengkapan/simpel/v2/`. Four independent places
/// agree on that prefix: `Trunk.toml` `public_url`, `<base href>` in
/// index.html, the nginx `location /perlengkapan/simpel/v2/` block, and the
/// Dockerfile COPY target `/usr/share/nginx/html/perlengkapan/simpel/v2`.
///
/// Three call sites used to hard-code `/perlengkapan/assets/kejaksaan-logo.png`,
/// missing the `/simpel/v2` segment. That 404s rather than falling back to the
/// SPA index because nginx's static-asset regex location
/// (`~* \.(wasm|js|css|png|…)$`) matches before any prefix location and has no
/// `try_files`, so it resolves straight against root to a path that does not
/// exist. One constant, so the prefix lives in exactly one place.
pub const LOGO: &str = "/perlengkapan/simpel/v2/assets/kejaksaan-logo.png";
