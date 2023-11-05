use super::{Detail, NewMember, Summary};
use crate::api::files::FileInfo;
use crate::data::{Id, Member, MemberNumber};
use crate::db::QueryAs;

pub fn list_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT id
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

pub fn list_past_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , city
    , left_at
    , created_at
FROM members_past
ORDER BY member_number DESC
",
    )
}

pub fn list_new_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , city
    , left_at
    , created_at
FROM members_new
ORDER BY member_number DESC
",
    )
}

pub fn list_current_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , city
    , left_at
    , created_at
FROM members_current
ORDER BY member_number DESC
",
    )
}

pub fn create_member(member_number: MemberNumber, new_member: &NewMember) -> QueryAs<'_, Summary> {
    sqlx::query_as(
        "
INSERT INTO members
    ( member_number
    , email
    , first_name
    , last_name
    , language
    , date_of_birth
    , address
    , city
    , postal_code
    , phone_number
    )
VALUES
    ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
RETURNING id
    , member_number
    , first_name
    , last_name
    , email
    , phone_number
    , city
    , left_at
    , created_at
",
    )
    .bind(member_number)
    .bind(&new_member.email)
    .bind(&new_member.first_name)
    .bind(&new_member.last_name)
    .bind(&new_member.language)
    .bind(new_member.date_of_birth)
    .bind(&new_member.address)
    .bind(&new_member.city)
    .bind(&new_member.postal_code)
    .bind(&new_member.phone_number)
}

pub fn detail<'a>(id: Id<Member>) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
SELECT id
    , member_number
    , first_name
    , last_name
    , date_of_birth
    , email
    , phone_number
    , address
    , city
    , postal_code
    , language
    , registration_request_id as application_id
    , left_at
    , onboarding_finished_at
    , created_at
FROM members
WHERE id = $1
",
    )
    .bind(id)
}

// Returns highest existing member_number + 1 or 1
// to be used as a member number for next new member
pub fn get_next_member_number<'a>() -> QueryAs<'a, (MemberNumber,)> {
    sqlx::query_as(
        "
SELECT COALESCE
    (1 + (
        SELECT member_number FROM members m
        ORDER BY member_number DESC
        LIMIT 1)
    , 1);
",
    )
}

pub fn list_member_files<'a>(id: Id<Member>) -> QueryAs<'a, FileInfo> {
    sqlx::query_as(
        "
SELECT f.id
, f.name
, f.file_type
, f.created_at
FROM members_files AS mf
INNER JOIN files AS f ON f.id = mf.file_id
ORDER BY f.created_at DESC
",
    )
    .bind(id)
}
