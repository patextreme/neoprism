-- Add migration script here
CREATE TABLE IF NOT EXISTS scheduled_operation (
    id UUID DEFAULT gen_random_uuid(),
    signed_operation BYTEA NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL
);
