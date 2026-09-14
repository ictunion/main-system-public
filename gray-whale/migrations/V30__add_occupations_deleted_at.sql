ALTER TABLE occupations ADD COLUMN deleted_at TIMESTAMPTZ;

COMMENT ON COLUMN occupations.deleted_at IS 'Soft-delete marker; null means the record is active.';
