//! Step 5 of `docs/testing.md`: the branches handlers take besides success —
//! state conflicts (409), missing records (404), invalid input (422/400), and
//! requests refused before any handler logic (400/404/405/413/415).

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use super::test_support::{ADMIN_SUB, ALL_ROLES, ApplicationState as State, FakeProvider, TestApp};
use crate::server::oid::Role;

/// Asserts each `(method, uri, body)` answers `expected`
async fn expect_all(
    app: &TestApp,
    expected: StatusCode,
    cases: Vec<(Method, String, Option<Value>)>,
) {
    for (method, uri, body) in cases {
        let (status, response) = app.admin_call(method.clone(), &uri, body).await;
        assert_eq!(status, expected, "{method} {uri}: {response}");
    }
}

// applications

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_actions_in_the_wrong_state_conflict() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await.id;

    let table: &[(State, Method, &str)] = &[
        (State::Unverified, Method::POST, "/accept"),
        (State::Processing, Method::PATCH, "/unreject"),
        (State::Processing, Method::PATCH, "/uninvalidate"),
        (State::Processing, Method::PATCH, "/verify"),
        (State::Processing, Method::POST, "/resend-email"),
        (State::Processing, Method::DELETE, "/hard"),
        (State::Rejected, Method::DELETE, ""),
        (State::Rejected, Method::PATCH, "/invalidate"),
        (State::Rejected, Method::PATCH, "/uninvalidate"),
        (State::Rejected, Method::POST, "/accept"),
        (State::Invalid, Method::PATCH, "/invalidate"),
        (State::Invalid, Method::PATCH, "/unreject"),
        (State::Invalid, Method::POST, "/accept"),
        (State::Accepted, Method::DELETE, ""),
        (State::Accepted, Method::PATCH, "/invalidate"),
        (State::Accepted, Method::POST, "/accept"),
        (State::Accepted, Method::DELETE, "/workplace"),
        (State::Accepted, Method::PUT, "/workplace/{workplace}"),
    ];

    let mut cases = Vec::new();
    for (n, (state, method, suffix)) in table.iter().enumerate() {
        let id = app.insert_application(&format!("a{n}"), None, *state).await;
        let suffix = suffix.replace("{workplace}", &workplace.to_string());
        let body = (*method == Method::POST).then(|| json!({}));
        cases.push((method.clone(), format!("/applications/{id}{suffix}"), body));
    }
    expect_all(&app, StatusCode::CONFLICT, cases).await;
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_accept_with_taken_member_number_conflicts() {
    let app = TestApp::with_db().await;
    app.insert_member("m", None).await; // member number 1
    let id = app.insert_application("a", None, State::Processing).await;

    let uri = format!("/applications/{id}/accept");
    let status = app
        .admin_status(Method::POST, &uri, Some(json!({ "member_number": 1 })))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let members = app
        .count(
            "SELECT count(*) FROM members WHERE registration_request_id = $1",
            id,
        )
        .await;
    assert_eq!(members, 0, "nothing created");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn application_missing_records_are_404() {
    let app = TestApp::with_db().await;
    let missing = Uuid::new_v4();
    let id = app.insert_application("a", None, State::Processing).await;

    expect_all(
        &app,
        StatusCode::NOT_FOUND,
        vec![
            (Method::GET, format!("/applications/{missing}"), None),
            (Method::DELETE, format!("/applications/{missing}"), None),
            (
                Method::PATCH,
                format!("/applications/{missing}/verify"),
                None,
            ),
            (
                Method::POST,
                format!("/applications/{missing}/accept"),
                Some(json!({})),
            ),
            // unknown workplace, not a 500 from the foreign key
            (
                Method::PUT,
                format!("/applications/{id}/workplace/{missing}"),
                None,
            ),
            // no workplace assigned
            (Method::GET, format!("/applications/{id}/workplace"), None),
        ],
    )
    .await;
}

// members

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_actions_in_the_wrong_state_conflict() {
    let app = TestApp::with_db().await;
    let onboarded = app.insert_member("onboarded", None).await;
    let past = app.insert_member("past", None).await;
    let paired = app.insert_member("paired", None).await;
    let unpaired = app.insert_member("unpaired", None).await;
    app.set_sub(paired).await;
    for (id, column) in [(onboarded, "onboarding_finished_at"), (past, "left_at")] {
        sqlx::query(&format!(
            "UPDATE members SET {column} = now() WHERE id = $1"
        ))
        .bind(id)
        .execute(app.admin())
        .await
        .expect("arrange member state");
    }
    let group = Uuid::new_v4();

    expect_all(
        &app,
        StatusCode::CONFLICT,
        vec![
            (Method::PATCH, format!("/members/{onboarded}/accept"), None),
            (Method::PATCH, format!("/members/{past}/accept"), None),
            (Method::DELETE, format!("/members/{past}"), None),
            (
                Method::POST,
                format!("/members/{paired}/create_oid_account"),
                None,
            ),
            (
                Method::PUT,
                format!("/members/{unpaired}/oidc_groups/{group}"),
                None,
            ),
            (
                Method::POST,
                "/members".to_owned(),
                Some(json!({ "email": "x@example.com", "language": "cs", "member_number": 1 })),
            ),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn member_invalid_input_and_missing_records() {
    let app = TestApp::with_db().await;
    let id = app.insert_member("m", None).await;
    let missing = Uuid::new_v4();

    expect_all(
        &app,
        StatusCode::UNPROCESSABLE_ENTITY,
        vec![
            (
                Method::POST,
                "/members".to_owned(),
                Some(json!({ "email": "not-an-email", "language": "cs" })),
            ),
            (Method::GET, "/members/not-a-uuid".to_owned(), None),
            (
                Method::POST,
                format!("/members/{id}/occupations"),
                Some(json!({ "company_name": " ", "position": "Engineer", "source": "orca" })),
            ),
        ],
    )
    .await;
    assert_eq!(
        app.admin_status(
            Method::PUT,
            &format!("/members/{id}/oidc_groups/not-a-uuid"),
            None
        )
        .await,
        StatusCode::BAD_REQUEST
    );
    expect_all(
        &app,
        StatusCode::NOT_FOUND,
        vec![
            (Method::GET, format!("/members/{missing}"), None),
            (Method::PATCH, format!("/members/{missing}/accept"), None),
            (Method::DELETE, format!("/members/{missing}"), None),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn office_term_must_not_end_before_election() {
    let app = TestApp::with_db().await;
    let member = app.insert_member("m", None).await;
    let office = |elected_on: &str, term_ends_on: &str| {
        json!({
            "member_id": member,
            "title": "Chair",
            "elected_by": "Assembly",
            "elected_on": elected_on,
            "term_ends_on": term_ends_on,
        })
    };

    let status = app
        .admin_status(
            Method::POST,
            "/members/offices",
            Some(office("2026-01-01", "2025-01-01")),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, current) = app
        .admin_call(
            Method::POST,
            "/members/offices",
            Some(office("2026-01-01", "2099-01-01")),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{current}");
    let uri = format!(
        "/members/offices/{}",
        current["id"].as_str().expect("office id")
    );
    let status = app
        .admin_status(
            Method::PATCH,
            &uri,
            Some(json!({ "term_ends_on": "2025-01-01" })),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "check constraint");

    // Past offices are read-only
    let (status, past) = app
        .admin_call(
            Method::POST,
            "/members/offices",
            Some(office("2020-01-01", "2021-01-01")),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{past}");
    let uri = format!(
        "/members/offices/{}",
        past["id"].as_str().expect("office id")
    );
    let status = app
        .admin_status(Method::PATCH, &uri, Some(json!({ "title": "Vice chair" })))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// workplaces

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_lifecycle_out_of_order_conflicts() {
    let app = TestApp::with_db().await;
    let fresh = app.insert_workplace("fresh").await.id;
    let established = app.insert_workplace("established").await.id;
    let cancelled = app.insert_workplace("cancelled").await.id;
    for (id, action) in [(established, "establish"), (cancelled, "cancel")] {
        let status = app
            .admin_status(Method::PATCH, &format!("/workplaces/{id}/{action}"), None)
            .await;
        assert_eq!(status, StatusCode::OK, "arrange {action}");
    }

    expect_all(
        &app,
        StatusCode::CONFLICT,
        vec![
            (Method::PATCH, format!("/workplaces/{fresh}/announce"), None),
            (
                Method::PATCH,
                format!("/workplaces/{established}/establish"),
                None,
            ),
            (
                Method::PATCH,
                format!("/workplaces/{cancelled}/cancel"),
                None,
            ),
            (
                Method::PATCH,
                format!("/workplaces/{cancelled}/establish"),
                None,
            ),
        ],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_member_without_keycloak_account_is_still_assigned() {
    let app = TestApp::with_db().await;
    let workplace = app.insert_workplace("w").await.id;
    let member = app.insert_member("m", None).await;

    let uri = format!("/workplaces/{workplace}/members/{member}");
    let status = app.admin_status(Method::PUT, &uri, None).await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Keycloak push failed"
    );
    let assigned = app
        .count(
            "SELECT count(*) FROM members_workplaces WHERE member_id = $1",
            member,
        )
        .await;
    assert_eq!(assigned, 1, "stored before the Keycloak calls");
}

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn workplace_document_preconditions() {
    let app = TestApp::with_db().await;
    let without_reps = app.insert_workplace("without-reps").await.id;
    let with_rep = app.insert_workplace("with-rep").await.id;
    let rep = app.insert_member("rep", Some(with_rep)).await;
    sqlx::query(
        "UPDATE members_workplaces SET became_representative_at = now() WHERE member_id = $1",
    )
    .bind(rep)
    .execute(app.admin())
    .await
    .expect("make the member a representative");
    let confirmation =
        || json!({ "document_type": "member_confirmation", "workplace_short_name": "W" });
    let entrustment = json!({
        "document_type": "entrustment",
        "workplace_short_name": "W",
        "meeting_date": "2026-01-01",
        "company_address": "Street 1",
        "company_id": "12345678",
        "removed_member_ids": [Uuid::new_v4()],
    });

    expect_all(
        &app,
        StatusCode::CONFLICT,
        vec![
            (
                Method::POST,
                format!("/workplaces/{without_reps}/files"),
                Some(confirmation()),
            ),
            (
                Method::POST,
                format!("/workplaces/{with_rep}/files"),
                Some(entrustment),
            ),
        ],
    )
    .await;
    let status = app
        .admin_status(
            Method::POST,
            &format!("/workplaces/{}/files", Uuid::new_v4()),
            Some(confirmation()),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let status = app
        .admin_status(
            Method::POST,
            &format!("/workplaces/{with_rep}/files"),
            Some(json!({ "document_type": "member_confirmation" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// notes

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn note_edge_cases() {
    let app = TestApp::with_db().await;
    let member = app.insert_member("m", None).await;
    let other = app.insert_member("other", None).await;
    let thread = format!("/notes/member/{member}");

    expect_all(
        &app,
        StatusCode::NOT_FOUND,
        vec![(
            Method::POST,
            format!("/notes/member/{}", Uuid::new_v4()),
            Some(json!({ "body": "text" })),
        )],
    )
    .await;
    let status = app
        .admin_status(Method::POST, &thread, Some(json!({ "body": "   " })))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let author = FakeProvider::token(Uuid::new_v4(), &[Role::ManageMembers]);
    let (status, note) = app
        .request(
            Method::POST,
            &thread,
            Some(&author),
            Some(&json!({ "body": "mine" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{note}");
    let note_id = note["id"].as_str().expect("note id");
    let edit = json!({ "body": "edited" });

    // Reached through another member's thread
    let status = app
        .admin_status(
            Method::PATCH,
            &format!("/notes/member/{other}/{note_id}"),
            Some(edit.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Someone else's note needs `manage-notes`
    let colleague = FakeProvider::token(Uuid::new_v4(), &[Role::ManageMembers]);
    let (status, _) = app
        .request(
            Method::PATCH,
            &format!("{thread}/{note_id}"),
            Some(&colleague),
            Some(&edit),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let status = app
        .admin_status(Method::PATCH, &format!("{thread}/{note_id}"), Some(edit))
        .await;
    assert_eq!(status, StatusCode::OK, "admin holds manage-notes");
}

// session

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn session_pair_by_email_edge_cases() {
    let app = TestApp::with_db().await;
    let paired = Uuid::new_v4();
    let member = app.insert_member(&paired.to_string(), None).await;
    sqlx::query("UPDATE members SET sub = $1 WHERE id = $2")
        .bind(paired)
        .bind(member)
        .execute(app.admin())
        .await
        .expect("pair the member");

    for (user, expected) in [
        (paired, StatusCode::CONFLICT),
        // no member with the token's email
        (Uuid::new_v4(), StatusCode::NOT_FOUND),
    ] {
        let token = FakeProvider::token(user, &[]);
        let (status, body) = app
            .request(
                Method::POST,
                "/session/current/pair-by-email",
                Some(&token),
                None,
            )
            .await;
        assert_eq!(status, expected, "{body}");
    }
}

// registration

#[tokio::test]
#[ignore = "needs postgres, see docs/testing.md"]
async fn registration_invalid_input() {
    let mut app = TestApp::with_db().await;
    let form = |email: &str, signature: &str| {
        json!({
            "email": email,
            "first_name": "Applicant",
            "last_name": "Test",
            "date_of_birth": "1990-01-01",
            "city": "Praha",
            "phone_number": "+420123456789",
            "company_name": "ACME",
            "occupation": "Engineer",
            "local": "cz",
            "signature": signature,
        })
    };

    for body in [
        form("not-an-email", "data:image/png;base64,iVBORw0KGgo="),
        // valid base64, but not an image
        form("a@example.com", "data:image/png;base64,aGVsbG8="),
    ] {
        let (status, response) = app
            .request(Method::POST, "/registration/join", None, Some(&body))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{response}");
    }
    assert_eq!(app.queued(), Vec::<String>::new(), "nothing processed");

    // Unknown codes redirect too, so they can't be probed
    let (status, _) = app
        .request(Method::GET, "/registration/unknown/confirm", None, None)
        .await;
    assert_eq!(status, StatusCode::FOUND);
}

// refused before the handler reads the database

#[tokio::test]
async fn unknown_route_and_method_answer_json_errors() {
    let app = TestApp::without_db();
    for (method, uri, expected) in [
        (Method::GET, "/no-such-route", StatusCode::NOT_FOUND),
        (Method::DELETE, "/status", StatusCode::METHOD_NOT_ALLOWED),
    ] {
        let (status, response) = app.admin_call(method.clone(), uri, None).await;
        assert_eq!(status, expected, "{method} {uri}");
        assert_eq!(response["error"]["code"], expected.as_u16(), "{response}");
    }
}

#[tokio::test]
async fn unreadable_json_bodies_are_refused() {
    let app = TestApp::without_db();
    let token = FakeProvider::token(ADMIN_SUB, ALL_ROLES);
    let json = Some("application/json");
    // a JSON string one byte over the 1 MiB test limit
    let too_big = format!("\"{}\"", "a".repeat(1_048_575));

    for (content_type, body, expected) in [
        (json, "{".to_owned(), StatusCode::BAD_REQUEST),
        (None, "{}".to_owned(), StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (
            Some("text/plain"),
            "{}".to_owned(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (json, too_big, StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let (status, response) = app
            .send_raw(Method::POST, "/members", Some(&token), content_type, body)
            .await;
        assert_eq!(status, expected, "{content_type:?}: {response}");
        assert_eq!(response["error"]["code"], expected.as_u16(), "{response}");
    }
}

#[tokio::test]
async fn oidc_group_lookup_bad_requests() {
    let app = TestApp::without_db();
    for (group, description) in [
        ("not-a-uuid".to_owned(), "Group ID needs to be valid UUID"),
        (
            FakeProvider::MISSING_GROUP.to_string(),
            "OIDC group not found",
        ),
    ] {
        let (status, response) = app
            .admin_call(Method::GET, &format!("/oidc/{group}"), None)
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{group}: {response}");
        assert_eq!(response["error"]["description"], description);
    }
}

#[tokio::test]
async fn routes_are_404_when_keycloak_is_not_configured() {
    let app = TestApp::without_keycloak();
    let status = app.admin_status(Method::GET, "/members", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
