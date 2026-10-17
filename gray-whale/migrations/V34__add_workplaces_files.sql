CREATE TABLE workplaces_files
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
    , workplace_id UUID NOT NULL REFERENCES workplaces(id)
    , file_id UUID NOT NULL REFERENCES files(id)
    , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

CREATE INDEX workplaces_files_workplace_id_idx ON workplaces_files(workplace_id);
CREATE INDEX workplaces_files_file_id_idx ON workplaces_files(file_id);

COMMENT ON TABLE workplaces_files IS 'Files generated for or attached to a workplace (e.g. entrustment PDFs)';

GRANT SELECT, INSERT ON TABLE workplaces_files TO orca;
