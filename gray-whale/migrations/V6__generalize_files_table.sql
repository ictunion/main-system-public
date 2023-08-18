CREATE TABLE registration_requests_files
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , registration_request_id UUID REFERENCES registration_requests(id)
    , file_id UUID REFERENCES files(id)
    , created_at TIMESTAMPTZ DEFAULT NOW()
    );

CREATE INDEX registration_request_files_registration_request ON registration_requests_files(registration_request_id);
CREATE INDEX registration_request_files_file ON registration_requests_files(file_id);

-- Migrate existing data into new column
INSERT INTO registration_requests_files (file_id, registration_request_id, created_at)
(SELECT id, registration_request_id, created_at from files);

-- Grant permissions for new table
GRANT SELECT, INSERT ON TABLE registration_requests_files TO orca;
GRANT SELECT ON TABLE registration_requests_files TO members_manager;

-- drop old column
ALTER TABLE files
DROP COLUMN registration_request_id;

-- create new table for members' files
CREATE TABLE members_files
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , member_id UUID REFERENCES members(id)
    , file_id UUID REFERENCES files(id)
    , created_at TIMESTAMPTZ DEFAULT NOW()
    );

CREATE INDEX members_files_registration_request ON registration_requests_files(registration_request_id);
CREATE INDEX members_files_file ON registration_requests_files(file_id);

GRANT ALL ON TABLE members_files TO members_manager;
