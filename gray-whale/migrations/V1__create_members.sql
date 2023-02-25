create table members
    ( id integer primary key generated always as identity

    /*
     Data filled during registration:
    */

    -- we allow a lot of nulls because of gdpr
    , email text
    , first_name text
    , last_name text
    , date_of_birth date
    , address text
    , city text
    -- better as text in case some valid value
    -- could start with 0 (though unlikely for cze)
    , postal_code text
    -- we store international prefix like +420 so text
    , phone_number text
    -- lets keep this as a free field
    -- but in the future we might also have a company table and
    -- have relations between companies and members
    , company_name text
    , occupation text
    , signature_file text

    /*
     Atomatically populated data
     Also can be null because we shuld also support
     adding users manually in which case we don't all the vlaues
    */
    , created_at timestamptz not null default now()
    , verification_sent_at timestamptz
    , registration_ip inet
    -- lets store it as text for now. eventually it might be relation?
    -- this will be an ISO 639-1 value
    , registration_local text
    , registration_user_agent text
    , registration_source text

    /*
     Values needed for further business logic processing
    */
    , confirmation_token text
    , confirmed_at timestamptz

    /*
     GDPR
    */
    , personal_info_wiped_at timestamptz
    );

create index member_email on members(email);
create index member_city on members(city);
create index member_postal_code on members(postal_code);
create index member_company_name on members(company_name);
create index member_created_at on members(created_at);
create index member_registration_local on members(registration_local);
create index member_registration_source on members(registration_source);
create index member_confirmation_token on members(confirmation_token);
create index member_confirmed_at on members(confirmed_at);

comment on column members.email is 'Null means user submitted GDPR request to remove or personal information';
comment on column members.phone_number is 'Phone number prefixed by code like +420 666 777 888';
comment on column members.company_name is 'Company name filled in during registration. This is free text that we can later normalize further';
comment on column members.registration_local is 'ISO 639-1 value of localization used during registration';
comment on column members.registration_source is 'Text field with identifier of source of registation';
comment on column members.confirmation_token is 'Value used for email confirmation logic';
comment on column members.confirmed_at is 'Time of email confirmation. Null mean user never confirmed their email';
comment on column members.personal_info_wiped_at is 'Time when GDPR request to remove all personal information was processed';
