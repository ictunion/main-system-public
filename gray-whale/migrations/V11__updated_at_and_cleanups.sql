CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

-- Track updated at for members

ALTER TABLE members
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TRIGGER update_updated_at_members
    BEFORE UPDATE ON members
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at();

-- Track update at for registration_tasks

ALTER TABLE registration_tasks
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TRIGGER update_updated_at_registration_tasks
    BEFORE UPDATE ON registration_tasks
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at();

-- Remove columns we decided not to use

ALTER table registration_tasks
    DROP COLUMN author_id;

-- Create sub for members

ALTER TABLE members
    ADD COLUMN sub UUID;

CREATE UNIQUE INDEX member_sub ON members(sub);

COMMENT ON COLUMN members.sub IS 'OpenID SUB which is unique idenifier for identity from identity provider like keycloak.';
