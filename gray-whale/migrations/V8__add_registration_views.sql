-- Make members number unique
ALTER TABLE members ADD UNIQUE (member_number);

-- Add rejceted at column to registration requests
ALTER TABLE registration_requests
    ADD COLUMN rejected_at TIMESTAMPTZ;

COMMENT ON COLUMN registration_requests.rejected_at IS 'Time when request was rejected. If NULL then it was never rejected';

-- Create views

CREATE OR REPLACE VIEW registration_requests_unverified AS
    SELECT * FROM registration_requests rr
    WHERE rr.confirmed_at IS NULL
    AND rr.rejected_at IS NULL
    AND NOT EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_unverified IS 'All registration requests which are waiting on email confirmation';

GRANT SELECT ON registration_requests_unverified TO orca;

CREATE OR REPLACE VIEW registration_requests_accepted AS
    SELECT * FROM registration_requests rr
    WHERE EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_accepted IS 'All registration requests that were accepted (as members)';

GRANT SELECT ON registration_requests_accepted TO orca;

CREATE OR REPLACE VIEW registration_requests_rejected AS
    SELECT * FROM registration_requests rr
    WHERE rr.rejected_at IS NOT NULL;

COMMENT ON VIEW registration_requests_rejected IS 'All registration requests that were rejected';

GRANT SELECT ON registration_requests_rejected TO orca;

CREATE OR REPLACE VIEW registration_requests_procession AS
    SELECT * FROM registration_requests rr
    WHERE rr.confirmed_at IS NOT NULL
    AND rr.rejected_at IS NULL
    AND NOT EXISTS (SELECT id FROM members m WHERE rr.id = m.registration_request_id);

COMMENT ON VIEW registration_requests_procession IS 'All registrations which are confirmed by applicant but are not yet either rejected or accepted';

GRANT SELECT ON registration_requests_procession TO orca;
