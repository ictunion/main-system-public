drop index member_confirmation_token;
create unique index member_confirmation_token on members(confirmation_token);
