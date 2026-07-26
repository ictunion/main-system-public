ALTER TABLE workplaces
    DROP COLUMN state,
    ADD COLUMN announced_at TIMESTAMPTZ,
    ADD COLUMN established_at TIMESTAMPTZ;

DROP TYPE workplace_state;

COMMENT ON COLUMN workplaces.announced_at IS 'Timestamp when the workplace was announced';
COMMENT ON COLUMN workplaces.established_at IS 'Timestamp when the workplace was officially established by the Board; null means not yet established';