use chrono::NaiveDate;
use uuid::Uuid;

use super::BankSyncMember;
use crate::db::DbPool;

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
