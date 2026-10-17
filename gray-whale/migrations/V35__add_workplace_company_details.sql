ALTER TABLE workplaces
    ADD COLUMN company_id TEXT,
    ADD COLUMN company_address TEXT,
    ADD COLUMN workplace_short_name TEXT;

COMMENT ON COLUMN workplaces.company_id IS 'Identification number (IČ) of the employer company';
COMMENT ON COLUMN workplaces.company_address IS 'Registered address of the employer company';
COMMENT ON COLUMN workplaces.workplace_short_name IS 'Short name of the workplace used in union documents (e.g. "ACME CZ")';
