use crate::data::{Id, Member};
use crate::db::Query;

pub(crate) fn create_email_subscription<'a>(
    member_id: Id<Member>,
    list: i64,
    listmonk_status: &'static str,
    listmonk_id: i32,
) -> Query<'a> {
    sqlx::query(
        "
        INSERT INTO email_subscriptions (member_id, list, listmonk_status, listmonk_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ",
    )
    .bind(member_id)
    .bind(list)
    .bind(listmonk_status)
    .bind(listmonk_id)
}
