CREATE OR REPLACE VIEW members_new AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.sub IS NULL;

COMMENT ON VIEW members_new IS ' Members who do not yet have login information';
GRANT SELECT ON members_new TO orca;

CREATE OR REPLACE VIEW members_current AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.sub IS NOT NULL;

COMMENT ON VIEW members_current IS 'Members with login information which did not left';
GRANT SELECT ON members_current TO orca;

CREATE OR REPLACE VIEW members_past AS
    SELECT * FROM members m
    WHERE m.left_at IS NOT NULL;

COMMENT ON VIEW members_past IS 'Members who left';
GRANT SELECT ON members_past TO orca;
