//! Who may call every route, checked in one table. A route missing from the
//! table fails `every_route_is_in_the_table`.
//!
//! Roles listed for a route are the ones that get past its guard: admin roles
//! and, for scoped routes, the scoped role (scope itself is tested per domain).
//! Any other role must be refused with 403 before the database is touched, so
//! the refusal checks run without one.

use std::collections::BTreeSet;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use super::test_support::{
    ALL_ROLES, BANK_SYNC_TOKEN, FakeProvider, NEWSLETTER_SYNC_TOKEN, TestApp,
};
use crate::server::oid::Role::{
    self, ListApplications, ListMembers, ListOwnWorkplace, ListOwnWorkplaceApplications,
    ListOwnWorkplaceMembers, ListWorkplaces, ManageMembers, ManageNotes, ManageWorkplaces,
    ResolveApplications, ResolveOwnWorkplaceApplications, SuperPowers, ViewApplication,
};

#[derive(Debug, Clone, Copy)]
enum Access {
    /// No token needed
    Public,
    /// Shared secret instead of a JWT
    SyncToken(&'static str),
    /// Any valid token, no role
    Authenticated,
    /// At least one of the roles
    Any(&'static [Role]),
    /// Every one of the roles
    All(&'static [Role]),
}

#[derive(Debug)]
struct Route {
    method: Method,
    /// Router pattern, e.g. `/members/{id}`
    pattern: &'static str,
    access: Access,
    /// A body that passes extraction, so the guard is what decides
    body: Option<fn() -> Value>,
}

const ID: &str = "00000000-0000-4000-8000-000000000001";

/// Roles whose reach is limited to the caller's own workplace(s)
const SCOPED_ROLES: &[Role] = &[
    ListOwnWorkplace,
    ListOwnWorkplaceMembers,
    ListOwnWorkplaceApplications,
    ResolveOwnWorkplaceApplications,
];

fn route(method: Method, pattern: &'static str, access: Access) -> Route {
    Route {
        method,
        pattern,
        access,
        body: None,
    }
}

fn with_body(method: Method, pattern: &'static str, access: Access, body: fn() -> Value) -> Route {
    Route {
        method,
        pattern,
        access,
        body: Some(body),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one line per route keeps the table easy to compare with the routers"
)]
fn routes() -> Vec<Route> {
    use Access::{All, Any, Authenticated, Public, SyncToken};
    use Method as M;

    let note = || json!({ "note": "text" });
    let member = || json!({ "email": "member@example.com", "language": "cs" });

    vec![
        route(M::GET, "/status", Public),
        // registration
        route(M::POST, "/registration/join", Public),
        route(M::GET, "/registration/{code}/confirm", Public),
        // applications
        route(M::GET, "/applications", Any(&[ListApplications])),
        route(M::GET, "/applications/unverified", Any(&[ListApplications])),
        route(M::GET, "/applications/accepted", Any(&[ListApplications])),
        route(M::GET, "/applications/rejected", Any(&[ListApplications])),
        route(M::GET, "/applications/processing", Any(&[ListApplications])),
        route(M::GET, "/applications/invalid", Any(&[ListApplications])),
        route(
            M::GET,
            "/applications/workplace",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/workplace/unverified",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/workplace/accepted",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/workplace/rejected",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/workplace/processing",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/workplace/invalid",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(
            M::GET,
            "/applications/{id}",
            Any(&[ViewApplication, ListOwnWorkplaceApplications]),
        ),
        route(
            M::DELETE,
            "/applications/{id}",
            Any(&[ResolveApplications, ResolveOwnWorkplaceApplications]),
        ),
        route(
            M::PATCH,
            "/applications/{id}/invalidate",
            Any(&[ResolveApplications]),
        ),
        route(
            M::PATCH,
            "/applications/{id}/unreject",
            Any(&[ResolveApplications]),
        ),
        route(
            M::PATCH,
            "/applications/{id}/uninvalidate",
            Any(&[ResolveApplications]),
        ),
        with_body(
            M::POST,
            "/applications/{id}/accept",
            Any(&[ResolveApplications]),
            || json!({}),
        ),
        // approving is the workplace's say: no admin role passes
        route(
            M::PATCH,
            "/applications/{id}/approve",
            Any(&[ResolveOwnWorkplaceApplications]),
        ),
        route(M::GET, "/applications/{id}/files", Any(&[ViewApplication])),
        route(
            M::PATCH,
            "/applications/{id}/verify",
            Any(&[ResolveApplications]),
        ),
        route(
            M::POST,
            "/applications/{id}/resend-email",
            Any(&[ResolveApplications, ResolveOwnWorkplaceApplications]),
        ),
        route(M::DELETE, "/applications/{id}/hard", Any(&[SuperPowers])),
        with_body(
            M::PATCH,
            "/applications/{id}/note",
            Any(&[ResolveApplications]),
            note,
        ),
        route(
            M::GET,
            "/applications/{id}/workplace",
            Any(&[ViewApplication, ListOwnWorkplaceApplications]),
        ),
        route(
            M::DELETE,
            "/applications/{id}/workplace",
            Any(&[ResolveApplications]),
        ),
        route(
            M::PUT,
            "/applications/{id}/workplace/{workplace_id}",
            Any(&[ResolveApplications]),
        ),
        // files: token in the query, see `token_query`
        route(
            M::GET,
            "/files/{id}",
            Any(&[ViewApplication, ListMembers, ListWorkplaces]),
        ),
        // members
        route(M::GET, "/members", Any(&[ListMembers])),
        with_body(M::POST, "/members", Any(&[ManageMembers]), member),
        route(M::GET, "/members/past", Any(&[ListMembers])),
        route(M::GET, "/members/new", Any(&[ListMembers])),
        route(M::GET, "/members/current", Any(&[ListMembers])),
        with_body(
            M::POST,
            "/members/{id}/send_email",
            Any(&[ManageMembers]),
            || json!({ "subject": "Subject", "body": "Body", "variables": {} }),
        ),
        route(
            M::GET,
            "/members/{id}",
            Any(&[ListMembers, ListOwnWorkplaceMembers]),
        ),
        with_body(M::PATCH, "/members/{id}", Any(&[ManageMembers]), member),
        route(M::DELETE, "/members/{id}", Any(&[ManageMembers])),
        route(
            M::PATCH,
            "/members/{id}/accept",
            Any(&[ResolveApplications]),
        ),
        route(M::GET, "/members/{id}/files", Any(&[ListMembers])),
        route(
            M::GET,
            "/members/{id}/occupations",
            Any(&[ListMembers, ListOwnWorkplaceMembers]),
        ),
        with_body(
            M::POST,
            "/members/{id}/occupations",
            Any(&[ManageMembers]),
            || json!({ "company_name": "ACME", "position": "Engineer", "source": "test" }),
        ),
        route(
            M::DELETE,
            "/members/{id}/occupations/{occupation_id}",
            Any(&[ManageMembers]),
        ),
        route(M::GET, "/members/offices", Any(&[ListMembers])),
        with_body(
            M::POST,
            "/members/offices",
            Any(&[ManageMembers]),
            || json!({ "member_id": ID, "title": "Chair", "elected_by": "Board", "elected_on": "2024-01-01" }),
        ),
        route(M::GET, "/members/offices/past", Any(&[ListMembers])),
        route(M::GET, "/members/offices/{id}", Any(&[ListMembers])),
        with_body(
            M::PATCH,
            "/members/offices/{id}",
            Any(&[ManageMembers]),
            || json!({}),
        ),
        route(M::DELETE, "/members/offices/{id}", Any(&[ManageMembers])),
        with_body(M::PATCH, "/members/{id}/note", Any(&[ManageMembers]), note),
        route(
            M::GET,
            "/members/{id}/list_candidate_users",
            Any(&[ManageMembers]),
        ),
        route(
            M::POST,
            "/members/{id}/create_oid_account",
            Any(&[ManageMembers]),
        ),
        route(
            M::PUT,
            "/members/{id}/oidc_groups/{group_id}",
            Any(&[SuperPowers]),
        ),
        with_body(
            M::PATCH,
            "/members/{id}/pair_oid",
            Any(&[ManageMembers]),
            || json!({ "sub": ID }),
        ),
        // notes, on the `member` target (staff only; reps reach `member_dues`)
        route(M::GET, "/notes/{target_type}", Any(&[ListMembers])),
        route(
            M::GET,
            "/notes/{target_type}/{target_id}",
            Any(&[ListMembers]),
        ),
        with_body(
            M::POST,
            "/notes/{target_type}/{target_id}",
            Any(&[ManageMembers]),
            || json!({ "body": "text" }),
        ),
        route(
            M::DELETE,
            "/notes/{target_type}/{target_id}",
            All(&[ManageMembers, ManageNotes]),
        ),
        with_body(
            M::PATCH,
            "/notes/{target_type}/{target_id}/{note_id}",
            Any(&[ManageMembers]),
            || json!({ "body": "text" }),
        ),
        route(
            M::DELETE,
            "/notes/{target_type}/{target_id}/{note_id}",
            Any(&[ManageMembers]),
        ),
        // oidc
        route(M::GET, "/oidc/{group_id}", Any(&[SuperPowers])),
        // session
        route(M::GET, "/session/current", Authenticated),
        route(M::POST, "/session/current/pair-by-email", Authenticated),
        // stats
        route(M::GET, "/stats/applications/basic", Authenticated),
        route(
            M::GET,
            "/stats/applications/workplace/basic",
            Any(&[ListOwnWorkplaceApplications]),
        ),
        route(M::GET, "/stats/members/basic", Authenticated),
        route(M::GET, "/stats/workplaces/basic", Authenticated),
        // sync
        route(M::GET, "/sync/bank/members", SyncToken(BANK_SYNC_TOKEN)),
        route(
            M::GET,
            "/sync/newsletter/members",
            SyncToken(NEWSLETTER_SYNC_TOKEN),
        ),
        // workplaces
        route(M::GET, "/workplaces", Any(&[ListWorkplaces])),
        with_body(
            M::POST,
            "/workplaces",
            Any(&[ManageWorkplaces]),
            || json!({ "name": "ACME", "email": "acme@example.com", "keycloak_group_id": ID, "keycloak_executive_group_id": ID }),
        ),
        route(M::GET, "/workplaces/assignable", Any(&[ListWorkplaces])),
        route(M::GET, "/workplaces/mine", Any(&[ListOwnWorkplace])),
        route(
            M::GET,
            "/workplaces/mine/members",
            Any(&[ListOwnWorkplaceMembers]),
        ),
        route(M::GET, "/workplaces/active", Any(&[ManageWorkplaces])),
        route(
            M::GET,
            "/workplaces/in_preparation",
            Any(&[ManageWorkplaces]),
        ),
        route(M::GET, "/workplaces/inactive", Any(&[ManageWorkplaces])),
        route(
            M::GET,
            "/workplaces/{workplace_id}",
            Any(&[ListWorkplaces, ListOwnWorkplace]),
        ),
        with_body(
            M::PATCH,
            "/workplaces/{workplace_id}",
            Any(&[ManageWorkplaces]),
            || json!({}),
        ),
        route(
            M::GET,
            "/workplaces/{workplace_id}/representatives",
            Any(&[ListWorkplaces, ListOwnWorkplace]),
        ),
        route(
            M::PATCH,
            "/workplaces/{workplace_id}/establish",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::PATCH,
            "/workplaces/{workplace_id}/announce",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::PATCH,
            "/workplaces/{workplace_id}/cancel",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::PUT,
            "/workplaces/{workplace_id}/members/{member_id}",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::DELETE,
            "/workplaces/{workplace_id}/members/{member_id}",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::PUT,
            "/workplaces/{workplace_id}/members/{member_id}/is_representative",
            Any(&[ManageWorkplaces]),
        ),
        route(
            M::GET,
            "/workplaces/{workplace_id}/members",
            All(&[ListWorkplaces, ListMembers]),
        ),
        route(
            M::GET,
            "/workplaces/{workplace_id}/files",
            Any(&[ListWorkplaces]),
        ),
        with_body(
            M::POST,
            "/workplaces/{workplace_id}/files",
            Any(&[ManageWorkplaces]),
            || json!({ "document_type": "member_confirmation", "workplace_short_name": "ACME", "signature_date": "2026-01-01" }),
        ),
    ]
}

impl Route {
    /// Concrete URI: ids are a valid UUID, the note target is `member`
    fn uri(&self) -> String {
        self.pattern
            .split('/')
            .map(|segment| match segment {
                "{target_type}" => "member",
                "{code}" => "code",
                s if s.starts_with('{') => ID,
                s => s,
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// `/files` reads the token from `?token=` so files open in a browser tab
    fn token_query(&self) -> bool {
        self.pattern.starts_with("/files/")
    }

    async fn send(&self, app: &TestApp, token: Option<&str>) -> StatusCode {
        let body = self.body.map(|body| body());
        let (uri, header) = match token {
            Some(token) if self.token_query() => (format!("{}?token={token}", self.uri()), None),
            token => (self.uri(), token),
        };
        app.request(self.method.clone(), &uri, header, body.as_ref())
            .await
            .0
    }

    fn label(&self) -> String {
        format!("{} {}", self.method, self.pattern)
    }
}

fn token(roles: &[Role]) -> String {
    FakeProvider::token(Uuid::new_v4(), roles)
}

/// Every `(method, path)` the routers declare, read from their source
fn declared_routes() -> BTreeSet<String> {
    let sources = [
        ("", include_str!("mod.rs")),
        ("/applications", include_str!("applications/mod.rs")),
        ("/files", include_str!("files/mod.rs")),
        ("/members", include_str!("members/mod.rs")),
        ("/notes", include_str!("notes/mod.rs")),
        ("/oidc", include_str!("oidc/mod.rs")),
        ("/registration", include_str!("registration/mod.rs")),
        ("/session", include_str!("session/mod.rs")),
        ("/stats", include_str!("stats/mod.rs")),
        ("/sync", include_str!("sync/mod.rs")),
        ("/workplaces", include_str!("workplaces/mod.rs")),
    ];

    let mut declared = BTreeSet::new();
    for (prefix, source) in sources {
        for call in source.split(".route(").skip(1) {
            let call = &call[..closing_paren(call)];
            let path = call.split('"').nth(1).expect("route path literal");
            let path = format!("{prefix}{path}");
            let path = path
                .strip_suffix('/')
                .filter(|p| !p.is_empty())
                .unwrap_or(&path);
            for method in ["get", "post", "put", "patch", "delete"] {
                let used = call.match_indices(&format!("{method}(")).any(|(at, _)| {
                    at == 0
                        || !call.as_bytes()[at - 1].is_ascii_alphanumeric()
                            && call.as_bytes()[at - 1] != b'_'
                });
                if used {
                    declared.insert(format!("{} {path}", method.to_uppercase()));
                }
            }
        }
    }
    declared
}

/// Index of the `)` closing the `.route(` call `rest` starts in
fn closing_paren(rest: &str) -> usize {
    let mut depth = 1;
    for (at, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced .route( call");
}

#[test]
fn every_route_is_in_the_table() {
    let table: BTreeSet<String> = routes().iter().map(Route::label).collect();
    assert_eq!(routes().len(), table.len(), "a route is listed twice");
    assert_eq!(declared_routes(), table);
}

#[tokio::test]
async fn missing_token_is_401() {
    let app = TestApp::without_db();
    for route in routes() {
        if matches!(route.access, Access::Public) {
            continue;
        }
        let status = route.send(&app, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{}", route.label());
    }
}

#[tokio::test]
async fn invalid_token_is_401() {
    let app = TestApp::without_db();
    for route in routes() {
        if matches!(route.access, Access::Public) {
            continue;
        }
        let status = route.send(&app, Some("not-a-token")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{}", route.label());
    }
}

#[tokio::test]
async fn user_token_does_not_open_sync() {
    let app = TestApp::without_db();
    for route in routes() {
        if let Access::SyncToken(_) = route.access {
            let status = route.send(&app, Some(&token(ALL_ROLES))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{}", route.label());
        }
    }
}

#[tokio::test]
async fn other_roles_are_refused_with_403() {
    let app = TestApp::without_db();
    for route in routes() {
        let (Access::Any(allowed) | Access::All(allowed)) = route.access else {
            continue;
        };
        let refused: Vec<&[Role]> = std::iter::once(&[][..])
            .chain(
                ALL_ROLES
                    .iter()
                    .filter(|role| {
                        matches!(route.access, Access::All(_)) || !allowed.contains(role)
                    })
                    .map(std::slice::from_ref),
            )
            .collect();
        for roles in refused {
            let status = route.send(&app, Some(&token(roles))).await;
            assert_eq!(
                status,
                StatusCode::FORBIDDEN,
                "{} with {roles:?}",
                route.label()
            );
        }
    }
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn allowed_callers_pass_the_guard() {
    let mut app = TestApp::with_db().await;
    for route in routes() {
        let token = match route.access {
            Access::Public => None,
            Access::SyncToken(secret) => Some(secret.to_string()),
            Access::Authenticated => Some(token(&[])),
            // Open to workplace reps only: whether a caller passes depends on
            // their workplace, which the scope tests arrange
            Access::Any(roles) if roles.iter().all(|role| SCOPED_ROLES.contains(role)) => {
                continue;
            }
            Access::Any(roles) | Access::All(roles) => Some(token(roles)),
        };
        let status = route.send(&app, token.as_deref()).await;
        app.drain_queue();
        assert!(
            status != StatusCode::UNAUTHORIZED && status != StatusCode::FORBIDDEN,
            "{} got {status}",
            route.label()
        );
    }
}
