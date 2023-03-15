use crate::db::{Query, QueryAs};

use super::MemberDetails;
use crate::data::{Id, Member};

pub fn query_member<'a>(id: Id<Member>) -> QueryAs<'a, MemberDetails> {
    sqlx::query_as("
select first_name, last_name, date_of_birth, phone_number, email, address, city, postal_code, company_name, occupation, confirmation_token
from members where id = $1
")
    .bind(id)
}

pub fn insert_registration_pdf(id: Id<Member>, data: &Vec<u8>) -> Query<'_> {
    sqlx::query(
        "
insert into files
( name
, file_type
, data
, user_id
) values ('registration', 'pdf', $1, $2)
",
    )
    .bind(data)
    .bind(id)
}

pub fn track_verification_sent_at(id: Id<Member>) -> Query<'static> {
    sqlx::query(
        "
update members as m
set verification_sent_at = now()
where id = $1
",
    )
    .bind(id)
}

pub fn fetch_registration_pdf_base64(id: Id<Member>) -> QueryAs<'static, (String,)> {
    sqlx::query_as(
        "
select encode(data, 'base64') from files
where user_id = $1
    and file_type = 'pdf'
    and name = 'registration'
order by created_at desc
limit 1
",
    )
    .bind(id)
}
