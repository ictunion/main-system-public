ALTER TABLE workplaces
    DROP COLUMN chairperson_id,
    ADD COLUMN email TEXT NOT NULL;

COMMENT ON COLUMN workplaces.email IS 'Email that was created for the workplace in our mail server';
