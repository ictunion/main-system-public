use crate::db::QueryAs;

use super::{ProcessingSummary, UnverifiedSummary};

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
