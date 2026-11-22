use crate::{
    data::{Member, MemberNumber},
    db::DbPool,
};
use uuid::Uuid;

use super::{
    ApplicationAcceptedSummary, ApplicationDetail, ApplicationInvalidSummary,
    ApplicationProcessingSummary, ApplicationRejectedSummary, ApplicationStatusData,
    ApplicationSummary, ApplicationUnverifiedSummary, ApplicationWorkplaceRow, FileInfo, Note,
};
use crate::data::{FileType, Id, RegistrationRequest, Workplace};

/// List queries take `workplace_ids`: `None` lists everything (staff),
/// `Some(ids)` only applications assigned to those workplaces. The workplace
/// filtered listing is the executive committee view, so it also drops `note`
/// and `city`, mirroring `ApplicationDetail::redact_for_workplace_executive`.
pub async fn list_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationSummary>> {
    sqlx::query_as!(
        ApplicationSummary,
        r#"
SELECT rr.id
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local
, rr.created_at
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests AS rr
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.created_at DESC
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_unverified_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationUnverifiedSummary>> {
    sqlx::query_as!(
        ApplicationUnverifiedSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, rr.verification_sent_at
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests_unverified AS rr
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.created_at DESC
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_processing_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationProcessingSummary>> {
    sqlx::query_as!(
        ApplicationProcessingSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, rr.confirmed_at AS "confirmed_at!"
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests_processing AS rr
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.confirmed_at DESC
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_accepted_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationAcceptedSummary>> {
    sqlx::query_as!(
        ApplicationAcceptedSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, m.created_at AS "accepted_at!"
, m.id AS "member_id!"
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests_accepted AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.created_at DESC
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_rejected_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationRejectedSummary>> {
    sqlx::query_as!(
        ApplicationRejectedSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, rr.rejected_at AS "rejected_at!"
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests_rejected AS rr
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.created_at DESC
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_invalid_summaries(
    pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<ApplicationInvalidSummary>> {
    sqlx::query_as!(
        ApplicationInvalidSummary,
        r#"
SELECT rr.id AS "id!"
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, CASE WHEN $1::uuid[] IS NULL THEN rr.note END AS note
, CASE WHEN $1::uuid[] IS NULL THEN rr.city END AS city
, rr.company_name
, rr.registration_local AS "registration_local!"
, rr.created_at AS "created_at!"
, rr.invalidated_at AS "invalidated_at!"
, rrw.approved_by_reps_at AS "approved_by_reps_at?"
FROM registration_requests_invalid AS rr
LEFT JOIN registration_requests_workplaces AS rrw ON rrw.registration_request_id = rr.id
WHERE ($1::uuid[] IS NULL OR rrw.workplace_id = ANY($1))
ORDER BY rr.created_at DESC
"#,
        workplace_ids
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
, m.created_at AS "accepted_at?"
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
, f.file_type AS "file_type: FileType"
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
, m.created_at AS "accepted_at?"
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
    source: &str,
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
, source
)
SELECT $2, rr.company_name, rr.occupation, $3
FROM registration_requests as rr
WHERE rr.id = $1
"#,
        registration_id as _,
        member_id as _,
        source,
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

/// The workplace an application is assigned to, if any.
pub async fn get_application_workplace<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<Option<ApplicationWorkplaceRow>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        ApplicationWorkplaceRow,
        r#"
SELECT workplace_id AS "workplace_id: Id<Workplace>"
, approved_by_reps_at
FROM registration_requests_workplaces
WHERE registration_request_id = $1
"#,
        id as _
    )
    .fetch_optional(executor)
    .await
}

/// Assign an application to a workplace, or move it to another one.
///
/// A move resets `assigned_at` and clears `approved_by_reps_at`: the approval was the
/// previous workplace's opinion. Re-assigning to the same workplace is a no-op
/// (0 rows) so it does not wipe an existing approval.
pub async fn assign_workplace<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
    workplace_id: Id<Workplace>,
) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    Ok(sqlx::query!(
        r#"
INSERT INTO registration_requests_workplaces
    ( registration_request_id
    , workplace_id
    )
VALUES ( $1, $2 )
ON CONFLICT ( registration_request_id )
DO UPDATE
SET workplace_id = EXCLUDED.workplace_id
    , assigned_at = now()
    , approved_by_reps_at = NULL
WHERE registration_requests_workplaces.workplace_id <> EXCLUDED.workplace_id
"#,
        id as _,
        workplace_id as _,
    )
    .execute(executor)
    .await?
    .rows_affected())
}

pub async fn unassign_workplace<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    Ok(sqlx::query!(
        r#"DELETE FROM registration_requests_workplaces WHERE registration_request_id = $1"#,
        id as _,
    )
    .execute(executor)
    .await?
    .rows_affected())
}

/// Whether the application is assigned to one of the given workplaces.
pub async fn is_application_of_workplaces(
    pool: &DbPool,
    id: Id<RegistrationRequest>,
    workplace_ids: &[Uuid],
) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"
SELECT EXISTS (
    SELECT 1
    FROM registration_requests_workplaces
    WHERE registration_request_id = $1
      AND workplace_id = ANY($2)
) AS "exists!: bool"
"#,
        id as _,
        workplace_ids
    )
    .fetch_one(pool)
    .await
}

/// Record the workplace executive committee's approval. Keeps the first
/// approval time if approved again.
pub async fn approve_application<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE registration_requests_workplaces
SET approved_by_reps_at = now()
WHERE registration_request_id = $1
  AND approved_by_reps_at IS NULL
"#,
        id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// On reject: a rejection supersedes the reps' recommendation, so an
/// unrejected application has to be approved again.
pub async fn clear_application_approval<'a, E>(
    executor: E,
    id: Id<RegistrationRequest>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE registration_requests_workplaces
SET approved_by_reps_at = NULL
WHERE registration_request_id = $1
"#,
        id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// On accept: the new member joins the workplace the application was assigned
/// to. The application keeps its assignment as a record.
pub async fn copy_workplace_to_member<'a, E>(
    executor: E,
    registration_id: Id<RegistrationRequest>,
    member_id: Id<Member>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
INSERT INTO members_workplaces
( member_id
, workplace_id
)
SELECT $2, rrw.workplace_id
FROM registration_requests_workplaces AS rrw
WHERE rrw.registration_request_id = $1
ON CONFLICT DO NOTHING
"#,
        registration_id as _,
        member_id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}
