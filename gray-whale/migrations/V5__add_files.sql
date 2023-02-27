-- lets use UUID so we generate less predictable id values for files
create extension if not exists "uuid-ossp";

create table files
    ( id uuid primary key default uuid_generate_v4()
    , name text not null
    , file_type text not null
    , data bytea
    , user_id int references members(id)
    , created_at timestamptz not null default now()
    );

create index file_name on files(name);
create index file_type on files(file_type);
create index created_at on files(created_at);

comment on table files is 'Files data are directly stored into this table (within database)!';
comment on column files.data is 'Binary data of a file';
comment on column files.file_type is 'This should contain values of normalized file extension (like "png" or "pdf" etc)';

-- remove columns we won't be using from members
alter table members
drop column signature_file;

-- grant permissions to orca
grant select, insert on table files to orca;
