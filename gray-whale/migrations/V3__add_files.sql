CREATE TABLE FILES
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , name TEXT NOT NULL
    , file_type TEXT NOT NULL
    , data BYTEA
    , registration_request_id UUID REFERENCES registration_requests(id)
    , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

CREATE INDEX file_name ON files(name);
CREATE INDEX file_type ON files(file_type);
CREATE INDEX created_at ON files(created_at);

COMMENT ON TABLE files IS 'Files data are directly stored into this table (within database)!';
COMMENT ON column files.data IS 'Binary data of a file';
COMMENT ON column files.file_type IS 'This should contain values of normalized file extension (like "png" or "pdf" etc)';

-- Grant permissions to orca
GRANT SELECT, INSERT ON TABLE files TO orca;
