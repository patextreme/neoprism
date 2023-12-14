CREATE TABLE raw_operation (
    did TEXT NOT NULL,
    signed_operation_data BLOB NOT NULL,
    slot INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    cbt TEXT NOT NULL,
    absn INTEGER NOT NULL,
    osn INTEGER NOT NULL,
    UNIQUE (did, block_number, absn, osn)
);

CREATE TABLE dlt_cursor (
    slot INTEGER PRIMARY KEY,
    block_hash BLOB NOT NULL
);
