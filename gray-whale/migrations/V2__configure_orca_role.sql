-- Create Roles
create role ictunion;
comment on role ictunion is 'Universal role permitting usage of ictunion database';

create user orca;
comment on role orca is 'Database user of orca service';

-- Make orca member of ictunion
grant ictunion to orca;

-- Setup Permissions
revoke all on schema public from public; -- removes default access to public schema from everyone

grant connect on database ictunion to ictunion; -- give role ictunion (all members) right to connect to the database
grant usage on schema public to ictunion; -- grant role ictunion access to public schema of the database

grant select on table members to orca; -- allow orca to read the data in members table
grant insert on table members to orca; -- allow orca to insert datat to members table
