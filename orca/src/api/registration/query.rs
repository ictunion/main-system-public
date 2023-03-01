use std::net::SocketAddr;

use super::RegistrationRequest;
use crate::db::{Query, QueryAs};
use crate::media::ImageData;
use crate::server::UserAgent;
use crate::data::{Id, Member};

pub fn create_join_request<'r>(
    remote_addr: SocketAddr,
    user_agent: UserAgent<'r>,
    confirmation_token: String,
    user: &RegistrationRequest<'r>,
) -> QueryAs<'r, (Id<Member>,)> {
    sqlx::query_as(
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
) values ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
returning id
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

pub fn confirm_email(code: &'_ str) -> QueryAs<'_, (String,)> {
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

pub fn create_singature_file<'r>(user_id: Id<Member>, image: &'r ImageData) -> Query<'r> {
    sqlx::query(
        r#"
insert into files
( name
, file_type
, data
, user_id
) values ('signature', $1, $2, $3)
"#,
    )
    .bind(&image.image_type)
    .bind(image.to_vec())
    .bind(user_id)
}
