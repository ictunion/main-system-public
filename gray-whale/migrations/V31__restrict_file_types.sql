ALTER TABLE files
    ADD CONSTRAINT files_file_type_check CHECK (file_type IN ('png', 'jpg', 'jpeg', 'pdf'));

COMMENT ON COLUMN files.file_type IS 'Normalized file extension. Allowed values are enforced by files_file_type_check and must match orca''s FileType enum.';
