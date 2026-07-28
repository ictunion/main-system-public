use super::{NewWorkplace, UpdateWorkplace, WorkplaceSummary};
use crate::api::members::Summary;
use crate::data::{Id, Member, Workplace};
use crate::db::{DbPool, Query, QueryAs};

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

pub fn remove_member_workplace_associations<'a>(id: Id<Member>) -> Query<'a> {
    sqlx::query(
        "
DELETE FROM members_workplaces
WHERE member_id = $1
",
    )
    .bind(id)
}

pub fn create_connection_between_member_and_workplace<'a>(
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> Query<'a> {
    sqlx::query(
        "
INSERT INTO members_workplaces
    ( workplace_id
    , member_id
    )
VALUES
    ( $1, $2 )
ON CONFLICT DO NOTHING
",
    )
    .bind(workplace_id)
    .bind(member_id)
}

pub fn remove_connection_between_member_and_workplace<'a>(
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> Query<'a> {
    sqlx::query(
        "
DELETE FROM members_workplaces
    WHERE 
        workplace_id=$1 AND member_id=$2
",
    )
    .bind(workplace_id)
    .bind(member_id)
}

pub fn get_all_workplace_members<'a>(workplace_id: Id<Workplace>) -> QueryAs<'a, Summary> {
    sqlx::query_as(
        "
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
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS company_names
    , m.created_at
    , ARRAY(SELECT wp.workplace_id FROM members_workplaces wp WHERE wp.member_id = m.id) AS workplace_ids
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
",
    )
    .bind(workplace_id)
}
