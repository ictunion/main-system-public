use uuid::Uuid;

use super::{
    MemberDetail, MemberOffice, MemberOfficePatch, MemberStatusData, MemberSummary, NewMember,
    NewMemberOffice, Note, Occupation, UpdateMember,
};
use crate::api::files::FileInfo;
use crate::data::{FileType, Id, Member, MemberNumber, Office};
use crate::db::DbPool;

pub async fn list_summaries(pool: &DbPool) -> sqlx::Result<Vec<MemberSummary>> {
    sqlx::query_as!(
        MemberSummary,
        r#"
SELECT m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS "company_names!: Vec<Option<String>>"
    , m.created_at
    , ARRAY(SELECT mw.workplace_id FROM members_workplaces mw WHERE mw.member_id = m.id) AS "workplace_ids!: Vec<Uuid>"
    , (SELECT bool_or(mw.became_representative_at IS NOT NULL) FROM members_workplaces mw WHERE mw.member_id = m.id) AS "is_representative?"
    , m.sub
FROM members AS m
LEFT JOIN occupations o ON o.member_id = m.id
GROUP BY m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , m.created_at
    , m.sub
ORDER BY m.member_number DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_past_summaries(pool: &DbPool) -> sqlx::Result<Vec<MemberSummary>> {
    sqlx::query_as!(
        MemberSummary,
        r#"
SELECT m.id AS "id!"
    , m.member_number AS "member_number!"
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS "company_names!: Vec<Option<String>>"
    , m.created_at AS "created_at!"
    , ARRAY(SELECT mw.workplace_id FROM members_workplaces mw WHERE mw.member_id = m.id) AS "workplace_ids!: Vec<Uuid>"
    , (SELECT bool_or(mw.became_representative_at IS NOT NULL) FROM members_workplaces mw WHERE mw.member_id = m.id) AS "is_representative?"
    , m.sub
FROM members_past AS m
LEFT JOIN occupations o ON o.member_id = m.id
GROUP BY m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , m.created_at
    , m.sub
ORDER BY m.member_number DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_new_summaries(pool: &DbPool) -> sqlx::Result<Vec<MemberSummary>> {
    sqlx::query_as!(
        MemberSummary,
        r#"
SELECT m.id AS "id!"
    , m.member_number AS "member_number!"
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS "company_names!: Vec<Option<String>>"
    , m.created_at AS "created_at!"
    , ARRAY(SELECT mw.workplace_id FROM members_workplaces mw WHERE mw.member_id = m.id) AS "workplace_ids!: Vec<Uuid>"
    , (SELECT bool_or(mw.became_representative_at IS NOT NULL) FROM members_workplaces mw WHERE mw.member_id = m.id) AS "is_representative?"
    , m.sub
FROM members_new AS m
LEFT JOIN occupations o ON o.member_id = m.id
GROUP BY m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , m.created_at
    , m.sub
ORDER BY m.member_number DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_current_summaries(pool: &DbPool) -> sqlx::Result<Vec<MemberSummary>> {
    sqlx::query_as!(
        MemberSummary,
        r#"
SELECT m.id AS "id!"
    , m.member_number AS "member_number!"
    , m.first_name
    , m.last_name
    , m.email
    , m.note
    , m.phone_number
    , m.city
    , m.language
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS "company_names!: Vec<Option<String>>"
    , m.created_at AS "created_at!"
    , ARRAY(SELECT mw.workplace_id FROM members_workplaces mw WHERE mw.member_id = m.id) AS "workplace_ids!: Vec<Uuid>"
    , (SELECT bool_or(mw.became_representative_at IS NOT NULL) FROM members_workplaces mw WHERE mw.member_id = m.id) AS "is_representative?"
    , m.sub
FROM members_current AS m
LEFT JOIN occupations o ON o.member_id = m.id
GROUP BY m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.language
    , m.left_at
    , m.created_at
    , m.sub
ORDER BY m.member_number DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn create_member<'a, E>(
    executor: E,
    member_number: MemberNumber,
    new_member: &NewMember,
) -> sqlx::Result<MemberSummary>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberSummary,
        r#"
INSERT INTO members
    ( member_number
    , email
    , first_name
    , last_name
    , language
    , date_of_birth
    , address
    , city
    , postal_code
    , phone_number
    )
VALUES
    ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
RETURNING id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , note
    , city
    , language
    , left_at
    , created_at
    , ARRAY[]::text[] AS "company_names!: Vec<Option<String>>"
    , ARRAY[]::uuid[] AS "workplace_ids!: Vec<Uuid>"
    , NULL::boolean AS "is_representative?"
    , sub
"#,
        member_number as _,
        new_member.email.as_deref(),
        new_member.first_name.as_deref(),
        new_member.last_name.as_deref(),
        new_member.language as _,
        new_member.date_of_birth,
        new_member.address.as_deref(),
        new_member.city.as_deref(),
        new_member.postal_code.as_deref(),
        new_member.phone_number.as_deref(),
    )
    .fetch_one(executor)
    .await
}

pub async fn detail<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
SELECT id
    , member_number
    , first_name
    , last_name
    , date_of_birth
    , email
    , phone_number
    , note
    , address
    , city
    , postal_code
    , language
    , registration_request_id as application_id
    , left_at
    , onboarding_finished_at
    , created_at
    , (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
    , (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
    , sub
FROM members
WHERE members.id = $1
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn get_next_member_number<'a, E>(executor: E) -> sqlx::Result<MemberNumber>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_scalar!(
        r#"
SELECT COALESCE
    (1 + (
        SELECT member_number FROM members m
        ORDER BY member_number DESC
        LIMIT 1)
    , 1) AS "value!"
"#
    )
    .fetch_one(executor)
    .await
    .map(MemberNumber::from)
}

pub async fn list_member_files<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<Vec<FileInfo>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        FileInfo,
        r#"
SELECT f.id
, f.name
, f.file_type AS "file_type: FileType"
, f.created_at
FROM members_files AS mf
INNER JOIN files AS f ON f.id = mf.file_id
WHERE mf.member_id = $1
ORDER BY f.created_at DESC
"#,
        id as _
    )
    .fetch_all(executor)
    .await
}

pub async fn list_occupations<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<Vec<Occupation>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        Occupation,
        r#"
SELECT id
, company_name
, position
, source
, created_at
FROM occupations
WHERE member_id = $1
AND deleted_at IS NULL
ORDER BY created_at DESC
"#,
        id as _
    )
    .fetch_all(executor)
    .await
}

/// Only the member's current occupation (most recent row). Served to workplace
/// executives, who may see where a member works now but not their full
/// employment history.
pub async fn latest_occupation<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<Vec<Occupation>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        Occupation,
        r#"
SELECT id
, company_name
, position
, source
, created_at
FROM occupations
WHERE member_id = $1
AND deleted_at IS NULL
ORDER BY created_at DESC
LIMIT 1
"#,
        id as _
    )
    .fetch_all(executor)
    .await
}

pub async fn create_occupation<'a, E>(
    executor: E,
    member_id: Id<Member>,
    company_name: &str,
    position: &str,
    source: &str,
) -> sqlx::Result<Occupation>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        Occupation,
        r#"
INSERT INTO occupations
    ( member_id
    , company_name
    , position
    , source
    )
VALUES
    ( $1, $2, $3, $4 )
RETURNING id
    , company_name
    , position
    , source
    , created_at
"#,
        member_id as _,
        company_name,
        position,
        source,
    )
    .fetch_one(executor)
    .await
}

pub async fn delete_occupation<'a, E>(
    executor: E,
    member_id: Id<Member>,
    occupation_id: Id<Occupation>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE occupations
SET deleted_at = now()
WHERE id = $1
AND member_id = $2
AND deleted_at IS NULL
RETURNING id
"#,
        occupation_id as _,
        member_id as _,
    )
    .fetch_one(executor)
    .await?;

    Ok(())
}

/// Offices whose term hasn't ended yet. `term_ends_on` is the last day in
/// office, so an office ending today is still current.
pub async fn list_active_offices(pool: &DbPool) -> sqlx::Result<Vec<MemberOffice>> {
    sqlx::query_as!(
        MemberOffice,
        r#"
SELECT o.id
    , o.member_id
    , m.member_number
    , m.first_name
    , m.last_name
    , o.title
    , o.elected_by
    , o.elected_on
    , o.term_ends_on
    , o.created_at
    , o.updated_at
FROM member_offices AS o
INNER JOIN members AS m ON m.id = o.member_id
WHERE o.deleted_at IS NULL
AND (o.term_ends_on IS NULL OR o.term_ends_on >= CURRENT_DATE)
ORDER BY o.elected_on DESC, m.member_number
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_past_offices(pool: &DbPool) -> sqlx::Result<Vec<MemberOffice>> {
    sqlx::query_as!(
        MemberOffice,
        r#"
SELECT o.id
    , o.member_id
    , m.member_number
    , m.first_name
    , m.last_name
    , o.title
    , o.elected_by
    , o.elected_on
    , o.term_ends_on
    , o.created_at
    , o.updated_at
FROM member_offices AS o
INNER JOIN members AS m ON m.id = o.member_id
WHERE o.deleted_at IS NULL
AND o.term_ends_on < CURRENT_DATE
ORDER BY o.term_ends_on DESC, m.member_number
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_office<'a, E>(executor: E, id: Id<Office>) -> sqlx::Result<MemberOffice>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberOffice,
        r#"
SELECT o.id
    , o.member_id
    , m.member_number
    , m.first_name
    , m.last_name
    , o.title
    , o.elected_by
    , o.elected_on
    , o.term_ends_on
    , o.created_at
    , o.updated_at
FROM member_offices AS o
INNER JOIN members AS m ON m.id = o.member_id
WHERE o.id = $1
AND o.deleted_at IS NULL
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

/// Ends the member's current offices today, e.g. when they leave the union.
/// Offices elected for a future date are left alone.
pub async fn end_current_offices<'a, E>(executor: E, member_id: Id<Member>) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE member_offices
SET term_ends_on = CURRENT_DATE
    , updated_at = NOW()
WHERE member_id = $1
AND deleted_at IS NULL
AND elected_on <= CURRENT_DATE
AND (term_ends_on IS NULL OR term_ends_on > CURRENT_DATE)
"#,
        member_id as _,
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Selecting from `members` makes an unknown member id, or one who left the
/// union, come back as `RowNotFound` (404) rather than a foreign key violation
/// (500).
pub async fn create_office<'a, E>(executor: E, office: &NewMemberOffice) -> sqlx::Result<Id<Office>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_scalar!(
        r#"
INSERT INTO member_offices
    ( member_id
    , title
    , elected_by
    , elected_on
    , term_ends_on
    )
SELECT id, $2, $3, $4, $5
FROM members
WHERE id = $1
AND left_at IS NULL
RETURNING id
"#,
        office.member_id as _,
        office.title,
        office.elected_by,
        office.elected_on,
        office.term_ends_on,
    )
    .fetch_one(executor)
    .await
    .map(Id::from)
}

/// Only a current office can be updated; a past or deleted one comes back as
/// `RowNotFound`. `member_id` and `deleted_at` are never changed here.
pub async fn update_office<'a, E>(
    executor: E,
    id: Id<Office>,
    patch: &MemberOfficePatch,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE member_offices
SET title = COALESCE($2, title)
    , elected_by = COALESCE($3, elected_by)
    , elected_on = COALESCE($4, elected_on)
    , term_ends_on = CASE WHEN $5::BOOLEAN THEN $6::DATE ELSE term_ends_on END
    , updated_at = NOW()
WHERE id = $1
AND deleted_at IS NULL
AND (term_ends_on IS NULL OR term_ends_on >= CURRENT_DATE)
RETURNING id
"#,
        id as _,
        patch.title,
        patch.elected_by,
        patch.elected_on,
        patch.term_ends_on.is_some(),
        patch.term_ends_on.flatten(),
    )
    .fetch_one(executor)
    .await?;

    Ok(())
}

/// Soft delete; an already deleted record comes back as `RowNotFound`.
pub async fn delete_office<'a, E>(executor: E, id: Id<Office>) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE member_offices
SET deleted_at = NOW()
WHERE id = $1
AND deleted_at IS NULL
RETURNING id
"#,
        id as _,
    )
    .fetch_one(executor)
    .await?;

    Ok(())
}

pub async fn assign_member_oid_sub<'a, E>(
    executor: E,
    id: Id<Member>,
    uuid: Uuid,
) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
UPDATE members
SET sub = $2
WHERE members.id = $1
RETURNING members.id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
, (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
, (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
, sub
"#,
        id as _,
        uuid,
    )
    .fetch_one(executor)
    .await
}

pub async fn get_status_data<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<MemberStatusData>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberStatusData,
        r#"
SELECT sub, left_at, onboarding_finished_at
FROM members
WHERE id = $1
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn set_onboarding_finished<'a, E>(
    executor: E,
    id: Id<Member>,
) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
UPDATE members
SET onboarding_finished_at = NOW()
WHERE members.id = $1
RETURNING members.id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
, (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
, (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
, sub
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

pub async fn update_member_note<'a, E>(
    executor: E,
    id: Id<Member>,
    new_note: &Note,
) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
UPDATE members
SET note = $2
WHERE members.id = $1
RETURNING members.id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
, (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
, (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
, sub
"#,
        id as _,
        new_note.note.as_deref(),
    )
    .fetch_one(executor)
    .await
}

pub async fn update_member<'a, E>(
    executor: E,
    id: Id<Member>,
    updated_member: UpdateMember,
) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
UPDATE members
SET first_name = $2
    , last_name = $3
    , date_of_birth = $4
    , email = $5
    , phone_number = $6
    , note = $7
    , address = $8
    , city = $9
    , postal_code = $10
    , language = $11
WHERE members.id = $1
RETURNING members.id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
, (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
, (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
, sub
"#,
        id as _,
        updated_member.first_name,
        updated_member.last_name,
        updated_member.date_of_birth,
        updated_member.email,
        updated_member.phone_number,
        updated_member.note,
        updated_member.address,
        updated_member.city,
        updated_member.postal_code,
        updated_member.language as _,
    )
    .fetch_one(executor)
    .await
}

// This doesn't really delete member from the database
// We're just adding left_at flag to the data and removing keycloak sub token
pub async fn remove_member<'a, E>(executor: E, id: Id<Member>) -> sqlx::Result<MemberDetail>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        MemberDetail,
        r#"
UPDATE members
SET left_at = NOW()
  , sub = NULL
WHERE members.id = $1
RETURNING members.id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
, (SELECT workplace_id FROM members_workplaces WHERE member_id = members.id) AS "workplace_id?"
, (SELECT became_representative_at IS NOT NULL FROM members_workplaces WHERE member_id = members.id) AS "is_representative?"
, sub
"#,
        id as _
    )
    .fetch_one(executor)
    .await
}

/// Whether this member currently belongs to any of the given workplaces.
///
/// Used to check a single member read against a workplace executive's scope.
/// Past members are excluded, matching `list_members_of_workplaces`: someone who
/// has left is not on the committee's roster and should not be reachable through
/// a stale link either.
pub async fn is_member_of_workplaces(
    pool: &DbPool,
    id: Id<Member>,
    workplace_ids: &[Uuid],
) -> sqlx::Result<bool> {
    let found = sqlx::query_scalar!(
        r#"
SELECT EXISTS (
    SELECT 1
    FROM members m
    JOIN members_workplaces mw ON mw.member_id = m.id
    WHERE m.id = $1
      AND mw.workplace_id = ANY($2)
      AND m.left_at IS NULL
) AS "exists!: bool"
"#,
        id as _,
        workplace_ids
    )
    .fetch_one(pool)
    .await?;

    Ok(found)
}
