-- Whether an office is current or past is now decided by term_ends_on, and
-- deleted_at only marks records entered by mistake. Not holding more than one
-- office at a time is left to the people managing the board.
DROP INDEX member_offices_active_member_id;

CREATE INDEX member_offices_member_id ON member_offices (member_id);

COMMENT ON TABLE member_offices IS 'Elected offices held by members within the organization. An office is current until its term_ends_on has passed, then it is history.';
COMMENT ON COLUMN member_offices.deleted_at IS 'Soft delete for records entered by mistake. Deleted records are hidden everywhere; ended offices are not deleted.';
