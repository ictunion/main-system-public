use crate::db::QueryAs;

pub fn count_unverified_applications<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM registration_requests_unverified
",
    )
}

pub fn count_accepted_applications<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM registration_requests_accepted
",
    )
}

pub fn count_rejected_applications<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM registration_requests_rejected
",
    )
}

pub fn count_processing_applications<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM registration_requests_processing
",
    )
}

pub fn count_invalid_applications<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM registration_requests_invalid
",
    )
}

pub fn count_new_members<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM members_new;
",
    )
}

pub fn count_current_members<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM members_current;
",
    )
}

pub fn count_past_members<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT COUNT(*) FROM members_past;
",
    )
}
