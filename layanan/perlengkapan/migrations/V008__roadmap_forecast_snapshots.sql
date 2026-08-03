-- Create the forecast snapshot store that the roadmap module has been
-- creating at RUNTIME.
--
-- `RoadmapRepository::ensure_snapshot_table()` issued this exact
-- `CREATE TABLE IF NOT EXISTS` before every snapshot read and write, so the
-- table did exist in practice — but only as a side effect of serving a request,
-- and it appeared in no migration. That is the same runtime-DDL pattern removed
-- in V007 (boot-time indexes): schema owned by application code drifts silently,
-- is invisible to anyone reading the migrations, cannot be reviewed, and costs a
-- DDL round-trip on every call.
--
-- Column definitions are carried over verbatim from that DDL so this is a pure
-- relocation, not a redefinition; `IF NOT EXISTS` keeps it a no-op on any
-- database where the runtime path already created it.
CREATE TABLE IF NOT EXISTS perlengkapan.roadmap_forecast_snapshots (
    id UUID PRIMARY KEY,
    method TEXT NOT NULL,
    confidence_level DOUBLE PRECISION NOT NULL DEFAULT 0.95,
    satker_id UUID,
    kode_barang TEXT,
    predictions_json TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- `get_previous_snapshots` always filters on satker_id + kode_barang and orders
-- by created_at DESC.
CREATE INDEX IF NOT EXISTS idx_roadmap_forecast_snapshots_lookup
    ON perlengkapan.roadmap_forecast_snapshots (satker_id, kode_barang, created_at DESC);
