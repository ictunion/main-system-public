use crate::db::QueryAs;

use super::{Detail, File, ProcessingSummary, UnverifiedSummary};
use crate::data::{Id, RegistrationRequest};

pub fn get_unverified_summaries<'a>() -> QueryAs<'a, UnverifiedSummary> {
    sqlx::query_as(
        "
SELECT id, email, first_name, last_name, phone_number, city, company_name, registration_local, created_at, verification_sent_at
FROM registration_requests_unverified
ORDER BY created_at DESC
",
    )
}

pub fn get_processing_summaries<'a>() -> QueryAs<'a, ProcessingSummary> {
    sqlx::query_as(
        "
SELECT id, email, first_name, last_name, phone_number, city, company_name, registration_local, created_at, confirmed_at
FROM registration_requests_processing
ORDER BY confirmed_at DESC
",
    )
}

pub fn get_application<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, Detail> {
    sqlx::query_as("
SELECT
    rr.id, rr.email, rr.first_name, rr.last_name, rr.date_of_birth, rr.phone_number, rr.city, rr.address,
    rr.postal_code, rr.occupation, rr.company_name, rr.verification_sent_at, rr.confirmed_at,
    rr.registration_ip, rr.registration_local, rr.registration_user_agent, rr.registration_source, rr.rejected_at, rr.created_at,
    m.created_at AS accepted_at
FROM registration_requests AS rr
LEFT JOIN members AS m ON rr.id = m.registration_request_id
WHERE rr.id = $1
")
    .bind(id)
}

pub fn get_application_files<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, File> {
    sqlx::query_as(
        "
    SELECT f.id, f.name, f.file_type, f.created_at
FROM registration_requests_files AS rrf
INNER JOIN files AS f ON f.id = rrf.file_id
WHERE rrf.registration_request_id = $1
ORDER BY f.created_at DESC
",
    )
    .bind(id)
}
