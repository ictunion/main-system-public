REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM members_manager;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM members_manager;
REVOKE ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public FROM members_manager;
REVOKE ALL PRIVILEGES ON SCHEMA public FROM members_manager;

DROP ROLE members_manager;
DROP ROLE postgrest_authenticator;
