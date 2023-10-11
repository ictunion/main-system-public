use super::Summary;
use crate::db::QueryAs;

pub fn list_summaries<'a>() -> QueryAs<'a, Summary> {
    sqlx::query_as(
        "
SELECT
      id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , city
    , left_at
    , created_at
FROM members
ORDER BY member_number DESC
",
    )
}
