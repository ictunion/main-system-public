use super::RegistrationRequest;
use crate::data::{self, Id};
use crate::db::DbPool;
use crate::media::ImageData;
use crate::server::{IpAddress, UserAgent};

pub async fn create_join_request<'r>(
    pool: &DbPool,
    ip_addr: IpAddress,
    user_agent: UserAgent<'r>,
    confirmation_token: String,
    user: &RegistrationRequest<'r>,
) -> sqlx::Result<Id<data::RegistrationRequest>> {
    sqlx::query_scalar!(
        r#"
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
"#,
        user.email,
        user.first_name,
        user.last_name,
        user.date_of_birth,
        user.address,
        user.city,
        user.postal_code,
        user.phone_number,
        user.company_name,
        user.occupation,
        user.local,
        ip_addr as _,
        user_agent as _,
        "website_join_form",
        confirmation_token,
    )
    .fetch_one(pool)
    .await
    .map(Id::from)
}

pub async fn confirm_email(pool: &DbPool, code: &str) -> sqlx::Result<String> {
    sqlx::query_scalar!(
        r#"
UPDATE registration_requests AS m
SET   confirmed_at = NOW()
    , confirmation_token = NULL
WHERE confirmation_token = $1
RETURNING m.registration_local
"#,
        code
    )
    .fetch_one(pool)
    .await
}

pub async fn create_signature_file(
    pool: &DbPool,
    reg_id: Id<data::RegistrationRequest>,
    image: &ImageData,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
WITH rows AS
( INSERT INTO files
    ( name
    , file_type
    , data
    ) VALUES ('signature', $1, $2)
    RETURNING id
)
INSERT INTO registration_requests_files
    ( registration_request_id
    , file_id
    )
    SELECT $3 as registration_request_id, id
    FROM rows
"#,
        &image.image_type,
        image.to_vec(),
        reg_id as _,
    )
    .execute(pool)
    .await?;
    Ok(())
}
