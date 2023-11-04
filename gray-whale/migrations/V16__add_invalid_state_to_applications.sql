ALTER TABLE registration_requests
    ADD COLUMN invalidated_at TIMESTAMPTZ;

COMMENT ON COLUMN registration_requests.invalidated_at IS 'Time at which application was marked as invalid';

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

CREATE OR REPLACE VIEW registration_requests_procession AS
    SELECT * FROM registration_requests rr
    WHERE rr.confirmed_at IS NOT NULL
    AND rr.rejected_at IS NULL
    AND rr.invalidated_at IS NULL
    AND NOT EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_procession IS 'All registrations which are confirmed by applicant but are not yet either rejected or accepted';
