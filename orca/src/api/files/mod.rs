use chrono::{DateTime, Utc};
use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::serde::Serialize;
use rocket::{get, routes, Route, State};

use crate::api::Response;
use crate::data::Id;
use crate::db::DbPool;
use crate::server::oid::{JwtToken, Provider, Role};

#[derive(Debug, sqlx::FromRow)]
pub struct File {
    // This is read only and using Rc<[u8]> rather than Vec<u8> removes some overhead
    data: Vec<u8>,
    file_type: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FileInfo {
    id: Id<File>,
    name: String,
    file_type: String,
    created_at: DateTime<Utc>,
}

impl<'r> Responder<'r, 'static> for File {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let prefix = match self.file_type.as_str() {
            "png" => "image",
            "jpg" => "image",
            _ => "application",
        };

        let content_type = ContentType::new(prefix, self.file_type);

        rocket::response::Response::build_from(self.data.respond_to(req)?)
            .header(content_type)
            .ok()
    }
}

#[get("/<id>?<token..>")]
async fn get<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<File>,
) -> Response<File> {
    oid_provider.require_role(&token, Role::ViewApplication)?;

    let file: File = read_file(id).fetch_one(db_pool.inner()).await?;
    Ok(file)
}

use crate::db::QueryAs;

fn read_file(id: Id<File>) -> QueryAs<'static, File> {
    sqlx::query_as(
        "
SELECT data, file_type FROM files
WHERE id = $1
",
    )
    .bind(id)
}

pub fn routes() -> Vec<Route> {
    routes![get]
}
