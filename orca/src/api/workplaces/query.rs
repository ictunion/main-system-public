use uuid::Uuid;

use super::{
    MyWorkplaceSummary, NewWorkplace, UpdateWorkplace, WorkplaceMemberSummary, WorkplaceSummary,
};
use crate::api::members::MemberSummary;
use crate::data::{Id, Member, Workplace};
use crate::db::DbPool;

// member count includes also members, who have already left union
// current process is to remove association between past members and workplaces manually
// in the future, it should be done when clicking on "remove member" in member detail
pub async fn list_summaries(pool: &DbPool) -> sqlx::Result<Vec<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
SELECT id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , COUNT(mw.member_id) AS "member_count!: i64"
FROM workplaces
LEFT JOIN members_workplaces mw ON mw.workplace_id = workplaces.id
GROUP BY workplaces.id
    , workplaces.name
    , workplaces.email
    , workplaces.created_at
    , workplaces.keycloak_group_id
    , workplaces.keycloak_executive_group_id
    , workplaces.announced_at
    , workplaces.established_at
    , workplaces.cancelled_at
    , workplaces.newsletter_id
ORDER BY workplaces.created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_active_summaries(pool: &DbPool) -> sqlx::Result<Vec<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
SELECT id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , COUNT(mw.member_id) AS "member_count!: i64"
FROM workplaces
LEFT JOIN members_workplaces mw ON mw.workplace_id = workplaces.id
WHERE workplaces.cancelled_at IS NULL
GROUP BY workplaces.id
    , workplaces.name
    , workplaces.email
    , workplaces.created_at
    , workplaces.keycloak_group_id
    , workplaces.keycloak_executive_group_id
    , workplaces.announced_at
    , workplaces.established_at
    , workplaces.cancelled_at
    , workplaces.newsletter_id
ORDER BY workplaces.created_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_inactive_summaries(pool: &DbPool) -> sqlx::Result<Vec<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
SELECT id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , COUNT(mw.member_id) AS "member_count!: i64"
FROM workplaces
LEFT JOIN members_workplaces mw ON mw.workplace_id = workplaces.id
WHERE workplaces.cancelled_at IS NOT NULL
GROUP BY workplaces.id
    , workplaces.name
    , workplaces.email
    , workplaces.created_at
    , workplaces.keycloak_group_id
    , workplaces.keycloak_executive_group_id
    , workplaces.announced_at
    , workplaces.established_at
    , workplaces.cancelled_at
    , workplaces.newsletter_id
ORDER BY workplaces.cancelled_at DESC
"#
    )
    .fetch_all(pool)
    .await
}

pub async fn detail(pool: &DbPool, id: Id<Workplace>) -> sqlx::Result<WorkplaceSummary> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
SELECT id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , count(mw.member_id) AS "member_count!: i64"
FROM workplaces
LEFT JOIN members_workplaces mw ON mw.workplace_id = workplaces.id
WHERE workplaces.id = $1
GROUP BY workplaces.id
    , workplaces.name
    , workplaces.email
    , workplaces.created_at
    , workplaces.keycloak_group_id
    , workplaces.keycloak_executive_group_id
    , workplaces.announced_at
    , workplaces.established_at
    , workplaces.cancelled_at
    , workplaces.newsletter_id
"#,
        id as _
    )
    .fetch_one(pool)
    .await
}

pub async fn establish_workplace(
    pool: &DbPool,
    id: Id<Workplace>,
) -> sqlx::Result<Option<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
UPDATE workplaces
SET established_at = NOW()
WHERE id = $1
  AND established_at IS NULL
  AND cancelled_at IS NULL
RETURNING id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , (SELECT COUNT(*) FROM members_workplaces mw WHERE mw.workplace_id = id)::bigint AS "member_count!: i64"
"#,
        id as _
    )
    .fetch_optional(pool)
    .await
}

pub async fn announce_workplace(
    pool: &DbPool,
    id: Id<Workplace>,
) -> sqlx::Result<Option<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
UPDATE workplaces
SET announced_at = NOW()
WHERE id = $1
  AND established_at IS NOT NULL
  AND announced_at IS NULL
  AND cancelled_at IS NULL
RETURNING id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , (SELECT COUNT(*) FROM members_workplaces mw WHERE mw.workplace_id = id)::bigint AS "member_count!: i64"
"#,
        id as _
    )
    .fetch_optional(pool)
    .await
}

pub async fn cancel_workplace(
    pool: &DbPool,
    id: Id<Workplace>,
) -> sqlx::Result<Option<WorkplaceSummary>> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
UPDATE workplaces
SET cancelled_at = NOW()
WHERE id = $1
  AND cancelled_at IS NULL
RETURNING id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , (SELECT COUNT(*) FROM members_workplaces mw WHERE mw.workplace_id = id)::bigint AS "member_count!: i64"
"#,
        id as _
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_workplace(
    pool: &DbPool,
    id: Id<Workplace>,
    update: &UpdateWorkplace,
) -> sqlx::Result<WorkplaceSummary> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
UPDATE workplaces
SET newsletter_id = $2
WHERE id = $1
RETURNING id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , (SELECT COUNT(*) FROM members_workplaces mw WHERE mw.workplace_id = id)::bigint AS "member_count!: i64"
"#,
        id as _,
        update.newsletter_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn create_workplace(
    pool: &DbPool,
    new_workplace: &NewWorkplace,
) -> sqlx::Result<WorkplaceSummary> {
    sqlx::query_as!(
        WorkplaceSummary,
        r#"
INSERT INTO workplaces
    ( name
    , email
    , keycloak_group_id
    , keycloak_executive_group_id
    , newsletter_id
    )
VALUES
    ( $1, $2, $3, $4, $5 )
RETURNING id
    , name
    , email
    , created_at
    , keycloak_group_id
    , keycloak_executive_group_id
    , announced_at
    , established_at
    , cancelled_at
    , newsletter_id
    , 0::bigint AS "member_count!: i64"
"#,
        new_workplace.name.as_deref().expect("validated"),
        new_workplace.email.as_deref().expect("validated"),
        new_workplace.keycloak_group_id.expect("validated"),
        new_workplace
            .keycloak_executive_group_id
            .expect("validated"),
        new_workplace.newsletter_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn remove_member_workplace_associations<'a, E>(
    executor: E,
    id: Id<Member>,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"DELETE FROM members_workplaces WHERE member_id = $1"#,
        id as _,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Whether a member is currently a representative of a workplace.
///
/// `false` both when there is no connection at all and when there is one with
/// `became_representative_at` unset -- callers only use this to decide whether a
/// Keycloak executive-group sync is a no-op, and both cases mean "not in the
/// group already".
pub async fn get_is_representative(
    pool: &DbPool,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> sqlx::Result<bool> {
    let is_representative = sqlx::query_scalar!(
        r#"SELECT became_representative_at IS NOT NULL AS "is_representative!" FROM members_workplaces WHERE workplace_id = $1 AND member_id = $2"#,
        workplace_id as _,
        member_id as _,
    )
    .fetch_optional(pool)
    .await?;

    Ok(is_representative.unwrap_or(false))
}

pub async fn create_connection_between_member_and_workplace(
    pool: &DbPool,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
    is_representative: bool,
) -> sqlx::Result<u64> {
    Ok(sqlx::query!(
        r#"
INSERT INTO members_workplaces
    ( workplace_id
    , member_id
    , became_representative_at
    )
VALUES
    ( $1, $2, CASE WHEN $3::boolean THEN now() ELSE NULL END )
ON CONFLICT
    ( workplace_id
    , member_id
    )
DO UPDATE
SET became_representative_at =
    CASE WHEN $3::boolean
         THEN COALESCE(members_workplaces.became_representative_at, now())
         ELSE NULL END
"#,
        workplace_id as _,
        member_id as _,
        is_representative as _,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

pub async fn remove_connection_between_member_and_workplace(
    pool: &DbPool,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> sqlx::Result<u64> {
    Ok(sqlx::query!(
        r#"DELETE FROM members_workplaces WHERE workplace_id=$1 AND member_id=$2"#,
        workplace_id as _,
        member_id as _,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

pub async fn get_all_workplace_members(
    pool: &DbPool,
    workplace_id: Id<Workplace>,
) -> sqlx::Result<Vec<MemberSummary>> {
    sqlx::query_as!(
        MemberSummary,
        r#"
SELECT m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.note
    , m.phone_number
    , m.city
    , m.language
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS "company_names!: Vec<Option<String>>"
    , m.created_at
    , ARRAY(SELECT wp.workplace_id FROM members_workplaces wp WHERE wp.member_id = m.id) AS "workplace_ids!: Vec<Uuid>"
    , (SELECT bool_or(wp.became_representative_at IS NOT NULL) FROM members_workplaces wp WHERE wp.member_id = m.id) AS "is_representative?"
    , m.sub
FROM members AS m
LEFT JOIN occupations o ON o.member_id = m.id
LEFT JOIN members_workplaces mw ON mw.member_id = m.id
WHERE mw.workplace_id = $1 AND left_at IS NULL
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
"#,
        workplace_id as _
    )
    .fetch_all(pool)
    .await
}

/// Workplaces the caller is an executive of, resolved by matching the caller's
/// Keycloak group IDs against `workplaces.keycloak_executive_group_id`.
///
/// An empty `executive_group_ids` yields an empty result, so a caller who is in
/// no executive group can never widen their scope.
pub async fn list_executive_workplaces(
    pool: &DbPool,
    executive_group_ids: &[Uuid],
) -> sqlx::Result<Vec<MyWorkplaceSummary>> {
    sqlx::query_as!(
        MyWorkplaceSummary,
        r#"
SELECT w.id
    , w.name
    , (
        SELECT COUNT(*)
        FROM members_workplaces mw
        JOIN members m ON m.id = mw.member_id
        WHERE mw.workplace_id = w.id AND m.left_at IS NULL
      )::bigint AS "member_count!: i64"
FROM workplaces w
WHERE w.keycloak_executive_group_id = ANY($1)
ORDER BY w.name
"#,
        executive_group_ids
    )
    .fetch_all(pool)
    .await
}

/// Current members of the given workplaces, in the reduced projection exposed
/// to workplace executive committees.
///
/// Callers must pass only workplace IDs already authorized for the caller --
/// this function does no authorization of its own. Past members are excluded:
/// `members_workplaces` rows outlive membership, and a roster of people who
/// have *left* is exposure without an organising purpose.
pub async fn list_members_of_workplaces(
    pool: &DbPool,
    workplace_ids: &[Uuid],
) -> sqlx::Result<Vec<WorkplaceMemberSummary>> {
    sqlx::query_as!(
        WorkplaceMemberSummary,
        r#"
SELECT m.id AS "id: Id<Member>"
    , mw.workplace_id AS "workplace_id: Id<Workplace>"
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.created_at
FROM members m
JOIN members_workplaces mw ON mw.member_id = m.id
WHERE mw.workplace_id = ANY($1)
  AND m.left_at IS NULL
ORDER BY m.last_name NULLS LAST, m.first_name NULLS LAST
"#,
        workplace_ids
    )
    .fetch_all(pool)
    .await
}
