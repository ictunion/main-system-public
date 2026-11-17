use chrono::NaiveDate;
use uuid::Uuid;

use super::BankSyncMember;
use crate::db::DbPool;
use crate::newsletter::NewsletterMember;

pub async fn list_bank_sync_members(pool: &DbPool) -> sqlx::Result<Vec<BankSyncMember>> {
    sqlx::query_as!(
        BankSyncMember,
        r#"
SELECT m.sub
    , m.member_number
    , m.onboarding_finished_at::date AS "fee_start_date: NaiveDate"
    , m.left_at::date AS "fee_stop_date: NaiveDate"
    , (m.onboarding_finished_at IS NOT NULL AND m.left_at IS NULL) AS "active!: bool"
    , (
        SELECT w.keycloak_executive_group_id
        FROM members_workplaces mw
        JOIN workplaces w ON w.id = mw.workplace_id
        WHERE mw.member_id = m.id
        ORDER BY w.name
        LIMIT 1
      ) AS "workplace_executive_committee_sub: Uuid"
FROM members AS m
"#
    )
    .fetch_all(pool)
    .await
}

/// Active members only (same notion as the bank sync `active` flag).
pub async fn list_newsletter_sync_members(pool: &DbPool) -> sqlx::Result<Vec<NewsletterMember>> {
    sqlx::query_as!(
        NewsletterMember,
        r#"
SELECT m.member_number
    , m.email
    , CONCAT_WS(' ', NULLIF(m.first_name, ''), NULLIF(m.last_name, '')) AS "name!: String"
    , m.city
    -- orca's own emails fall back to English for a missing language
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
WHERE m.onboarding_finished_at IS NOT NULL
  AND m.left_at IS NULL
"#
    )
    .fetch_all(pool)
    .await
}
