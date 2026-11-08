-- Who holds which pakaian dinas role where, learned from who actually uses the app.
--
-- A notification needs a recipient, and the workflow engine's approver lookup has
-- always been a placeholder that returns nobody (it was meant to ask authenc, and
-- the call was never wired). So the only notifications that ever reached a person
-- were the ones addressed to a user already named on the record, such as the
-- reporter of a help ticket.
--
-- The pakaian dinas flow hands work from the operator to a wilayah validator to the
-- pusat validator and back; each handover is exactly "tell the next role". The
-- people who can receive that are the people who open the app (the bell is the only
-- place a notification is read), so this table is filled from authenticated
-- requests: the first time a user touches the pakaian dinas screens they become
-- reachable, with the roles and satker their token carried.
--
-- Additive: a new table nothing else reads, so rolling back to an older binary
-- leaves it unused. Replay-safe (the runner re-applies every migration).
CREATE TABLE IF NOT EXISTS perlengkapan.pengguna_dikenal (
    user_id     UUID PRIMARY KEY,
    nama        TEXT,
    -- Every role seen for this user. With session-scoped role activation a token
    -- carries one role, so the set grows as the user switches.
    roles       TEXT[] NOT NULL DEFAULT '{}',
    satker_code TEXT,
    last_seen   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pengguna_dikenal_roles
    ON perlengkapan.pengguna_dikenal USING GIN (roles);
