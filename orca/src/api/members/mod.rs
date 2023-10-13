use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_validation::{Validate, Validated};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::api::Response;
use crate::data::{Id, Member, MemberNumber};
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak, Role};

pub mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Summary {
    id: Id<Member>,
    member_number: MemberNumber,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone_number: Option<String>,
    city: Option<String>,
    left_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[get("/")]
async fn list_all<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    keycloak.require_role(token, Role::ListMembers)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[derive(Deserialize, Validate)]
pub struct NewMember {
    member_number: Option<MemberNumber>,
    first_name: Option<String>,
    last_name: Option<String>,
    date_of_birth: Option<Date>,
    #[validate(required)]
    #[validate(email)]
    email: Option<String>,
    phone_number: Option<String>,
    city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    language: String,
}

#[post("/", format = "json", data = "<new_member>")]
async fn create_member<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    new_member: Validated<Json<NewMember>>,
) -> Response<Json<Summary>> {
    keycloak.require_role(token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;

    // ensure member number
    let member_number = match new_member.0.member_number {
        Some(num) => num,
        None => {
            let (new_num,) = query::get_next_member_number().fetch_one(&mut tx).await?;
            new_num
        }
    };

    // Create new member
    let summary = query::create_member(member_number, &new_member.0)
        .fetch_one(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(summary))
}

pub fn routes() -> Vec<Route> {
    routes![list_all, create_member]
}
