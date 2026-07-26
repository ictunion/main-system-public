CREATE TYPE workplace_state AS ENUM ('announced', 'established', 'cancelled');

ALTER TABLE workplaces
    ADD COLUMN state workplace_state NOT NULL DEFAULT 'announced',
    ADD COLUMN cancelled_at TIMESTAMPTZ,
    ADD COLUMN newsletter_id INTEGER,
    ADD COLUMN keycloak_executive_group_id UUID,
    ALTER COLUMN keycloak_group_id SET NOT NULL;

COMMENT ON COLUMN workplaces.state IS 'Lifecycle state of the workplace';
COMMENT ON COLUMN workplaces.cancelled_at IS 'Timestamp when the workplace was cancelled';
COMMENT ON COLUMN workplaces.newsletter_id IS 'Listmonk newsletter/list ID associated with this workplace';
COMMENT ON COLUMN workplaces.keycloak_executive_group_id IS 'Keycloak group ID for the workplace executives/representatives';