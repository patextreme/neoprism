-- Add migration script here
CREATE TABLE IF NOT EXISTS staging_operation (
    id UUID DEFAULT gen_random_uuid(),
    signed_operation BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
