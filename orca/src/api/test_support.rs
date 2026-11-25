//! Shared harness for endpoint tests: the full router with a fake identity
//! provider and a captured processing queue, plus throwaway databases cloned
//! from a migrated template. See `docs/testing.md`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request, StatusCode};
use serde_json::Value;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection};
use tokio::sync::{Mutex, OnceCell, mpsc};
use tower::ServiceExt;
use uuid::Uuid;

use super::{AppState, router};
use crate::config::templates::Templates;
use crate::config::{Config, LogLevel};
use crate::db::DbPool;
use crate::newsletter::Newsletter;
use crate::processing::{Command, QueueSender};
pub(crate) use crate::server::oid::fake::FakeProvider;
use crate::server::oid::{Provider, Role};

pub(crate) const BANK_SYNC_TOKEN: &str = "bank-sync-test-token";
pub(crate) const NEWSLETTER_SYNC_TOKEN: &str = "newsletter-sync-test-token";

/// Every role, for "all other roles are refused" checks
pub(crate) const ALL_ROLES: &[Role] = &[
    Role::ListApplications,
    Role::ViewApplication,
    Role::ResolveApplications,
    Role::ListMembers,
    Role::ManageMembers,
    Role::ListWorkplaces,
    Role::ManageWorkplaces,
    Role::ListOwnWorkplaceMembers,
    Role::ListOwnWorkplace,
    Role::ListOwnWorkplaceApplications,
    Role::ResolveOwnWorkplaceApplications,
    Role::ManageNotes,
    Role::SuperPowers,
];

/// Keycloak id behind `TestApp::admin_call`
pub(crate) const ADMIN_SUB: Uuid = Uuid::from_u128(0xad);

pub(crate) struct TestApp {
    router: Router,
    pub(crate) oid: FakeProvider,
    /// Commands handlers sent to background processing; no worker runs
    pub(crate) queue: mpsc::Receiver<Command>,
    /// Superuser pool, for arranging data the `orca` role may not write
    pub(crate) admin_db: Option<DbPool>,
    /// Server connection and name of this test's database, dropped with the app
    database: Option<(PgConnectOptions, String)>,
}

/// Lazy pool whose queries fail after 1 s
fn unreachable_db() -> DbPool {
    let options = PgConnectOptions::new()
        .host("127.0.0.1")
        .port(1)
        .database("unreachable");
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(1))
        .connect_lazy_with(options)
}

impl TestApp {
    /// The database is unreachable (a query fails after 1 s): for checks that
    /// must be decided before any query runs, such as role guards.
    pub(crate) fn without_db() -> Self {
        Self::new(unreachable_db(), None, None, None)
    }

    /// No database and Keycloak not configured
    pub(crate) fn without_keycloak() -> Self {
        Self::new(unreachable_db(), None, None, Some(Provider::disconnected()))
    }

    /// Fresh migrated database. The app connects as `orca`, like in production.
    pub(crate) async fn with_db() -> Self {
        let (server, name) = test_database().await;
        let options = server.clone().database(&name);
        // Superuser login switched to `orca` for the whole session (`-c role=orca`)
        let app_pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options.clone().options([("role", "orca")]))
            .await
            .expect("connect to the test database as orca");
        let admin_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .expect("connect to the test database");
        Self::new(app_pool, Some(admin_pool), Some((server, name)), None)
    }

    fn new(
        db: DbPool,
        admin_db: Option<DbPool>,
        database: Option<(PgConnectOptions, String)>,
        provider: Option<Provider>,
    ) -> Self {
        let oid = FakeProvider::default();
        let provider = provider.unwrap_or_else(|| Provider::fake(oid.clone()));
        let (sender, queue) = mpsc::channel(64);
        let router = router(AppState {
            db,
            queue: QueueSender(sender),
            config: Arc::new(test_config()),
            provider: Arc::new(provider),
            newsletter: Newsletter::new(None),
        });

        Self {
            router,
            oid,
            queue,
            admin_db,
            database,
        }
    }

    /// Sends a request; the body is JSON (`Value::Null` when empty or not JSON)
    pub(crate) async fn request(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<&Value>,
    ) -> (StatusCode, Value) {
        match body {
            Some(body) => {
                let json = Some("application/json");
                self.send_raw(method, uri, token, json, body.to_string())
                    .await
            }
            None => self.send_raw(method, uri, token, None, "").await,
        }
    }

    /// Sends `body` as is, with `content_type` if given
    pub(crate) async fn send_raw(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        content_type: Option<&str>,
        body: impl Into<Body>,
    ) -> (StatusCode, Value) {
        // Client address, as the reverse proxy sets it in production
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-real-ip", "127.0.0.1");
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        let request = request.body(body.into()).expect("valid test request");

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("readable response body");
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    /// Drops queued commands so a long test can't fill the channel
    pub(crate) fn drain_queue(&mut self) {
        while self.queue.try_recv().is_ok() {}
    }

    /// Request as an admin holding every role
    pub(crate) async fn admin_call(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let token = FakeProvider::token(ADMIN_SUB, ALL_ROLES);
        self.request(method, uri, Some(&token), body.as_ref()).await
    }

    pub(crate) async fn admin_status(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> StatusCode {
        self.admin_call(method, uri, body).await.0
    }

    /// Commands queued since the last call, as their `Display` text
    pub(crate) fn queued(&mut self) -> Vec<String> {
        std::iter::from_fn(|| self.queue.try_recv().ok())
            .map(|command| command.to_string())
            .collect()
    }
}

/// State an arranged application is in
#[derive(Clone, Copy, Debug)]
pub(crate) enum ApplicationState {
    Unverified,
    Processing,
    Accepted,
    Rejected,
    Invalid,
}

pub(crate) struct ArrangedWorkplace {
    pub(crate) id: Uuid,
    pub(crate) keycloak_group_id: Uuid,
    pub(crate) keycloak_executive_group_id: Uuid,
}

/// Arranging data with the superuser pool; `name` makes emails and names unique
impl TestApp {
    pub(crate) fn admin(&self) -> &DbPool {
        self.admin_db
            .as_ref()
            .expect("arranging data needs TestApp::with_db")
    }

    pub(crate) async fn insert_workplace(&self, name: &str) -> ArrangedWorkplace {
        let workplace = ArrangedWorkplace {
            id: Uuid::nil(),
            keycloak_group_id: Uuid::new_v4(),
            keycloak_executive_group_id: Uuid::new_v4(),
        };
        let id = sqlx::query_scalar(
            "INSERT INTO workplaces (name, email, keycloak_group_id, keycloak_executive_group_id)
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(name)
        .bind(format!("{name}@example.com"))
        .bind(workplace.keycloak_group_id)
        .bind(workplace.keycloak_executive_group_id)
        .fetch_one(self.admin())
        .await
        .expect("insert workplace");
        ArrangedWorkplace { id, ..workplace }
    }

    /// Result of a `SELECT count(*) … $1` query
    pub(crate) async fn count(&self, sql: &str, id: Uuid) -> i64 {
        sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(self.admin())
            .await
            .expect("count rows")
    }

    /// Gives a member a Keycloak account, so group changes reach the provider
    pub(crate) async fn set_sub(&self, member: Uuid) -> Uuid {
        let sub = Uuid::new_v4();
        sqlx::query("UPDATE members SET sub = $1 WHERE id = $2")
            .bind(sub)
            .bind(member)
            .execute(self.admin())
            .await
            .expect("pair member with a Keycloak user");
        sub
    }

    /// Current member, optionally in `workplace`
    pub(crate) async fn insert_member(&self, name: &str, workplace: Option<Uuid>) -> Uuid {
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO members (member_number, email, first_name, last_name, city)
             VALUES ((SELECT coalesce(max(member_number), 0) + 1 FROM members),
                     $1, 'Member', $2, 'Brno')
             RETURNING id",
        )
        .bind(format!("{name}@example.com"))
        .bind(name)
        .fetch_one(self.admin())
        .await
        .expect("insert member");
        if let Some(workplace) = workplace {
            sqlx::query("INSERT INTO members_workplaces (member_id, workplace_id) VALUES ($1, $2)")
                .bind(id)
                .bind(workplace)
                .execute(self.admin())
                .await
                .expect("assign member to workplace");
        }
        id
    }

    /// Application in `state`, optionally assigned to `workplace`
    pub(crate) async fn insert_application(
        &self,
        name: &str,
        workplace: Option<Uuid>,
        state: ApplicationState,
    ) -> Uuid {
        use ApplicationState::{Accepted, Invalid, Rejected, Unverified};

        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO registration_requests
                 (email, first_name, last_name, city, registration_local,
                  confirmed_at, rejected_at, invalidated_at)
             VALUES ($1, 'Applicant', $2, 'Brno', 'cs',
                     CASE WHEN $3 THEN now() END,
                     CASE WHEN $4 THEN now() END,
                     CASE WHEN $5 THEN now() END)
             RETURNING id",
        )
        .bind(format!("{name}@example.com"))
        .bind(name)
        .bind(!matches!(state, Unverified | Invalid))
        .bind(matches!(state, Rejected))
        .bind(matches!(state, Invalid))
        .fetch_one(self.admin())
        .await
        .expect("insert application");

        if let Some(workplace) = workplace {
            sqlx::query(
                "INSERT INTO registration_requests_workplaces
                     (registration_request_id, workplace_id)
                 VALUES ($1, $2)",
            )
            .bind(id)
            .bind(workplace)
            .execute(self.admin())
            .await
            .expect("assign application to workplace");
        }

        if matches!(state, Accepted) {
            // Accepted means a member was created from the application
            sqlx::query(
                "INSERT INTO members (member_number, email, registration_request_id)
                 VALUES ((SELECT coalesce(max(member_number), 0) + 1 FROM members), $1, $2)",
            )
            .bind(format!("{name}@example.com"))
            .bind(id)
            .execute(self.admin())
            .await
            .expect("insert member of the accepted application");
        }

        id
    }
}

fn test_config() -> Config {
    Config {
        email_sender_email: "orca@example.com".into(),
        email_sender_name: None,
        treasurer_reply_email: "treasurer@example.com".into(),
        treasurer_reply_name: "Treasurer".into(),
        host: "http://localhost:8000".into(),
        admin_host: None,
        postgres: String::new(),
        web_db_pool: 1,
        processing_db_pool: 1,
        tex_exe: "xelatex".into(),
        processing_queue_size: 64,
        verify_redirects_to: HashMap::new(),
        notification_email: None,
        email_confirmation_subjects: HashMap::new(),
        log_level: LogLevel::Off,
        address: std::net::Ipv4Addr::LOCALHOST.into(),
        port: 0,
        json_limit: 1_048_576,
        smtp_host: String::new(),
        smtp_user: String::new(),
        smtp_password: String::new(),
        keycloak_host: None,
        keycloak_realm: None,
        keycloak_client_id: None,
        keycloak_members_group_id: None,
        keycloak_service_client_id: None,
        keycloak_service_client_secret: None,
        templates: Templates::new(),
        newsletter_host: None,
        bank_sync_token: Some(BANK_SYNC_TOKEN.into()),
        newsletter_sync_token: Some(NEWSLETTER_SYNC_TOKEN.into()),
        // Required to queue workplace documents; no document is rendered
        union_chairperson_member_id: Some(Uuid::nil()),
        union_chairperson_email: Some("chair@example.com".into()),
        union_chairperson_signature: Some(Vec::new()),
    }
}

const TEMPLATE_DB: &str = "orca_test_template";

/// Server connection from `DATABASE_URL`, once the template is migrated
static SERVER: OnceCell<PgConnectOptions> = OnceCell::const_new();
/// Postgres refuses to clone a template that another clone is reading
static CLONING: Mutex<()> = Mutex::const_new(());

/// Drops the test's database, also when the test panicked. The template stays
/// until the next run recreates it; databases of a killed run are dropped then too.
impl Drop for TestApp {
    fn drop(&mut self) {
        let Some((server, name)) = self.database.take() else {
            return;
        };
        // `drop` can't await, and blocking on the test's own runtime panics,
        // so the cleanup runs on a thread with its own runtime
        let cleanup = std::thread::spawn({
            let name = name.clone();
            move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("cleanup runtime")
                    .block_on(drop_database(&server, &name))
            }
        })
        .join();
        if !matches!(cleanup, Ok(Ok(()))) {
            // Not a panic: a panic while the test is already unwinding aborts
            eprintln!("could not drop test database {name}, the next run drops it");
        }
    }
}

/// `WITH (FORCE)` closes the app's still open connections
async fn drop_database(server: &PgConnectOptions, name: &str) -> sqlx::Result<()> {
    let mut conn = PgConnection::connect_with(server).await?;
    sqlx::raw_sql(&format!(r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#))
        .execute(&mut conn)
        .await?;
    conn.close().await
}

/// Server connection and name of a new database cloned from the migrated template.
///
/// shortcut: two `cargo test` runs against one server at the same time break
/// each other (each drops the other's template). Upgrade to a per-run template
/// name if CI ever runs in parallel against a shared server.
async fn test_database() -> (PgConnectOptions, String) {
    let server = SERVER.get_or_init(create_template).await;
    let name = format!("orca_test_{}", Uuid::new_v4().simple());

    let _cloning = CLONING.lock().await;
    let mut conn = PgConnection::connect_with(server)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::raw_sql(&format!(
        r#"CREATE DATABASE "{name}" TEMPLATE "{TEMPLATE_DB}""#
    ))
    .execute(&mut conn)
    .await
    .expect("clone the template database");
    conn.close().await.expect("close the server connection");

    (server.clone(), name)
}

async fn create_template() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be a superuser connection, see docs/testing.md");
    let server = PgConnectOptions::from_str(&url).expect("DATABASE_URL is a valid URL");

    let mut conn = PgConnection::connect_with(&server)
        .await
        .expect("connect to DATABASE_URL");
    let stale: Vec<String> =
        sqlx::query_scalar(r"SELECT datname FROM pg_database WHERE datname LIKE 'orca\_test\_%'")
            .fetch_all(&mut conn)
            .await
            .expect("list old test databases");
    for name in stale {
        sqlx::raw_sql(&format!(r#"DROP DATABASE "{name}" WITH (FORCE)"#))
            .execute(&mut conn)
            .await
            .expect("drop an old test database");
    }
    sqlx::raw_sql(&format!(r#"CREATE DATABASE "{TEMPLATE_DB}""#))
        .execute(&mut conn)
        .await
        .expect("create the template database");
    conn.close().await.expect("close the server connection");

    let mut template = PgConnection::connect_with(&server.clone().database(TEMPLATE_DB))
        .await
        .expect("connect to the template database");
    for path in migrations() {
        let sql = std::fs::read_to_string(&path).expect("readable migration");
        sqlx::raw_sql(&adapt_migration(&sql))
            .execute(&mut template)
            .await
            .unwrap_or_else(|err| panic!("migration {} failed: {err}", path.display()));
    }
    template
        .close()
        .await
        .expect("close the template connection");

    server
}

/// gray-whale migrations (refinery naming `V<n>__<name>.sql`) in version order
fn migrations() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../gray-whale/migrations");
    let mut files: Vec<(u32, PathBuf)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", dir.display()))
        .map(|entry| entry.expect("readable directory entry").path())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let version = name.strip_prefix('V')?.split_once("__")?.0.parse().ok()?;
            Some((version, path))
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, path)| path).collect()
}

/// Migrations assume the `ictunion` database on a fresh server. Roles are
/// server-wide, so they may already exist (or be gone) when a test database is
/// built; the database name is the template's.
fn adapt_migration(sql: &str) -> String {
    sql.lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed
                .strip_prefix("CREATE USER ")
                .or_else(|| trimmed.strip_prefix("CREATE ROLE "))
            {
                let role = rest
                    .split(|c: char| c.is_whitespace() || c == ';')
                    .next()
                    .unwrap_or_default();
                format!(
                    "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '{role}') \
                     THEN {trimmed} END IF; END $$;"
                )
            } else if let Some(rest) = trimmed.strip_prefix("DROP ROLE ") {
                format!("DROP ROLE IF EXISTS {rest}")
            } else {
                line.replace("DATABASE ictunion", &format!("DATABASE {TEMPLATE_DB}"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
#[ignore = "reads ../gray-whale, outside the nix build source"]
fn migrations_are_found_in_version_order() {
    let names: Vec<String> = migrations()
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names[0].starts_with("V1__"));
    assert!(
        names[9].starts_with("V10__"),
        "V10 sorts after V9, not after V1"
    );
}

#[test]
fn role_statements_are_made_idempotent() {
    let sql = adapt_migration(
        "CREATE USER orca;\nGRANT CONNECT ON DATABASE ictunion TO orca;\nDROP ROLE x;",
    );
    assert_eq!(
        sql,
        "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'orca') \
         THEN CREATE USER orca; END IF; END $$;\n\
         GRANT CONNECT ON DATABASE orca_test_template TO orca;\n\
         DROP ROLE IF EXISTS x;"
    );
}
