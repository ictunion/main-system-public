use crate::db::QueryAs;

pub fn count_unverified<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT count(*) FROM registration_requests_unverified
",
    )
}

pub fn count_accepted<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT count(*) FROM registration_requests_accepted
",
    )
}

pub fn count_rejected<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT count(*) FROM registration_requests_rejected
",
    )
}

pub fn count_processing<'a>() -> QueryAs<'a, (i64,)> {
    sqlx::query_as(
        "
SELECT count(*) FROM registration_requests_processing
",
    )
}
