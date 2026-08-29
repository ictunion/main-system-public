use crate::db::DbPool;

pub async fn count_unverified_applications(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM registration_requests_unverified"#)
        .fetch_one(pool)
        .await
}

pub async fn count_accepted_applications(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM registration_requests_accepted"#)
        .fetch_one(pool)
        .await
}

pub async fn count_rejected_applications(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM registration_requests_rejected"#)
        .fetch_one(pool)
        .await
}

pub async fn count_processing_applications(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM registration_requests_processing"#)
        .fetch_one(pool)
        .await
}

pub async fn count_invalid_applications(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM registration_requests_invalid"#)
        .fetch_one(pool)
        .await
}

pub async fn count_new_members(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM members_new"#)
        .fetch_one(pool)
        .await
}

pub async fn count_current_members(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM members_current"#)
        .fetch_one(pool)
        .await
}

pub async fn count_past_members(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM members_past"#)
        .fetch_one(pool)
        .await
}

pub async fn count_workplace_members(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT m.id) as "count!"
FROM members m
JOIN members_workplaces mw ON mw.member_id = m.id
WHERE m.left_at IS NULL"#
    )
    .fetch_one(pool)
    .await
}

pub async fn count_sectoral_members(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!"
FROM members m
WHERE m.left_at IS NULL
  AND NOT EXISTS (SELECT 1 FROM members_workplaces mw WHERE mw.member_id = m.id)"#
    )
    .fetch_one(pool)
    .await
}

pub async fn count_current_workplaces(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM workplaces WHERE cancelled_at IS NULL"#)
        .fetch_one(pool)
        .await
}

pub async fn count_past_workplaces(pool: &DbPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM workplaces WHERE cancelled_at IS NOT NULL"#
    )
    .fetch_one(pool)
    .await
}
