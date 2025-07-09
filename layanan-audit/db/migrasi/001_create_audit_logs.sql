CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE audit.logs (
    id SERIAL PRIMARY KEY,
    user_id TEXT,
    method TEXT,
    path TEXT,
    timestamp TIMESTAMPTZ,
    ip TEXT
);
