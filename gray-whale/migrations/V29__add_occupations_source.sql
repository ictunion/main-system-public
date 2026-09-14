ALTER TABLE occupations ADD COLUMN source TEXT;

UPDATE occupations SET source = 'application';

ALTER TABLE occupations ALTER COLUMN source SET NOT NULL;

COMMENT ON COLUMN occupations.source IS 'Where this occupation record came from, e.g. "application" (filled during registration) or "orca" (added by admins on the member detail page)';
