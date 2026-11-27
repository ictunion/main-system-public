//! Step 3 of `docs/testing.md`: workplace reps reach only their own workplace.
//!
//! Every test arranges two workplaces, each with one current member and one
//! application in processing; the rep sits on the executive committee of the
//! first only. Anything of the second must be refused with the same 403 as a
//! caller holding no role.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use super::test_support::{ApplicationState as State, FakeProvider, TestApp};
use crate::server::oid::Role;

struct Workplace {
    id: Uuid,
    member: Uuid,
    application: Uuid,
}

struct Fixture {
    app: TestApp,
    own: Workplace,
    other: Workplace,
    /// Keycloak user sitting on `own`'s executive committee
    rep: Uuid,
}

impl Fixture {
    async fn new() -> Self {
        let app = TestApp::with_db().await;
        let own = arrange_workplace(&app, 1).await;
        let other = arrange_workplace(&app, 2).await;

        let rep = Uuid::new_v4();
        app.oid.add_to_group(rep, own.1);

        Self {
            app,
            own: own.0,
            other: other.0,
            rep,
        }
    }

    fn token(&self, roles: &[Role]) -> String {
        FakeProvider::token(self.rep, roles)
    }

    async fn status(&self, method: Method, uri: &str, roles: &[Role]) -> StatusCode {
        self.call(method, uri, roles).await.0
    }

    async fn call(&self, method: Method, uri: &str, roles: &[Role]) -> (StatusCode, Value) {
        let body = (method == Method::POST).then(|| json!({}));
        self.app
            .request(method, uri, Some(&self.token(roles)), body.as_ref())
            .await
    }
}

/// Workplace with one current member and one application in processing,
/// plus its executive group
async fn arrange_workplace(app: &TestApp, n: i32) -> (Workplace, Uuid) {
    let workplace = app.insert_workplace(&format!("workplace{n}")).await;
    let member = app
        .insert_member(&format!("member{n}"), Some(workplace.id))
        .await;
    let application = app
        .insert_application(
            &format!("applicant{n}"),
            Some(workplace.id),
            State::Processing,
        )
        .await;

    (
        Workplace {
            id: workplace.id,
            member,
            application,
        },
        workplace.keycloak_executive_group_id,
    )
}

/// Values of `key` in a JSON array response
fn ids(body: &Value, key: &str) -> Vec<String> {
    body.as_array()
        .expect("JSON array")
        .iter()
        .map(|item| item[key].as_str().expect("string id").to_owned())
        .collect()
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_reads_only_own_workplace() {
    let f = Fixture::new().await;
    let roles = &[Role::ListOwnWorkplace];
    let (own, other) = (f.own.id, f.other.id);

    let (status, mine) = f.call(Method::GET, "/workplaces/mine", roles).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mine["id"], json!(own));

    for uri in ["/workplaces/{id}", "/workplaces/{id}/representatives"] {
        let ok = uri.replace("{id}", &own.to_string());
        let refused = uri.replace("{id}", &other.to_string());
        assert_eq!(
            f.status(Method::GET, &ok, roles).await,
            StatusCode::OK,
            "{ok}"
        );
        assert_eq!(
            f.status(Method::GET, &refused, roles).await,
            StatusCode::FORBIDDEN,
            "{refused}"
        );
    }

    // On no committee at all
    let outsider = FakeProvider::token(Uuid::new_v4(), roles);
    let (status, _) = f
        .app
        .request(Method::GET, "/workplaces/mine", Some(&outsider), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_reads_only_own_members_redacted() {
    let f = Fixture::new().await;
    let roles = &[Role::ListOwnWorkplaceMembers];
    let (own, other) = (f.own.member, f.other.member);

    let (status, roster) = f.call(Method::GET, "/workplaces/mine/members", roles).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ids(&roster, "id"), [own.to_string()]);

    let (status, detail) = f.call(Method::GET, &format!("/members/{own}"), roles).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["email"], "member1@example.com");
    assert_eq!(detail["city"], Value::Null, "redacted for reps");
    assert_eq!(
        f.status(Method::GET, &format!("/members/{other}"), roles)
            .await,
        StatusCode::FORBIDDEN
    );

    // Notes: reps reach only the dues thread of their own members
    for (method, uri, expected) in [
        (
            Method::GET,
            format!("/notes/member_dues/{own}"),
            StatusCode::OK,
        ),
        (
            Method::GET,
            format!("/notes/member_dues/{other}"),
            StatusCode::FORBIDDEN,
        ),
        (
            Method::GET,
            format!("/notes/member/{own}"),
            StatusCode::FORBIDDEN,
        ),
        (Method::GET, "/notes/member_dues".to_owned(), StatusCode::OK),
        (
            Method::GET,
            "/notes/member".to_owned(),
            StatusCode::FORBIDDEN,
        ),
    ] {
        assert_eq!(f.status(method, &uri, roles).await, expected, "{uri}");
    }
}

/// Each `/applications/workplace/*` listing returns exactly the rep's own
/// application in that state, and the workplace stats count only those.
#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_lists_only_own_applications_by_state() {
    let f = Fixture::new().await;
    let roles = &[Role::ListOwnWorkplaceApplications];

    // The fixture already has one application in processing per workplace
    let mut own_ids = vec![f.own.application.to_string()];
    let mut expected = vec![("processing", vec![f.own.application.to_string()])];
    for (path, state) in [
        ("unverified", State::Unverified),
        ("accepted", State::Accepted),
        ("rejected", State::Rejected),
        ("invalid", State::Invalid),
    ] {
        let own = f
            .app
            .insert_application(&format!("{path}1"), Some(f.own.id), state)
            .await;
        f.app
            .insert_application(&format!("{path}2"), Some(f.other.id), state)
            .await;
        own_ids.push(own.to_string());
        expected.push((path, vec![own.to_string()]));
    }

    let (status, all) = f.call(Method::GET, "/applications/workplace", roles).await;
    assert_eq!(status, StatusCode::OK);
    let mut listed = ids(&all, "id");
    listed.sort();
    own_ids.sort();
    assert_eq!(listed, own_ids, "/applications/workplace");

    for (path, ids_in_state) in &expected {
        let uri = format!("/applications/workplace/{path}");
        let (status, list) = f.call(Method::GET, &uri, roles).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(&ids(&list, "id"), ids_in_state, "{uri}");
    }

    let (status, counts) = f
        .call(Method::GET, "/stats/applications/workplace/basic", roles)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        counts,
        json!({"unverified": 1, "accepted": 1, "rejected": 1, "processing": 1, "invalid": 1}),
        "only own workplace counted"
    );

    // On no committee: every listing is empty, not unbounded
    let outsider = FakeProvider::token(Uuid::new_v4(), roles);
    for path in [
        "",
        "/unverified",
        "/accepted",
        "/rejected",
        "/processing",
        "/invalid",
    ] {
        let uri = format!("/applications/workplace{path}");
        let (status, list) = f
            .app
            .request(Method::GET, &uri, Some(&outsider), None)
            .await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(list, json!([]), "{uri}");
    }
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_reads_only_own_applications_redacted() {
    let f = Fixture::new().await;
    let roles = &[Role::ListOwnWorkplaceApplications];
    let (own, other) = (f.own.application, f.other.application);

    let (status, detail) = f
        .call(Method::GET, &format!("/applications/{own}"), roles)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["email"], "applicant1@example.com");
    assert_eq!(detail["city"], Value::Null, "redacted for reps");

    let (status, workplace) = f
        .call(
            Method::GET,
            &format!("/applications/{own}/workplace"),
            roles,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(workplace["workplace"]["id"], json!(f.own.id));

    for uri in ["/applications/{id}", "/applications/{id}/workplace"] {
        let refused = uri.replace("{id}", &other.to_string());
        assert_eq!(
            f.status(Method::GET, &refused, roles).await,
            StatusCode::FORBIDDEN,
            "{refused}"
        );
    }
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_resolves_only_own_applications() {
    let mut f = Fixture::new().await;
    let roles = &[Role::ResolveOwnWorkplaceApplications];
    let (own, other) = (f.own.application, f.other.application);

    // Reps never accept or unreject, even in scope
    assert_eq!(
        f.status(Method::POST, &format!("/applications/{own}/accept"), roles)
            .await,
        StatusCode::FORBIDDEN
    );

    for method in [Method::PATCH, Method::DELETE] {
        let uri = match method {
            Method::PATCH => format!("/applications/{other}/approve"),
            _ => format!("/applications/{other}"),
        };
        assert_eq!(
            f.status(method, &uri, roles).await,
            StatusCode::FORBIDDEN,
            "{uri}"
        );
    }

    assert_eq!(
        f.status(
            Method::PATCH,
            &format!("/applications/{own}/approve"),
            roles
        )
        .await,
        StatusCode::ACCEPTED
    );

    let (status, rejected) = f
        .call(Method::DELETE, &format!("/applications/{own}"), roles)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(rejected["rejected_at"], Value::Null);
    assert_eq!(rejected["city"], Value::Null, "redacted for reps");

    assert_eq!(
        f.status(
            Method::PATCH,
            &format!("/applications/{own}/unreject"),
            roles
        )
        .await,
        StatusCode::FORBIDDEN
    );

    // The rejected application of the other workplace is untouched
    let admin = FakeProvider::token(Uuid::new_v4(), &[Role::ViewApplication]);
    let (_, untouched) = f
        .app
        .request(
            Method::GET,
            &format!("/applications/{other}"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(untouched["rejected_at"], Value::Null);
    f.app.drain_queue();
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn rep_resends_email_only_for_own_applications() {
    let mut f = Fixture::new().await;
    let roles = &[Role::ResolveOwnWorkplaceApplications];
    let own = f
        .app
        .insert_application("unverified1", Some(f.own.id), State::Unverified)
        .await;
    let other = f
        .app
        .insert_application("unverified2", Some(f.other.id), State::Unverified)
        .await;

    assert_eq!(
        f.status(
            Method::POST,
            &format!("/applications/{other}/resend-email"),
            roles
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.status(
            Method::POST,
            &format!("/applications/{own}/resend-email"),
            roles
        )
        .await,
        StatusCode::ACCEPTED
    );
    f.app.drain_queue();
}
