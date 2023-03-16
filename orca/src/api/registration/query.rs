use super::RegistrationRequest;
use crate::data::{self, Id};
use crate::db::{Query, QueryAs};
use crate::media::ImageData;
use crate::server::{IpAddress, UserAgent};

pub fn create_join_request<'r>(
    ip_addr: IpAddress,
    user_agent: UserAgent<'r>,
    confirmation_token: String,
    user: &RegistrationRequest<'r>,
) -> QueryAs<'r, (Id<data::RegistrationRequest>,)> {
    sqlx::query_as(
        "
INSERT INTO registration_requests
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
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
RETURNING id
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
    .bind(ip_addr)
    .bind(user_agent)
    .bind("website_join_form")
    .bind(confirmation_token)
}

pub fn confirm_email(code: &'_ str) -> QueryAs<'_, (Id<data::RegistrationRequest>, String)> {
    sqlx::query_as(
        "
UPDATE registration_requests AS m
SET   confirmed_at = now()
    , confirmation_token = NULL
WHERE confirmation_token = $1
RETURNING m.id, m.registration_local
",
    )
    .bind(code)
}

pub fn create_singature_file(
    reg_id: Id<data::RegistrationRequest>,
    image: &'_ ImageData,
) -> Query<'_> {
    sqlx::query(
        r#"
INSERT INTO files
( name
, file_type
, data
, registration_request_id
) VALUES ('signature', $1, $2, $3)
"#,
    )
    .bind(&image.image_type)
    .bind(image.to_vec())
    .bind(reg_id)
}
