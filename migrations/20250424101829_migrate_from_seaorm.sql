DROP TABLE IF EXISTS seaql_migrations;

CREATE TABLE IF NOT EXISTS dlt_cursor (
    slot int8 NOT NULL,
    block_hash bytea NOT NULL,
    CONSTRAINT dlt_cursor_pkey PRIMARY KEY (slot, block_hash)
);

CREATE TABLE IF NOT EXISTS raw_operation (
    did bytea NOT NULL,
    signed_operation_data bytea NOT NULL,
    slot int8 NOT NULL,
    block_number int8 NOT NULL,
    cbt timestamptz NOT NULL,
    absn int4 NOT NULL,
    osn int4 NOT NULL,
    CONSTRAINT raw_operation_pkey PRIMARY KEY (did, block_number, absn, osn)
);
