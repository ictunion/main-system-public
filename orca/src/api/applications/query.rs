use crate::{
    data::{Member, MemberNumber},
    db::{Query, QueryAs},
};

use super::{
    AcceptedSummary, ApplicationStatusData, Detail, File, ProcessingSummary, RejectedSummary,
    Summary, UnverifiedSummary,
};
use crate::data::{Id, RegistrationRequest};

pub fn list_summaries<'a>() -> QueryAs<'a, Summary> {
    sqlx::query_as(
        "
SELECT id
, email
, first_name
, last_name
, phone_number
, city
, company_name
, registration_local
, created_at
FROM registration_requests
ORDER BY created_at DESC
",
    )
}

pub fn list_unverified_summaries<'a>() -> QueryAs<'a, UnverifiedSummary> {
    sqlx::query_as(
        "
SELECT id
, email
, first_name
, last_name
, phone_number
, city
, company_name
, registration_local
, created_at
, verification_sent_at
FROM registration_requests_unverified
ORDER BY created_at DESC
",
    )
}

pub fn list_processing_summaries<'a>() -> QueryAs<'a, ProcessingSummary> {
    sqlx::query_as(
        "
SELECT id
, email
, first_name
, last_name
, phone_number
, city
, company_name
, registration_local
, created_at
, confirmed_at
FROM registration_requests_processing
ORDER BY confirmed_at DESC
",
    )
}

pub fn list_accepted_summaries<'a>() -> QueryAs<'a, AcceptedSummary> {
    sqlx::query_as(
        "
SELECT rr.id
, rr.email
, rr.first_name
, rr.last_name
, rr.phone_number
, rr.city
, rr.company_name
, rr.registration_local
, rr.created_at
, m.created_at AS accepted_at
, m.id AS member_id
FROM registration_requests_accepted as rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
ORDER BY created_at DESC
",
    )
}

pub fn list_rejected_summaries<'a>() -> QueryAs<'a, RejectedSummary> {
    sqlx::query_as(
        "
SELECT id
, email
, first_name
, last_name
, phone_number
, city
, company_name
, registration_local
, created_at
, rejected_at
FROM registration_requests_rejected
ORDER BY created_at DESC
",
    )
}

pub fn get_application<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
SELECT rr.id
, rr.email
, rr.first_name
, rr.last_name
, rr.date_of_birth
, rr.phone_number
, rr.city
, rr.address
, rr.postal_code
, rr.occupation
, rr.company_name
, rr.verification_sent_at
, rr.confirmed_at
, rr.registration_ip
, rr.registration_local
, rr.registration_user_agent
, rr.registration_source
, rr.rejected_at
, rr.created_at
, m.created_at AS accepted_at
FROM registration_requests AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
WHERE rr.id = $1
",
    )
    .bind(id)
}

pub fn list_application_files<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, File> {
    sqlx::query_as(
        "
SELECT f.id
, f.name
, f.file_type
, f.created_at
FROM registration_requests_files AS rrf
INNER JOIN files AS f ON f.id = rrf.file_id
WHERE rrf.registration_request_id = $1
ORDER BY f.created_at DESC
",
    )
    .bind(id)
}

pub fn reject_application<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, Detail> {
    // We can hardcode null because we know rejected application can't belong to user
    sqlx::query_as(
        "
UPDATE registration_requests
SET rejected_at = NOW()
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, created_at
, NULL AS accepted_at
",
    )
    .bind(id)
}

pub fn unreject_application<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
UPDATE registration_requests
SET rejected_at = NULL
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, created_at
, NULL AS accepted_at
",
    )
    .bind(id)
}

pub fn verify_application<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, Detail> {
    sqlx::query_as(
        "
UPDATE registration_requests
SET   confirmed_at = NOW()
    , confirmation_token = NULL
WHERE id = $1
RETURNING id
, email
, first_name
, last_name
, date_of_birth
, phone_number
, city
, address
, postal_code
, occupation
, company_name
, verification_sent_at
, confirmed_at
, registration_ip
, registration_local
, registration_user_agent
, registration_source
, rejected_at
, created_at
, NULL AS accepted_at
",
    )
    .bind(id)
}

pub fn get_application_status_data<'a>(
    id: Id<RegistrationRequest>,
) -> QueryAs<'a, ApplicationStatusData> {
    sqlx::query_as(
        "
SELECT rr.id
, rr.created_at
, rr.confirmed_at
, rr.rejected_at
, m.created_at AS accepted_at
, m.id AS member_id
FROM registration_requests AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
WHERE rr.id = $1
",
    )
    .bind(id)
}

/// TODO: this should return member once we have it type for it
pub fn create_new_member<'a>(
    id: Id<RegistrationRequest>,
    number: MemberNumber,
) -> QueryAs<'a, (Id<Member>,)> {
    sqlx::query_as(
        "
INSERT INTO members AS m
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
, registration_request_id
)
SELECT $2
, email
, first_name
, last_name
, registration_local
, date_of_birth
, address
, city
, postal_code
, phone_number
, $1
FROM registration_requests as rr
WHERE rr.id = $1
RETURNING m.id
",
    )
    .bind(id)
    .bind(number)
}

pub fn attach_files_to_member<'a>(
    registration_id: Id<RegistrationRequest>,
    member_id: Id<Member>,
) -> Query<'a> {
    sqlx::query(
        "
INSERT INTO members_files
( member_id
, file_id
)
SELECT $2, file_id FROM registration_requests_files
WHERE registration_request_id = $1
",
    )
    .bind(registration_id)
    .bind(member_id)
}

pub fn attach_occupation<'a>(
    registration_id: Id<RegistrationRequest>,
    member_id: Id<Member>,
) -> Query<'a> {
    sqlx::query(
        "
INSERT INTO occupations
( member_id
, company_name
, position
)
SELECT $2, rr.company_name, rr.occupation
FROM registration_requests as rr
WHERE rr.id = $1
",
    )
    .bind(registration_id)
    .bind(member_id)
}

pub fn dangerous_hard_delete_application_data<'a>(
    registration_id: Id<RegistrationRequest>,
) -> Query<'a> {
    sqlx::query(
        "
WITH file_ids AS
    (DELETE FROM registration_requests_files
        WHERE registration_request_id = $1
        RETURNING file_id)
DELETE FROM files WHERE id IN (SELECT file_id FROM file_ids)
",
    )
    .bind(registration_id)
}

pub fn dangerous_hard_delete_application<'a>(
    registration_id: Id<RegistrationRequest>,
) -> Query<'a> {
    sqlx::query(
        "
DELETE FROM registration_requests
    WHERE id = $1
",
    )
    .bind(registration_id)
}
