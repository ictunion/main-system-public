-- Note that we make some assumptions about db users and database names to keep things simple
-- If one would want to deploy system in some different settings
-- there will likely need to be changes in migrations files

-- Harden Database
REVOKE ALL ON SCHEMA PUBLIC FROM public; -- removes default access to public schema from everyone

-- Create Role for Orca
CREATE USER orca;
COMMENT ON ROLE orca is 'Database user of orca service';

-- Setup Permissions
GRANT CONNECT ON DATABASE ictunion TO orca; -- grant connect to DB
GRANT USAGE ON SCHEMA public TO orca; -- grant usage of public schema

-- Ensure we have UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

