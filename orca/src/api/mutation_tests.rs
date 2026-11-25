//! Step 4 of `docs/testing.md`: one success test per mutating route.
//!
//! A `Detail` struct with a field missing from `UPDATE … RETURNING` (or a
//! missing `GRANT` to `orca`) only fails at runtime, as a 500. Each test runs
//! the mutation as an admin holding every role and checks the status plus the
//! field the mutation is about, and side effects (queued commands, Keycloak
//! calls) where the handler has them.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use super::test_support::{ADMIN_SUB, ApplicationState as State, FakeProvider, TestApp};

// applications

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_reject() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Processing).await;

    let (status, body) = app
        .admin_call(Method::DELETE, &format!("/applications/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["rejected_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_invalidate() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Processing).await;

    let uri = format!("/applications/{id}/invalidate");
    let (status, body) = app.admin_call(Method::PATCH, &uri, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["invalidated_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_unreject() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Rejected).await;

    let uri = format!("/applications/{id}/unreject");
    let (status, body) = app.admin_call(Method::PATCH, &uri, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["rejected_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_uninvalidate() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Invalid).await;

    let uri = format!("/applications/{id}/uninvalidate");
    let (status, body) = app.admin_call(Method::PATCH, &uri, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["invalidated_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_accept() {
    let mut app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;
    let id = app
        .insert_application("a", Some(workplace.id), State::Processing)
        .await;

    let uri = format!("/applications/{id}/accept");
    let (status, body) = app.admin_call(Method::POST, &uri, Some(json!({}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "a@example.com");

    let member: Uuid =
        sqlx::query_scalar("SELECT id FROM members WHERE registration_request_id = $1")
            .bind(id)
            .fetch_one(app.admin())
            .await
            .expect("member created from the application");
    let in_workplace = app
        .count(
            "SELECT count(*) FROM members_workplaces WHERE member_id = $1",
            member,
        )
        .await;
    assert_eq!(in_workplace, 1, "workplace copied to the member");
    assert_eq!(
        app.queued(),
        [
            format!("NewMemberCreated id: {member}"),
            "SendNotificationToTreasurer".to_owned()
        ]
    );
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_verify() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Unverified).await;

    let uri = format!("/applications/{id}/verify");
    let (status, body) = app.admin_call(Method::PATCH, &uri, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["confirmed_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_resend_email() {
    let mut app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Unverified).await;

    let uri = format!("/applications/{id}/resend-email");
    assert_eq!(
        app.admin_status(Method::POST, &uri, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(app.queued(), [format!("ResentRegistrationEmail id: {id}")]);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_hard_delete() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Invalid).await;

    let uri = format!("/applications/{id}/hard");
    assert_eq!(
        app.admin_status(Method::DELETE, &uri, None).await,
        StatusCode::ACCEPTED
    );
    let left = app
        .count(
            "SELECT count(*) FROM registration_requests WHERE id = $1",
            id,
        )
        .await;
    assert_eq!(left, 0);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_note() {
    let app = TestApp::with_db().await;
    let id = app.insert_application("a", None, State::Processing).await;

    let uri = format!("/applications/{id}/note");
    let (status, body) = app
        .admin_call(Method::PATCH, &uri, Some(json!({ "note": "text" })))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["note"], "text");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_assign_and_unassign_workplace() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;
    let id = app.insert_application("a", None, State::Processing).await;

    let assign = format!("/applications/{id}/workplace/{}", workplace.id);
    assert_eq!(
        app.admin_status(Method::PUT, &assign, None).await,
        StatusCode::CREATED
    );
    assert_eq!(
        app.admin_status(Method::PUT, &assign, None).await,
        StatusCode::NO_CONTENT,
        "assigning again changes nothing"
    );
    let (status_code, body) = app
        .admin_call(Method::GET, &format!("/applications/{id}/workplace"), None)
        .await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["workplace"]["id"], json!(workplace.id));

    let unassign = format!("/applications/{id}/workplace");
    assert_eq!(
        app.admin_status(Method::DELETE, &unassign, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        app.admin_status(Method::DELETE, &unassign, None).await,
        StatusCode::NO_CONTENT
    );
}

// members

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_create() {
    let app = TestApp::with_db().await;

    let new = json!({ "email": "new@example.com", "first_name": "New", "language": "cs" });
    let (status, body) = app.admin_call(Method::POST, "/members", Some(new)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "new@example.com");
    assert_eq!(body["member_number"], 1);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_send_email() {
    let mut app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let email = json!({ "subject": "Hi", "body": "Hello {{name}}", "variables": { "name": "M" } });
    let uri = format!("/members/{id}/send_email");
    assert_eq!(
        app.admin_status(Method::POST, &uri, Some(email)).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        app.queued(),
        ["SendEmailAsTreasurer to member with email: m@example.com"]
    );
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_update() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let update = json!({ "email": "changed@example.com", "city": "Praha", "language": "en" });
    let (status, body) = app
        .admin_call(Method::PATCH, &format!("/members/{id}"), Some(update))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "changed@example.com");
    assert_eq!(body["city"], "Praha");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_remove() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;
    let id = app.insert_member("m", Some(workplace.id)).await;
    let sub = app.set_sub(id).await;

    let (status, body) = app
        .admin_call(Method::DELETE, &format!("/members/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["left_at"], Value::Null);
    assert!(app.oid.calls().contains(&format!("remove_user {sub}")));
    let in_workplace = app
        .count(
            "SELECT count(*) FROM members_workplaces WHERE member_id = $1",
            id,
        )
        .await;
    assert_eq!(in_workplace, 0, "workplace associations removed");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_accept() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let (status, body) = app
        .admin_call(Method::PATCH, &format!("/members/{id}/accept"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(body["onboarding_finished_at"], Value::Null);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_occupation_create_and_delete() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let occupation = json!({ "company_name": "ACME", "position": "Engineer", "source": "orca" });
    let uri = format!("/members/{id}/occupations");
    let (status_code, body) = app.admin_call(Method::POST, &uri, Some(occupation)).await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["company_name"], "ACME");

    let occupation_id = body["id"].as_str().expect("occupation id");
    let uri = format!("/members/{id}/occupations/{occupation_id}");
    assert_eq!(
        app.admin_status(Method::DELETE, &uri, None).await,
        StatusCode::ACCEPTED
    );
    let (_, list) = app
        .admin_call(Method::GET, &format!("/members/{id}/occupations"), None)
        .await;
    assert_eq!(list, json!([]));
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_office_create_update_delete() {
    let app = TestApp::with_db().await;
    let member = app.insert_member("m", None).await;

    let office = json!({
        "member_id": member,
        "title": "Chair",
        "elected_by": "Assembly",
        "elected_on": "2026-01-01",
    });
    let (status_code, body) = app
        .admin_call(Method::POST, "/members/offices", Some(office))
        .await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["title"], "Chair");
    let id = body["id"].as_str().expect("office id").to_owned();

    let uri = format!("/members/offices/{id}");
    let patch = json!({ "title": "Vice chair", "term_ends_on": "2027-01-01" });
    let (status_code, body) = app.admin_call(Method::PATCH, &uri, Some(patch)).await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["title"], "Vice chair");
    assert_eq!(body["term_ends_on"], "2027-01-01");

    assert_eq!(
        app.admin_status(Method::DELETE, &uri, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        app.admin_status(Method::GET, &uri, None).await,
        StatusCode::NOT_FOUND,
        "deleted offices are hidden"
    );
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_note() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let uri = format!("/members/{id}/note");
    let (status, body) = app
        .admin_call(Method::PATCH, &uri, Some(json!({ "note": "text" })))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["note"], "text");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_create_oid_account() {
    let mut app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;

    let uri = format!("/members/{id}/create_oid_account");
    assert_eq!(
        app.admin_status(Method::POST, &uri, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(app.queued(), [format!("NewMemberCreated id: {id}")]);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_add_to_oid_group() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;
    let sub = app.set_sub(id).await;
    let group = Uuid::new_v4();

    let uri = format!("/members/{id}/oidc_groups/{group}");
    assert_eq!(
        app.admin_status(Method::PUT, &uri, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(app.oid.calls(), [format!("connect {sub} {group}")]);
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_pair_oid() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;
    let sub = Uuid::new_v4();

    let uri = format!("/members/{id}/pair_oid");
    let (status, body) = app
        .admin_call(Method::PATCH, &uri, Some(json!({ "sub": sub })))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["sub"], json!(sub));
}

// workplaces

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_create() {
    let app = TestApp::with_db().await;

    let new = json!({
        "name": "ACME",
        "email": "acme@example.com",
        "keycloak_group_id": Uuid::new_v4(),
        "keycloak_executive_group_id": Uuid::new_v4(),
        "company_id": "  ",
    });
    let (status, body) = app.admin_call(Method::POST, "/workplaces", Some(new)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "ACME");
    assert_eq!(body["company_id"], Value::Null, "blank text is not stored");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_update() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;

    let update = json!({ "company_id": " 12345678 ", "workplace_short_name": "W" });
    let uri = format!("/workplaces/{}", workplace.id);
    let (status, body) = app.admin_call(Method::PATCH, &uri, Some(update)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["company_id"], "12345678");
    assert_eq!(body["workplace_short_name"], "W");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_establish_announce_cancel() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;

    for (action, field) in [
        ("establish", "established_at"),
        ("announce", "announced_at"),
        ("cancel", "cancelled_at"),
    ] {
        let uri = format!("/workplaces/{}/{action}", workplace.id);
        let (status, body) = app.admin_call(Method::PATCH, &uri, None).await;
        assert_eq!(status, StatusCode::OK, "{action}: {body}");
        assert_ne!(body[field], Value::Null, "{action}");
    }
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_member_assign_promote_remove() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;
    let member = app.insert_member("m", None).await;
    let sub = app.set_sub(member).await;
    let (group, executive) = (
        workplace.keycloak_group_id,
        workplace.keycloak_executive_group_id,
    );
    let uri = format!("/workplaces/{}/members/{member}", workplace.id);

    assert_eq!(
        app.admin_status(Method::PUT, &uri, None).await,
        StatusCode::CREATED
    );
    let rep_uri = format!("{uri}/is_representative");
    assert_eq!(
        app.admin_status(Method::PUT, &rep_uri, None).await,
        StatusCode::CREATED
    );
    let (_, reps) = app
        .admin_call(
            Method::GET,
            &format!("/workplaces/{}/representatives", workplace.id),
            None,
        )
        .await;
    assert_eq!(reps[0]["id"], json!(member), "{reps}");

    assert_eq!(
        app.admin_status(Method::DELETE, &uri, None).await,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        app.oid.calls(),
        [
            format!("connect {sub} {group}"),
            format!("connect {sub} {group}"),
            format!("connect {sub} {executive}"),
            format!("disconnect {sub} {group}"),
            format!("disconnect {sub} {executive}"),
        ]
    );
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_create_document() {
    let mut app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await;
    let member = app.insert_member("m", Some(workplace.id)).await;
    sqlx::query(
        "UPDATE members_workplaces SET became_representative_at = now() WHERE member_id = $1",
    )
    .bind(member)
    .execute(app.admin())
    .await
    .expect("make the member a representative");

    let document = json!({ "document_type": "member_confirmation", "workplace_short_name": "W" });
    let uri = format!("/workplaces/{}/files", workplace.id);
    let (status, body) = app.admin_call(Method::POST, &uri, Some(document)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let queued = app.queued();
    assert_eq!(queued.len(), 1, "{queued:?}");
    assert!(queued[0].starts_with("NewWorkplaceDocument"), "{queued:?}");
}

// notes

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn note_create_update_delete_clear() {
    let app = TestApp::with_db().await;
    let member = app.insert_member("m", None).await;
    let thread = format!("/notes/member/{member}");

    let (status_code, body) = app
        .admin_call(Method::POST, &thread, Some(json!({ "body": " first " })))
        .await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["body"], "first");
    assert_eq!(body["author_sub"], json!(ADMIN_SUB));
    let note = format!("{thread}/{}", body["id"].as_str().expect("note id"));

    let (status_code, body) = app
        .admin_call(Method::PATCH, &note, Some(json!({ "body": "edited" })))
        .await;
    assert_eq!(status_code, StatusCode::OK, "{body}");
    assert_eq!(body["body"], "edited");

    assert_eq!(
        app.admin_status(Method::DELETE, &note, None).await,
        StatusCode::ACCEPTED
    );

    for text in ["one", "two"] {
        app.admin_call(Method::POST, &thread, Some(json!({ "body": text })))
            .await;
    }
    assert_eq!(
        app.admin_status(Method::DELETE, &thread, None).await,
        StatusCode::ACCEPTED
    );
    let (_, list) = app.admin_call(Method::GET, &thread, None).await;
    assert_eq!(list, json!([]));
}

// session

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn session_pair_by_email() {
    let app = TestApp::with_db().await;
    let user = Uuid::new_v4();
    // The token's email is `<sub>@example.com`
    let member = app.insert_member(&user.to_string(), None).await;

    let token = FakeProvider::token(user, &[]);
    let (status, body) = app
        .request(
            Method::POST,
            "/session/current/pair-by-email",
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["member_id"], json!(member));
}

// registration

/// 1×1 PNG, see `docs/useful-examples.md`
const SIGNATURE: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn registration_join_and_confirm() {
    let mut app = TestApp::with_db().await;

    let form = json!({
        "email": "applicant@example.com",
        "first_name": "Applicant",
        "last_name": "Test",
        "date_of_birth": "1990-01-01",
        "city": "Praha",
        "phone_number": "+420123456789",
        "company_name": "ACME",
        "occupation": "Engineer",
        "local": "cz",
        "signature": SIGNATURE,
    });
    let (status, body) = app
        .request(Method::POST, "/registration/join", None, Some(&form))
        .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");

    let (id, code): (Uuid, String) =
        sqlx::query_as("SELECT id, confirmation_token FROM registration_requests WHERE email = $1")
            .bind("applicant@example.com")
            .fetch_one(app.admin())
            .await
            .expect("application stored");
    assert_eq!(app.queued(), [format!("NewRegistrationRequest id: {id}")]);

    let (status, _) = app
        .request(
            Method::GET,
            &format!("/registration/{code}/confirm"),
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FOUND);
    let confirmed = app
        .count(
            "SELECT count(*) FROM registration_requests WHERE id = $1 AND confirmed_at IS NOT NULL",
            id,
        )
        .await;
    assert_eq!(confirmed, 1);
}
