CREATE TABLE registration_requests_workplaces
    ( registration_request_id UUID PRIMARY KEY
        REFERENCES registration_requests(id) ON DELETE CASCADE
    , workplace_id UUID NOT NULL REFERENCES workplaces(id)
    , assigned_at TIMESTAMPTZ NOT NULL DEFAULT now()
    , approved_by_reps_at TIMESTAMPTZ
    );

-- Workplace-scoped listing (rep "Workplace Applications" page) filters by workplace_id
CREATE INDEX registration_requests_workplaces_workplace_id_idx
    ON registration_requests_workplaces (workplace_id);

COMMENT ON TABLE registration_requests_workplaces IS 'Workplace an application is assigned to; at most one per application. Assigned by admins, read and approved/rejected by that workplace''s executive committee. Kept after acceptance as a record of which workplace handled it.';
COMMENT ON COLUMN registration_requests_workplaces.approved_by_reps_at IS 'When the workplace executive committee recommended acceptance; null means not approved. Reset when the application is moved to another workplace or rejected. Admins still perform the accept.';

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE registration_requests_workplaces TO orca;
