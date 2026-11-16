# ci-cache — shared compile cache for the CI runners

CI compiles the same dependency tree in a dozen ephemeral runner pods per run, from
scratch each time: one full CI run is ~1,200 runner-minutes, most of it `rustc`. This
chart runs the store those pods share: [sccache](https://github.com/mozilla/sccache)
caches each `rustc` invocation by a hash of its inputs, here in a small WebDAV
server with a Longhorn volume.

It is a **cache**. Losing the volume costs one cold run, never data. Nothing in the
cluster depends on it, and CI treats an unreachable cache as "no cache".

## What it measured

On a dev machine, building `lib-core` twice through sccache against this server
(same target path, as CI has):

| Build | Time | sccache |
|---|---|---|
| 1st (cold) | 53 s | 65 misses |
| 2nd | 12 s | 65 hits, 0 misses, 0 errors |

The saving in CI depends on how much of a job is `rustc` rather than linking and
running tests; read the `sccache` block each job now writes to its summary.

## Install

Once per cluster, in its own namespace. The chart does not create the namespace, so
`helm uninstall` can never delete one:

```bash
helm upgrade --install ci-cache infra/helm/ci-cache \
  --namespace ci-cache --create-namespace
kubectl -n ci-cache rollout status deploy/ci-cache
# From a node: 207 means the server answers what sccache's startup check asks.
curl -s -o /dev/null -w '%{http_code}\n' -X PROPFIND -H 'Depth: 0' \
  http://172.15.10.252:32010/
```

Merging the CI change before installing is safe: `setup-rust` probes the endpoint,
logs a warning and builds without the cache when it does not answer.

## How CI uses it

`.github/actions/setup-rust` with `sccache: "true"` downloads a pinned sccache
(checked against a pinned sha256), exports `RUSTC_WRAPPER`, and points it at
`http://172.15.10.252:32010` (the node IP: runner pods cannot resolve cluster DNS).
Only the CI checks opt in (clippy, tests, the gate builds). `check-ci-scope.py`
fails if a job enables it without reporting its statistics, or if `release.yml`
ever uses it.

Entries only hit when the *absolute* paths match, which they do on ARC runners (the
workspace and `CARGO_HOME` have the same path in every pod). A job that builds in a
different directory simply misses.

## Trust model

The server has no authentication, like the in-cluster registry next to it
(`172.15.10.252:32000`): both are reachable by anything on the node network. That is
acceptable for a store whose contents influence **checks**, and it is why release
builds, which produce what gets shipped, never use it. If that boundary ever needs to
move, put users in `templates/configmap.yaml` and the credentials in a repository
secret first.

## Operations

- **Size / expiry.** `persistence.size` (default 40Gi) is the hard ceiling; when it
  fills, writes fail and sccache carries on uncached. The `janitor` sidecar deletes
  entries older than `janitor.retentionDays` (default 14) every `janitor.intervalHours`.
- **Empty it.** Delete the PVC's contents or the PVC itself; the next run is cold.
- **Why this image.** `values.yaml` records what was tried: nginx's stock DAV module
  answers `PROPFIND` with 405 and every write fails; Apache `mod_dav` needs a lock
  database the alpine image cannot open; `hacdias/webdav` works.
- **Upgrading sccache** means bumping `SCCACHE_VERSION` and `SCCACHE_SHA256` together
  in `setup-rust/action.yml`.
