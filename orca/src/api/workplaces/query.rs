use uuid::Uuid;

use super::{NewWorkplace, WorkplaceSummary};
use crate::api::members::Summary;
use crate::data::{Id, Workplace};
use crate::db::{Query, QueryAs};

pub fn list_summaries() -> QueryAs<'static, WorkplaceSummary> {
    sqlx::query_as(
        "
SELECT id
    , name
    , email
    , created_at
FROM workplaces
ORDER BY created_at DESC
",
    )
}

pub fn detail<'a>(id: Id<Workplace>) -> QueryAs<'a, WorkplaceSummary> {
    sqlx::query_as(
        "
SELECT id
    , name
    , email
    , created_at
FROM workplaces
WHERE id = $1
",
    )
    .bind(id)
}

pub fn create_workplace(new_workplace: &NewWorkplace) -> QueryAs<'_, WorkplaceSummary> {
    sqlx::query_as(
        "
INSERT INTO workplaces
    ( name
     , email
    )
VALUES
    ( $1, $2 )
RETURNING id
    , name
    , email
    , created_at
",
    )
    .bind(&new_workplace.name)
    .bind(&new_workplace.email)
}

pub fn create_connection_between_member_and_workplace<'a>(
    workplace_id: Id<Workplace>,
    member_id: Uuid,
) -> Query<'a> {
    sqlx::query(
        "
INSERT INTO members_workplaces
    ( workplace_id
    , member_id
    )
VALUES
    ( $1, $2 )
",
    )
    .bind(workplace_id)
    .bind(member_id)
}

pub fn remove_connection_between_member_and_workplace<'a>(
    workplace_id: Id<Workplace>,
    member_id: Uuid,
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
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS company_names
    , m.created_at
FROM members AS m
LEFT JOIN occupations o ON o.member_id = m.id
LEFT JOIN members_workplaces mw ON mw.member_id = m.id 
WHERE mw.workplace_id = $1
GROUP BY m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.left_at
    , m.created_at
ORDER BY m.member_number DESC
",
    )
    .bind(workplace_id)
}
