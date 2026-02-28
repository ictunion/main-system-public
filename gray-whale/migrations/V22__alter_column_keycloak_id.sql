ALTER TABLE workplaces
    ALTER COLUMN keycloak_group_id TYPE uuid USING keycloak_group_id::uuid;
