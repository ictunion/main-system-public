ALTER TABLE members_workplaces
    ADD COLUMN became_representative_at TIMESTAMPTZ;

UPDATE members_workplaces
    SET became_representative_at = now()
    WHERE is_representative = TRUE;

ALTER TABLE members_workplaces
    DROP COLUMN is_representative;

COMMENT ON COLUMN members_workplaces.became_representative_at IS 'Timestamp when the member became a representative for the workplace; null means not a representative';

-- Speed up lookups by workplace_id. The primary key (member_id, workplace_id)
-- already covers filtering by member_id via its leading column, but a bare
-- workplace_id filter (workplace member counts, roster joins) has no usable
-- index and falls back to a sequential scan.
CREATE INDEX members_workplaces_workplace_id_idx
    ON members_workplaces (workplace_id);

-- Unindexed foreign keys on per-member child tables. Both are filtered by
-- member_id on hot paths -- occupations is LEFT JOINed on member_id in every
-- member listing query, and members_files is filtered by member_id when
-- listing a member's files -- and both are scanned to validate the FK
-- whenever a member row is deleted.
CREATE INDEX occupations_member_id_idx
    ON occupations (member_id);

CREATE INDEX members_files_member_id_idx
    ON members_files (member_id);

-- members.workplace_id predates the members_workplaces junction table (V19)
-- and is no longer written or read by application code -- the junction table
-- is now the single source of truth for member <-> workplace links.
--
-- The members_new / members_current / members_past views are SELECT * over
-- members, so the dropped column is frozen into their definitions. They must
-- be dropped and recreated (CREATE OR REPLACE cannot remove a column), which
-- also drops the grants from V14, so those are reissued here. View bodies
-- below match their latest definitions (V23 for new/current, V17 for past).
DROP VIEW members_new;
DROP VIEW members_current;
DROP VIEW members_past;

ALTER TABLE members DROP COLUMN workplace_id;

CREATE VIEW members_new AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.onboarding_finished_at IS NULL;
COMMENT ON VIEW members_new IS 'Members who have not yet been onboarded';
GRANT SELECT ON members_new TO orca;

CREATE VIEW members_current AS
    SELECT * FROM members m
    WHERE m.left_at IS NULL
    AND m.onboarding_finished_at IS NOT NULL;
COMMENT ON VIEW members_current IS 'Members who have been onboarded and have not left';
GRANT SELECT ON members_current TO orca;

CREATE VIEW members_past AS
    SELECT * FROM members m
    WHERE m.left_at IS NOT NULL;
COMMENT ON VIEW members_past IS 'Members who left';
GRANT SELECT ON members_past TO orca;
