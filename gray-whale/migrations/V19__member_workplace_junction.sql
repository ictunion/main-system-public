CREATE TABLE members_workplaces
    ( member_id UUID NOT NULL REFERENCES members(id)
    , workplace_id UUID NOT NULL REFERENCES workplaces(id)
    , PRIMARY KEY (member_id, workplace_id)
    );

COMMENT ON TABLE members_workplaces IS 'Members Workplaces is junction table for Many-to-Many relation between members and workplaces';
