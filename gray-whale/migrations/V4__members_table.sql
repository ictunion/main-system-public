CREATE TABLE members
	 ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
     -- incremental number that is also used as variable symbol for payments
	 , member_number INT NOT NULL
	 , email TEXT NOT NULL
	 , last_name TEXT
	 , first_name TEXT
	 , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
	 , left_at TIMESTAMPTZ
	 , date_of_birth DATE
	 , address TEXT
	 , city TEXT
	 , postal_code TEXT
	 , language TEXT
	 , phone_number TEXT
	 , registration_request_id UUID REFERENCES registration_requests(id)
	 );

create index member_member_number on members(member_number);
create index member_email on members(email);
create index member_last_name on members(last_name);
create index member_first_name on members(first_name);
create index member_city on members(city);
create index member_postal_code on members(postal_code);
create index member_created_at on members(created_at);

comment on column members.member_number is 'Incremental number that is also used as variable symbol in bank transfers';

CREATE TABLE workplaces
	( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
	, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
	, name TEXT NOT NULL
	, chairperson_id UUID NOT NULL REFERENCES members(id)
	);

create index workplace_created_at on workplaces(created_at);
create index workplace_name on workplaces(name);

comment on column workplaces.chairperson_id is 'Id of union member who in role of chairpeson of workplace';

ALTER TABLE members
    ADD COLUMN workplace_id UUID REFERENCES workplaces(id);

CREATE TABLE occupations
	( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
	, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
	, company_name TEXT
	, position TEXT
	, member_id UUID REFERENCES members(id)
	);

create index occupations_created_at on occupations(created_at);
create index occupations_company_name on occupations(company_name);

comment on column occupations.company_name is 'Company name filled by member. This is free text that can later be normalized further';
comment on column occupations.position is 'Job position filled by member. This is free text that can later be normalized further';

CREATE TABLE registration_events
	( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
	, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
	, registration_request_id UUID REFERENCES registration_requests(id)
    --Id of user who initiated that event (eg. called with new member, added them to mailing list, etc)
	, initiator_id UUID REFERENCES members(id)
	, type INT
	, note TEXT
	);

create index registration_event_created_at on registration_events(created_at);
create index registration_event_registration_request_id on registration_events(registration_request_id);
create index registration_event_initiator_id on registration_events(initiator_id);
create index registration_event_type on registration_events(type);

comment on column registration_events.registration_request_id is 'All registration events should be connected to some registration request';
comment on column registration_events.initiator_id is 'Id of user who initiated that event (eg. called with new member, added them to mailing list, etc)';
comment on column registration_events.type is 'Type is enum of predefined steps of user onboarding';
comment on column registration_events.note is 'Note is free text that can initiator add to any registration event. Comment, practical info, etc.';
