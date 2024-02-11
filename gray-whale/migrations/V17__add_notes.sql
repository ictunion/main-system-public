ALTER TABLE registration_requests
    ADD COLUMN note TEXT;

COMMENT ON COLUMN registration_requests.note IS 'Editable note for registration request';

ALTER TABLE members
    ADD COLUMN note TEXT;

COMMENT ON COLUMN members.note IS 'Editable note for members';

CREATE OR REPLACE VIEW registration_requests_invalid AS
    SELECT * FROM registration_requests rr
    WHERE rr.invalidated_at IS NOT NULL;

COMMENT ON VIEW registration_requests_invalid IS 'All registration requests which are waiting on email confirmation';

GRANT SELECT ON registration_requests_invalid TO orca;

CREATE OR REPLACE VIEW registration_requests_unverified AS
    SELECT * FROM registration_requests rr
    WHERE rr.confirmed_at IS NULL
    AND rr.rejected_at IS NULL
    AND rr.invalidated_at IS NULL
    AND NOT EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_unverified IS 'All registration requests which are waiting on email confirmation';

CREATE OR REPLACE VIEW registration_requests_processing AS
    SELECT * FROM registration_requests rr
    WHERE rr.confirmed_at IS NOT NULL
    AND rr.rejected_at IS NULL
    AND rr.invalidated_at IS NULL
    AND NOT EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_processing IS 'All registrations which are confirmed by applicant but are not yet either rejected or accepted';

CREATE OR REPLACE VIEW registration_requests_accepted AS
    SELECT * FROM registration_requests rr
    WHERE EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_accepted IS 'All registration requests that were accepted (as members)';

CREATE OR REPLACE VIEW registration_requests_rejected AS
    SELECT * FROM registration_requests rr
    WHERE rr.rejected_at IS NOT NULL;

COMMENT ON VIEW registration_requests_rejected IS 'All registration requests that were rejected';

CREATE OR REPLACE VIEW members_new AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.sub IS NULL;

COMMENT ON VIEW members_new IS ' Members who do not yet have login information';

CREATE OR REPLACE VIEW members_current AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.sub IS NOT NULL;

COMMENT ON VIEW members_current IS 'Members with login information which did not left';

CREATE OR REPLACE VIEW members_past AS
    SELECT * FROM members m
    WHERE m.left_at IS NOT NULL;

COMMENT ON VIEW members_past IS 'Members who left';
