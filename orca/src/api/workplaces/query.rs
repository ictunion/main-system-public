use uuid::Uuid;

use super::{NewWorkplace, UpdateWorkplace, WorkplaceSummary};
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

pub async fn create_connection_between_member_and_workplace(
    pool: &DbPool,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> sqlx::Result<u64> {
    Ok(sqlx::query!(
        r#"
INSERT INTO members_workplaces
    ( workplace_id
    , member_id
    )
VALUES
    ( $1, $2 )
ON CONFLICT DO NOTHING
"#,
        workplace_id as _,
        member_id as _,
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
