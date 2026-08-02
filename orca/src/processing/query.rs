use uuid::Uuid;

use crate::db::DbPool;
use crate::server::oid;

use super::RegistrationDetails;
use crate::data::{Id, Member, RegistrationRequest};

pub async fn query_registration(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<RegistrationDetails> {
    sqlx::query_as!(
        RegistrationDetails,
        r#"
SELECT id
, first_name
, last_name
, date_of_birth
, phone_number
, email AS "email!"
, address
, city
, postal_code
, company_name
, occupation
, confirmation_token
, registration_local
FROM registration_requests WHERE id = $1
"#,
        id as _
    )
    .fetch_one(pool)
    .await
}

pub async fn insert_registration_pdf(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
    data: &Vec<u8>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
WITH rows AS
( INSERT INTO files
    ( name
    , file_type
    , data
    ) VALUES ('registration', 'pdf', $1)
    RETURNING id
)
INSERT INTO registration_requests_files
    ( registration_request_id
    , file_id
    )
    SELECT $2 as registration_request_id, id
    FROM rows
"#,
        data,
        id as _,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn track_verification_sent_at(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
UPDATE registration_requests AS m
SET verification_sent_at = now()
WHERE id = $1
"#,
        id as _,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_members_without_sub(pool: &DbPool) -> sqlx::Result<Vec<Id<Member>>> {
    sqlx::query_scalar!(r#"SELECT id FROM members WHERE sub IS NULL AND left_at IS NULL"#)
        .fetch_all(pool)
        .await
        .map(|ids| ids.into_iter().map(Id::from).collect())
}

pub async fn get_member_for_oid(pool: &DbPool, id: Id<Member>) -> sqlx::Result<oid::User> {
    sqlx::query_as!(
        oid::User,
        r#"SELECT NULL::text as id, first_name, last_name, email FROM members WHERE id = $1"#,
        id as _
    )
    .fetch_one(pool)
    .await
}

pub async fn assign_member_oid_sub(pool: &DbPool, id: Id<Member>, uuid: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE members SET sub = $2 WHERE id = $1"#,
        id as _,
        uuid,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_registration_pdf(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<Vec<u8>> {
    sqlx::query_scalar!(
        r#"
SELECT f.data AS "data!" FROM registration_requests_files rrf
INNER JOIN files f
ON rrf.file_id = f.id
WHERE registration_request_id = $1
    AND file_type = 'pdf'
    AND name = 'registration'
ORDER BY f.created_at DESC
LIMIT 1
"#,
        id as _
    )
    .fetch_one(pool)
    .await
}
