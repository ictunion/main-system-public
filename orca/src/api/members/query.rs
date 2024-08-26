use uuid::Uuid;

use super::{Detail, MemberStatusData, NewMember, Occupation, Summary, UpdateMember};
use crate::api::files::FileInfo;
use crate::data::{Id, Member, MemberNumber};
use crate::db::QueryAs;
use crate::server::oid;

pub fn list_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS company_names
    , m.created_at
FROM members AS m
LEFT JOIN occupations o ON o.member_id = m.id
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
}

pub fn list_past_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS company_names
    , m.created_at
FROM members_past AS m
LEFT JOIN occupations o ON o.member_id = m.id
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
}

pub fn list_new_summaries() -> QueryAs<'static, Summary> {
    sqlx::query_as(
        "
SELECT m.id
    , m.member_number
    , m.first_name
    , m.last_name
    , m.email
    , m.phone_number
    , m.note
    , m.city
    , m.left_at
    , array_agg(o.company_name ORDER BY o.created_at DESC) AS company_names
    , m.created_at
FROM members_new AS m
LEFT JOIN occupations o ON o.member_id = m.id
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
}

pub fn list_current_summaries() -> QueryAs<'static, Summary> {
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
FROM members_current AS m
LEFT JOIN occupations o ON o.member_id = m.id
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
    , note
    , city
    , left_at
    , created_at
    , ARRAY[]::text[] AS company_names
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
    , note
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
WHERE mf.member_id = $1
ORDER BY f.created_at DESC
",
    )
    .bind(id)
}

pub fn list_occupations<'a>(id: Id<Member>) -> QueryAs<'a, Occupation> {
    sqlx::query_as(
        "
SELECT id
, company_name
, position
, created_at
FROM occupations
WHERE member_id = $1
ORDER BY created_at DESC
",
    )
    .bind(id)
}

pub fn get_new_oid_user<'a>(id: Id<Member>) -> QueryAs<'a, oid::User> {
    sqlx::query_as(
        "
SELECT NULL as id, first_name, last_name, email
FROM members
WHERE id = $1
",
    )
    .bind(id)
}

pub fn assing_member_oid_sub<'a>(id: Id<Member>, uuid: Uuid) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
UPDATE members
SET sub = $2
, onboarding_finished_at = NOW()
WHERE id = $1
RETURNING id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
",
    )
    .bind(id)
    .bind(uuid)
}

pub fn get_status_data<'a>(id: Id<Member>) -> QueryAs<'a, MemberStatusData> {
    sqlx::query_as(
        "
SELECT sub, left_at
FROM members
WHERE id = $1
",
    )
    .bind(id)
}

pub fn update_member_note<'a>(id: Id<Member>, new_note: String) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
    UPDATE members
    SET note = $2
    WHERE id = $1
    RETURNING id
    , member_number
    , first_name
    , last_name
    , date_of_birth
    , email
    , phone_number
    , note
    , address
    , city
    , postal_code
    , language
    , registration_request_id as application_id
    , left_at
    , onboarding_finished_at
    , created_at
    ",
    )
    .bind(id)
    .bind(new_note)
}

pub fn update_member<'a>(id: Id<Member>, updated_member: UpdateMember) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
UPDATE members
SET first_name = $2
    , last_name = $3
    , date_of_birth = $4
    , email = $5
    , phone_number = $6
    , note = $7
    , address = $8
    , city = $9
    , postal_code = $10
    , language = $11
WHERE id = $1
RETURNING id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
",
    )
    .bind(id)
    .bind(updated_member.first_name)
    .bind(updated_member.last_name)
    .bind(updated_member.date_of_birth)
    .bind(updated_member.email)
    .bind(updated_member.phone_number)
    .bind(updated_member.note)
    .bind(updated_member.address)
    .bind(updated_member.city)
    .bind(updated_member.postal_code)
    .bind(updated_member.language)
}

// This doesn't realy delete member from the database
// We're just adding left_at flag to the data
pub fn remove_member<'a>(id: Id<Member>) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
UPDATE members
SET left_at = NOW()
WHERE id = $1
RETURNING id
, member_number
, first_name
, last_name
, date_of_birth
, email
, phone_number
, note
, address
, city
, postal_code
, language
, registration_request_id as application_id
, left_at
, onboarding_finished_at
, created_at
",
    )
    .bind(id)
}
