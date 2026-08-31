ALTER TABLE members_workplaces
    ADD COLUMN is_representative BOOLEAN DEFAULT FALSE;

COMMENT ON COLUMN members_workplaces.is_representative IS 'Is the member a representative for the workplace?';
