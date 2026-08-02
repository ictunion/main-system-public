use crate::{
    data::{Member, MemberNumber},
    db::DbPool,
};

use super::{
    ApplicationAcceptedSummary, ApplicationDetail, ApplicationInvalidSummary,
    ApplicationProcessingSummary, ApplicationRejectedSummary, ApplicationStatusData,
    ApplicationSummary, ApplicationUnverifiedSummary, FileInfo, Note,
};
use crate::data::{Id, RegistrationRequest};

pub async fn list_summaries(pool: &DbPool) -> sqlx::Result<Vec<ApplicationSummary>> {
    sqlx::query_as!(
        ApplicationSummary,
        r#"
SELECT id
, email
, first_name
, last_name
, phone_number
, note
, city
, company_name
, registration_local
, created_at
FROM registration_requests
ORDER BY created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unverified_summaries(
    pool: &DbPool,
) -> sqlx::Result<Vec<ApplicationUnverifiedSummary>> {
    sqlx::query_as!(
        ApplicationUnverifiedSummary,
        r#"
SELECT id AS "id!"
, email
, first_name
, last_name
, phone_number
, note
, city
, company_name
, registration_local AS "registration_local!"
, created_at AS "created_at!"
, verification_sent_at
FROM registration_requests_unverified
ORDER BY created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_processing_summaries(
    pool: &DbPool,
) -> sqlx::Result<Vec<ApplicationProcessingSummary>> {
    sqlx::query_as!(
        ApplicationProcessingSummary,
        r#"
SELECT id AS "id!"
, email
, first_name
, last_name
, phone_number
, note
, city
, company_name
, registration_local AS "registration_local!"
, created_at AS "created_at!"
, confirmed_at AS "confirmed_at!"
FROM registration_requests_processing
ORDER BY confirmed_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_accepted_summaries(
    pool: &DbPool,
) -> sqlx::Result<Vec<ApplicationAcceptedSummary>> {
    sqlx::query_as!(
        ApplicationAcceptedSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, rr.note
, rr.city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, m.created_at AS "accepted_at!"
, m.id AS "member_id!"
FROM registration_requests_accepted as rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
ORDER BY rr.created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_rejected_summaries(
    pool: &DbPool,
) -> sqlx::Result<Vec<ApplicationRejectedSummary>> {
    sqlx::query_as!(
        ApplicationRejectedSummary,
        r#"
SELECT id AS "id!"
, email
, first_name
, last_name
, phone_number
, note
, city
, company_name
, registration_local AS "registration_local!"
, created_at AS "created_at!"
, rejected_at AS "rejected_at!"
FROM registration_requests_rejected
ORDER BY created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_invalid_summaries(pool: &DbPool) -> sqlx::Result<Vec<ApplicationInvalidSummary>> {
    sqlx::query_as!(
        ApplicationInvalidSummary,
        r#"
SELECT id AS "id!"
, email
, first_name
, last_name
, phone_number
, note
, city
, company_name
, registration_local AS "registration_local!"
, created_at AS "created_at!"
, invalidated_at AS "invalidated_at!"
FROM registration_requests_invalid
ORDER BY created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationDetail,
        r#"
SELECT rr.id
, rr.email
, rr.first_name
, rr.last_name
, rr.date_of_birth
, rr.phone_number
, rr.note
, rr.city
, rr.address
, rr.postal_code
, rr.occupation
, rr.company_name
, rr.verification_sent_at
, rr.confirmed_at
, rr.registration_ip AS "registration_ip: crate::server::IpAddress"
, rr.registration_local
, rr.registration_user_agent
, rr.registration_source
, rr.rejected_at
, rr.invalidated_at
, rr.created_at
, rr.registration_local AS language
, m.created_at AS accepted_at
FROM registration_requests AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
WHERE rr.id = $1
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn list_application_files(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<Vec<FileInfo>> {
    sqlx::query_as!(
        FileInfo,
        r#"
SELECT f.id
, f.name
, f.file_type
, f.created_at
FROM registration_requests_files AS rrf
INNER JOIN files AS f ON f.id = rrf.file_id
WHERE rrf.registration_request_id = $1
ORDER BY f.created_at DESC
"#,
        id as _
    )
    .fetch_all(pool)
    .await
}

pub async fn reject_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    // We can hardcode null because we know rejected application can't belong to user
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET rejected_at = NOW()
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn invalidate_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    // We can hardcode null because we know rejected application can't belong to user
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET invalidated_at = NOW()
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn unreject_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET rejected_at = NULL
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn uninvalidate_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET invalidated_at = NULL
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn verify_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET   confirmed_at = NOW()
    , confirmation_token = NULL
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn get_application_status_data<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<ApplicationStatusData>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationStatusData,
        r#"
SELECT rr.id
, rr.created_at
, rr.confirmed_at
, rr.rejected_at
, rr.invalidated_at
, m.created_at AS accepted_at
, m.id AS "member_id?"
FROM registration_requests AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
WHERE rr.id = $1
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

/// TODO: this should return member once we have it type for it
pub async fn create_new_member<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
    number: MemberNumber,
) -> sqlx::Result<Id<Member>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_scalar!(
        r#"
INSERT INTO members AS m
( member_number
, email
, first_name
, last_name
, language
, date_of_birth
, note
, address
, city
, postal_code
, phone_number
, registration_request_id
)
SELECT $2
, email
, first_name
, last_name
, registration_local
, date_of_birth
, note
, address
, city
, postal_code
, phone_number
, $1
FROM registration_requests as rr
WHERE rr.id = $1
RETURNING m.id
"#,
        id as _,
        number as _,
    )
    .fetch_one(executor)
    .await
    .map(Id::from)
}

pub async fn attach_files_to_member<'a, E>(
    executor: E,
    registration_id: Id<RegistrationRequest>,
    member_id: Id<Member>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
INSERT INTO members_files
( member_id
, file_id
)
SELECT $2, file_id FROM registration_requests_files
WHERE registration_request_id = $1
"#,
        registration_id as _,
        member_id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn attach_occupation<'a, E>(
    executor: E,
    registration_id: Id<RegistrationRequest>,
    member_id: Id<Member>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
INSERT INTO occupations
( member_id
, company_name
, position
)
SELECT $2, rr.company_name, rr.occupation
FROM registration_requests as rr
WHERE rr.id = $1
"#,
        registration_id as _,
        member_id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn dangerous_hard_delete_application_data<'a, E>(
    executor: E,
    registration_id: Id<RegistrationRequest>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
WITH file_ids AS
    (DELETE FROM registration_requests_files
        WHERE registration_request_id = $1
        RETURNING file_id)
DELETE FROM files WHERE id IN (SELECT file_id FROM file_ids)
"#,
        registration_id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn dangerous_hard_delete_application<'a, E>(
    executor: E,
    registration_id: Id<RegistrationRequest>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM registration_requests
    WHERE id = $1
"#,
        registration_id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn update_registration_note(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
    new_note: &Note,
) -> sqlx::Result<ApplicationDetail> {
    sqlx::query_as!(
        ApplicationDetail,
        r#"
UPDATE registration_requests
SET note = $2
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, note
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip AS "registration_ip: crate::server::IpAddress"
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, invalidated_at
, created_at
, registration_local AS language
, NULL::timestamptz AS accepted_at
"#,
        id as _,
        new_note.note.as_deref(),
    )
    .fetch_one(pool)
    .await
}
