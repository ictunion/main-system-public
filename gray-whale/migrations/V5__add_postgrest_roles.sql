CREATE ROLE postgrest_authenticator noinherit login;
CREATE ROLE members_manager nologin;

GRANT members_manager TO postgrest_authenticator;

-- Grant permissions to members_manager
GRANT USAGE ON SCHEMA public TO members_manager;
GRANT ALL ON public.members TO members_manager;
GRANT ALL ON public.workplaces TO members_manager;
GRANT ALL ON public.occupations TO members_manager;
GRANT SELECT ON public.registration_events TO members_manager;
GRANT SELECT ON public.registration_requests TO members_manager;
GRANT SELECT ON public.files TO members_manager;
