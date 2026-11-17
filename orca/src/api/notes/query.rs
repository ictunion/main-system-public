use uuid::Uuid;

use super::{NoteDetail, NoteSummary, NoteTarget};
use crate::data::{Id, Note};
use crate::db::DbPool;

/// `workplace_ids` limits the result to current members of those workplaces
/// (rep scope); `None` returns every target of the type.
pub async fn list_summary(
    pool: &DbPool,
    target: NoteTarget,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<Vec<NoteSummary>> {
    sqlx::query_as!(
        NoteSummary,
        r#"
SELECT COALESCE(n.registration_request_id, n.member_id, n.workplace_id) AS "target_id!"
    , COUNT(*) AS "count!"
    , (ARRAY_AGG(n.body ORDER BY n.created_at DESC, n.id))[1] AS "latest_body!"
FROM notes n
WHERE n.target_type = $1
  AND ( $2::uuid[] IS NULL
     OR n.member_id IN (
        SELECT mw.member_id
        FROM members_workplaces mw
        JOIN members m ON m.id = mw.member_id
        WHERE mw.workplace_id = ANY($2)
          AND m.left_at IS NULL
     ))
GROUP BY 1
"#,
        target as _,
        workplace_ids as _,
    )
    .fetch_all(pool)
    .await
}

pub async fn list(
    pool: &DbPool,
    target: NoteTarget,
    target_id: Uuid,
) -> sqlx::Result<Vec<NoteDetail>> {
    sqlx::query_as!(
        NoteDetail,
        r#"
SELECT id
    , target_type AS "target_type: NoteTarget"
    , COALESCE(registration_request_id, member_id, workplace_id) AS "target_id!"
    , body
    , author_sub
    , author_name
    , created_at
    , updated_at
FROM notes
WHERE target_type = $1
  AND (registration_request_id = $2 OR member_id = $2 OR workplace_id = $2)
ORDER BY created_at DESC, id
"#,
        target as _,
        target_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn create(
    pool: &DbPool,
    target: NoteTarget,
    target_id: Uuid,
    body: &str,
    author_sub: Uuid,
    author_name: &str,
) -> sqlx::Result<NoteDetail> {
    let (registration_request_id, member_id, workplace_id) = target.columns(target_id);

    sqlx::query_as!(
        NoteDetail,
        r#"
INSERT INTO notes
    ( target_type
    , registration_request_id
    , member_id
    , workplace_id
    , body
    , author_sub
    , author_name
    )
VALUES ( $1, $2, $3, $4, $5, $6, $7 )
RETURNING id
    , target_type AS "target_type: NoteTarget"
    , COALESCE(registration_request_id, member_id, workplace_id) AS "target_id!"
    , body
    , author_sub
    , author_name
    , created_at
    , updated_at
"#,
        target as _,
        registration_request_id,
        member_id,
        workplace_id,
        body,
        author_sub,
        author_name,
    )
    .fetch_one(pool)
    .await
}

/// Author of a note that belongs to the given target; `RowNotFound` when the
/// note does not exist or belongs to another target.
pub async fn author_of(
    pool: &DbPool,
    target: NoteTarget,
    target_id: Uuid,
    note_id: Id<Note>,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        r#"
SELECT author_sub
FROM notes
WHERE id = $1
  AND target_type = $2
  AND (registration_request_id = $3 OR member_id = $3 OR workplace_id = $3)
"#,
        note_id as _,
        target as _,
        target_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &DbPool, note_id: Id<Note>, body: &str) -> sqlx::Result<NoteDetail> {
    sqlx::query_as!(
        NoteDetail,
        r#"
UPDATE notes
SET body = $2
WHERE id = $1
RETURNING id
    , target_type AS "target_type: NoteTarget"
    , COALESCE(registration_request_id, member_id, workplace_id) AS "target_id!"
    , body
    , author_sub
    , author_name
    , created_at
    , updated_at
"#,
        note_id as _,
        body,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &DbPool, note_id: Id<Note>) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM notes WHERE id = $1", note_id as _)
        .execute(pool)
        .await?;

    Ok(())
}

/// Returns how many notes were deleted.
pub async fn delete_all(pool: &DbPool, target: NoteTarget, target_id: Uuid) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        r#"
DELETE FROM notes
WHERE target_type = $1
  AND (registration_request_id = $2 OR member_id = $2 OR workplace_id = $2)
"#,
        target as _,
        target_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
