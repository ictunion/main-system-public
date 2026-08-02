use crate::data::{Id, Member};

pub(crate) async fn create_email_subscription<'a, E>(
    executor: E,
    member_id: Id<Member>,
    list: i64,
    listmonk_status: &str,
    listmonk_id: i32,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let list_str = list.to_string();
    sqlx::query!(
        r#"
        INSERT INTO email_subscriptions (member_id, list, listmonk_status, listmonk_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
        member_id as _,
        list_str,
        listmonk_status,
        listmonk_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}
