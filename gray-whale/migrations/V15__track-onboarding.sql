ALTER TABLE members
    ADD COLUMN onboarding_finished_at TIMESTAMPTZ;

COMMENT ON COLUMN members.onboarding_finished_at IS 'Time member went from NEW to CURRENT members';

UPDATE members
    SET onboarding_finished_at = NOW()
    WHERE sub IS NOT NULL;
