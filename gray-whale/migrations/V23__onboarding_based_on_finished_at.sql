CREATE OR REPLACE VIEW members_new AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.onboarding_finished_at IS NULL;

COMMENT ON VIEW members_new IS 'Members who have not yet been onboarded';

CREATE OR REPLACE VIEW members_current AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.onboarding_finished_at IS NOT NULL;

COMMENT ON VIEW members_current IS 'Members who have been onboarded and have not left';
