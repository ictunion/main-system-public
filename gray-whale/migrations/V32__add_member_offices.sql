CREATE TABLE member_offices
     ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
     , member_id UUID NOT NULL REFERENCES members(id)
     , title TEXT NOT NULL
     , elected_by TEXT NOT NULL
     , elected_on DATE NOT NULL
     , term_ends_on DATE
     , deleted_at TIMESTAMPTZ
     , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
     , updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
     , CHECK (term_ends_on IS NULL OR term_ends_on >= elected_on)
     );

-- A member holds at most one active office; ended offices stay as history.
CREATE UNIQUE INDEX member_offices_active_member_id ON member_offices (member_id) WHERE deleted_at IS NULL;

COMMENT ON TABLE member_offices IS 'Elected offices held by members within the organization, including past ones. A member holds at most one active office at a time.';
COMMENT ON COLUMN member_offices.title IS 'Name of the office, e.g. "Chairperson", "Vice-chairperson", "Board member". Free text so new offices need no migration.';
COMMENT ON COLUMN member_offices.elected_by IS 'Body or process that elected the member, e.g. all members meeting. Free text.';
COMMENT ON COLUMN member_offices.term_ends_on IS 'Last day of the term of office; null means the term has no fixed end.';
COMMENT ON COLUMN member_offices.deleted_at IS 'Soft-delete marker: when the record was moved to history. Null means the office is active. Metadata only -- the end of the office is term_ends_on.';

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE member_offices TO orca;
