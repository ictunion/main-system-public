use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{Route, State};
use serde::Serialize;

use crate::api::Response;
use crate::data::{Id, Member, MemberNumber};
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak, Role};

mod query;

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
async fn list<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    keycloak.require_role(token, Role::ListMembers)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;

    Ok(Json(summaries))
}

pub fn routes() -> Vec<Route> {
    routes![list]
}
