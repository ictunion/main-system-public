use std::net::SocketAddr;

use super::RegistrationRequest;
use crate::db::{Query, QueryAs};
use crate::server::UserAgent;

pub fn create_join_request<'r>(
    remote_addr: SocketAddr,
    user_agent: UserAgent<'r>,
    confirmation_token: String,
    user: &RegistrationRequest<'r>,
) -> Query<'r> {
    sqlx::query(
        "
insert into members
( email
, first_name
, last_name
, date_of_birth
, address
, city
, postal_code
, phone_number
, company_name
, occupation
, registration_local
, registration_ip
, registration_user_agent
, registration_source
, confirmation_token
)
values
( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
",
    )
    .bind(user.email)
    .bind(user.first_name)
    .bind(user.last_name)
    .bind(user.date_of_birth)
    .bind(user.address)
    .bind(user.city)
    .bind(user.postal_code)
    .bind(user.phone_number)
    .bind(user.company_name)
    .bind(user.occupation)
    .bind(user.local)
    .bind(remote_addr.ip())
    .bind(user_agent)
    .bind("website_join_form")
    .bind(confirmation_token)
}

pub fn confirm_email(code: &'_ str) -> QueryAs<'_, (Option<String>, )> {
    sqlx::query_as(
        "
update members as m
set confirmed_at = now()
  , confirmation_token = NULL
where
  confirmation_token = $1
returning
  m.registration_local
",
    )
    .bind(code)
}
