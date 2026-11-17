use super::NewsletterMember;
use crate::data::{Id, Member, Workplace};
use crate::db::DbPool;

// Same columns as `api::sync::query::list_newsletter_sync_members`; keep them in step.

/// Members who haven't left (newsletter-system already deleted those), narrowed by
/// whichever filters are given: one member, the members of one workplace, or both.
pub async fn list_members(
    pool: &DbPool,
    member_id: Option<Id<Member>>,
    workplace_id: Option<Id<Workplace>>,
) -> sqlx::Result<Vec<NewsletterMember>> {
    sqlx::query_as!(
        NewsletterMember,
        r#"
SELECT m.member_number
    , m.email
    , CONCAT_WS(' ', NULLIF(m.first_name, ''), NULLIF(m.last_name, '')) AS "name!: String"
    , m.city
    , COALESCE(m.language, 'en') AS "lang!: String"
    , (
        SELECT w.newsletter_id
        FROM members_workplaces mw
        JOIN workplaces w ON w.id = mw.workplace_id
        WHERE mw.member_id = m.id
          AND w.newsletter_id IS NOT NULL
        ORDER BY w.name
        LIMIT 1
      ) AS "workplace_newsletter_id: i32"
FROM members AS m
WHERE m.left_at IS NULL
  AND ($1::uuid IS NULL OR m.id = $1)
  AND ($2::uuid IS NULL OR EXISTS (
        SELECT 1
        FROM members_workplaces wm
        WHERE wm.member_id = m.id
          AND wm.workplace_id = $2
      ))
"#,
        member_id as _,
        workplace_id as _,
    )
    .fetch_all(pool)
    .await
}
