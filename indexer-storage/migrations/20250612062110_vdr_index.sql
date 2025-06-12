-- migrate raw_operation to not be indexed by did
DROP VIEW did_stats RESTRICT;

ALTER TABLE raw_operation
DROP COLUMN did;

ALTER TABLE raw_operation
ADD CONSTRAINT raw_operation_abs_order UNIQUE (block_number, absn, osn);

ALTER TABLE raw_operation
ADD COLUMN is_indexed BOOLEAN;

UPDATE raw_operation SET is_indexed = false
WHERE is_indexed IS null;

ALTER TABLE raw_operation
ALTER COLUMN is_indexed SET NOT NULL;

-- add indexing tables
CREATE TABLE IF NOT EXISTS indexed_ssi_operation (
    id UUID DEFAULT gen_random_uuid(),
    did BYTEA NOT NULL,
    raw_operation_id UUID NOT NULL UNIQUE,
    FOREIGN KEY (raw_operation_id) REFERENCES raw_operation (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS indexed_vdr_operation (
    id UUID DEFAULT gen_random_uuid(),
    did BYTEA NOT NULL,
    operation_hash BYTEA NOT NULL,
    initial_operation_hash BYTEA NOT NULL,
    prev_operation_hash BYTEA,
    raw_operation_id UUID NOT NULL UNIQUE,
    FOREIGN KEY (raw_operation_id) REFERENCES raw_operation (id) ON DELETE CASCADE
);
