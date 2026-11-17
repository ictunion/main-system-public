CREATE TYPE note_target AS ENUM ('application', 'member', 'member_dues', 'workplace');

CREATE TABLE notes
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , target_type note_target NOT NULL
    , registration_request_id UUID REFERENCES registration_requests(id) ON DELETE CASCADE
    , member_id UUID REFERENCES members(id) ON DELETE CASCADE
    , workplace_id UUID REFERENCES workplaces(id) ON DELETE CASCADE
    , body TEXT NOT NULL CHECK (length(btrim(body)) > 0)
    , author_sub UUID
    , author_name TEXT NOT NULL
    , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    , updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    , CONSTRAINT notes_one_target CHECK (
        CASE target_type
          WHEN 'application' THEN registration_request_id IS NOT NULL AND member_id IS NULL AND workplace_id IS NULL
          WHEN 'member'      THEN member_id IS NOT NULL AND registration_request_id IS NULL AND workplace_id IS NULL
          WHEN 'member_dues' THEN member_id IS NOT NULL AND registration_request_id IS NULL AND workplace_id IS NULL
          WHEN 'workplace'   THEN workplace_id IS NOT NULL AND registration_request_id IS NULL AND member_id IS NULL
        END)
    );

CREATE INDEX notes_application_idx ON notes(registration_request_id, created_at DESC) WHERE registration_request_id IS NOT NULL;
CREATE INDEX notes_member_idx      ON notes(member_id, target_type, created_at DESC)  WHERE member_id IS NOT NULL;
CREATE INDEX notes_workplace_idx   ON notes(workplace_id, created_at DESC)            WHERE workplace_id IS NOT NULL;

CREATE TRIGGER notes_updated_at BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

COMMENT ON TABLE notes IS 'Free-text notes attached to an application, member, member dues thread or workplace. Notes are temporary and HARD-deleted on purpose (unlike every other table); do not add soft delete.';
COMMENT ON COLUMN notes.target_type IS 'Which FK column is set: application -> registration_request_id, member and member_dues -> member_id, workplace -> workplace_id';
COMMENT ON COLUMN notes.author_sub IS 'Keycloak sub of the author, used for ownership checks. NULL for notes imported from the old single-note columns';
COMMENT ON COLUMN notes.author_name IS 'Display name of the author copied from the access token at write time';

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE notes TO orca;
