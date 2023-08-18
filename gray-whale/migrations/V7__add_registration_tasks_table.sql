-- Crete table for checklist items during registration/application processing
CREATE TABLE registration_tasks
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , name TEXT NOT NULL
    , description TEXT
    , author_id UUID REFERENCES members(id)
    , created_at TIMESTAMPTZ DEFAULT NOW()
    , canceled_at TIMESTAMPTZ
    );

CREATE INDEX registration_tasks_canceled_at ON registration_tasks(canceled_at);

COMMENT ON TABLE registration_tasks IS 'Checklist items for registration processing';

-- Grant permissions for new table
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE registration_tasks TO orca;

-- Alter existing table so it refers to new tasks
ALTER TABLE registration_events
DROP COLUMN TYPE,
ADD COLUMN registration_task_id UUID REFERENCES registration_tasks(id);

CREATE INDEX registration_events_registration_task_id ON registration_events(registration_task_id);
