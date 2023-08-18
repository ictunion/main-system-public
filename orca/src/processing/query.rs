use crate::db::{Query, QueryAs};

use super::RegistrationDetails;
use crate::data::{Id, RegistrationRequest};

pub fn query_registration<'a>(id: Id<RegistrationRequest>) -> QueryAs<'a, RegistrationDetails> {
    sqlx::query_as("
SELECT first_name, last_name, date_of_birth, phone_number, email, address, city, postal_code, company_name, occupation, confirmation_token, registration_local
FROM registration_requests WHERE id = $1
")
    .bind(id)
}

pub fn insert_registration_pdf(id: Id<RegistrationRequest>, data: &Vec<u8>) -> Query<'_> {
    sqlx::query(
        "
WITH rows AS
( INSERT INTO files
    ( name
    , file_type
    , data
    ) VALUES ('registration', 'pdf', $1)
    RETURNING id
)
INSERT INTO registration_requests_files
    ( registration_request_id
    , file_id
    )
    SELECT $2 as registration_request_id, id
    FROM rows
",
    )
    .bind(data)
    .bind(id)
}

pub fn track_verification_sent_at(id: Id<RegistrationRequest>) -> Query<'static> {
    sqlx::query(
        "
UPDATE registration_requests AS m
SET verification_sent_at = now()
WHERE id = $1
",
    )
    .bind(id)
}

pub fn fetch_registration_pdf(id: Id<RegistrationRequest>) -> QueryAs<'static, (Vec<u8>,)> {
    sqlx::query_as(
        "
SELECT f.data FROM registration_requests_files rrf
RIGHT JOIN files f
ON rrf.file_id = f.id
WHERE registration_request_id = $1
    AND file_type = 'pdf'
    AND name = 'registration'
ORDER BY f.created_at DESC
LIMIT 1
",
    )
    .bind(id)
}
