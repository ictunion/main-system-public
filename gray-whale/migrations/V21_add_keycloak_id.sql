-- Initially, the column is created as nullable to backfill missing data for existing entries, next migration sets NOT NULL
ALTER TABLE workplaces
    ADD COLUMN keycloak_group_id TEXT;

COMMENT ON COLUMN workplaces.keycloak_group_id IS 'Workplace ID as present in keycloak';
