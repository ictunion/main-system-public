//! Free-text notes on applications, members, member dues threads and workplaces.
//!
//! Notes are temporary and hard-deleted on purpose. Any caller with the
//! target's write role can add a note and edit or delete their own. Modifying
//! other people's notes, or clearing a whole thread, also needs `manage-notes`.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use jsonwebtoken::TokenData;
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::{ApiError, AppState, SuccessResponse, members, workplaces};
use crate::api::Response;
use crate::api::extract::ApiPath;
use crate::data::{Id, Member, Note};
use crate::db::{self, DbPool};
use crate::server::oid::{JwtClaims, JwtToken, Provider, Role};
use crate::validation::Validated;

mod query;

pub const MAX_BODY_LENGTH: u64 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "note_target", rename_all = "snake_case")]
pub enum NoteTarget {
    Application,
    Member,
    MemberDues,
    Workplace,
}

impl NoteTarget {
    /// FK columns `(registration_request_id, member_id, workplace_id)` for a
    /// target; exactly one is set, matching the `notes_one_target` constraint.
    fn columns(self, target_id: Uuid) -> (Option<Uuid>, Option<Uuid>, Option<Uuid>) {
        match self {
            Self::Application => (Some(target_id), None, None),
            Self::Member | Self::MemberDues => (None, Some(target_id), None),
            Self::Workplace => (None, None, Some(target_id)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    Read,
    Write,
}

/// Staff roles that grant access to every target of a type. Reps get scoped
/// access to `member_dues` on top of this, see `authorize`.
fn staff_roles(target: NoteTarget, access: Access) -> &'static [Role] {
    match (target, access) {
        (NoteTarget::Application, Access::Read) => &[Role::ListApplications, Role::ViewApplication],
        (NoteTarget::Application, Access::Write) => &[Role::ResolveApplications],
        (NoteTarget::Member, Access::Read) | (NoteTarget::MemberDues, _) => &[Role::ListMembers],
        (NoteTarget::Member, Access::Write) => &[Role::ManageMembers],
        (NoteTarget::Workplace, Access::Read) => &[Role::ListWorkplaces],
        (NoteTarget::Workplace, Access::Write) => &[Role::ManageWorkplaces],
    }
}

/// Author of the note, or a holder of `manage-notes`. Imported notes have no
/// author, so only managers can modify them.
fn may_modify(author_sub: Option<Uuid>, caller_sub: Uuid, is_manager: bool) -> bool {
    is_manager || author_sub == Some(caller_sub)
}

/// Reps reach only `member_dues`; never `member` notes, which are staff-only.
/// Which members are in the rep's scope is checked in `authorize`.
fn rep_may_reach(target: NoteTarget) -> bool {
    target == NoteTarget::MemberDues
}

/// Check the caller may read or write notes of one target and return their
/// claims. Staff roles reach every target. A rep with
/// `list-own-workplace-members` reaches only the `member_dues` thread of a
/// current member of their own workplace(s). Anything else is refused with
/// the same 403 as a caller holding no role.
async fn authorize(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
    target: NoteTarget,
    target_id: Uuid,
    access: Access,
) -> Result<TokenData<JwtClaims>, ApiError> {
    let staff_denial = match oid_provider.require_any_role(token, staff_roles(target, access)) {
        Ok(claims) => return Ok(claims),
        Err(denial) => denial,
    };

    if rep_may_reach(target)
        && let Ok(claims) = oid_provider.require_role(token, Role::ListOwnWorkplaceMembers)
    {
        let workplace_ids =
            workplaces::executive_workplace_id_list(db_pool, oid_provider, token).await?;

        if !workplace_ids.is_empty()
            && members::query::is_member_of_workplaces(
                db_pool,
                Id::<Member>::from(target_id),
                &workplace_ids,
            )
            .await?
        {
            return Ok(claims);
        }
    }

    Err(staff_denial.into())
}

fn author_name(claims: &JwtClaims) -> String {
    claims
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(claims.email.as_str())
        .to_owned()
}

/// A foreign key violation on insert means the target does not exist.
fn missing_target_as_not_found(err: sqlx::Error) -> ApiError {
    if db::is_foreign_key_violation(&err) {
        return ApiError::Status(StatusCode::NOT_FOUND);
    }
    err.into()
}

#[cfg_attr(test, derive(serde::Deserialize))]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NoteDetail {
    id: Id<Note>,
    target_type: NoteTarget,
    target_id: Uuid,
    body: String,
    author_sub: Option<Uuid>,
    author_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// One row per target that has at least one note. Only what the table cell
/// shows; the full thread is loaded when the drawer opens.
#[cfg_attr(test, derive(serde::Deserialize))]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NoteSummary {
    target_id: Uuid,
    count: i64,
    latest_body: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct NoteBody {
    #[serde(deserialize_with = "super::trimmed")]
    #[validate(length(min = 1, max = MAX_BODY_LENGTH))]
    body: String,
}

async fn list_summary(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(target): ApiPath<NoteTarget>,
) -> Response<Json<Vec<NoteSummary>>> {
    let Err(staff_denial) =
        oid_provider.require_any_role(&token, staff_roles(target, Access::Read))
    else {
        return Ok(Json(query::list_summary(&db_pool, target, None).await?));
    };

    // Reps: only dues threads of current members of their own workplace(s).
    if target != NoteTarget::MemberDues
        || oid_provider
            .require_role(&token, Role::ListOwnWorkplaceMembers)
            .is_err()
    {
        return Err(staff_denial.into());
    }

    let workplace_ids =
        workplaces::executive_workplace_id_list(&db_pool, &oid_provider, &token).await?;

    if workplace_ids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let summaries = query::list_summary(&db_pool, target, Some(&workplace_ids)).await?;

    Ok(Json(summaries))
}

async fn list(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((target, target_id)): ApiPath<(NoteTarget, Uuid)>,
) -> Response<Json<Vec<NoteDetail>>> {
    authorize(
        &db_pool,
        &oid_provider,
        &token,
        target,
        target_id,
        Access::Read,
    )
    .await?;

    let notes = query::list(&db_pool, target, target_id).await?;

    Ok(Json(notes))
}

async fn create(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((target, target_id)): ApiPath<(NoteTarget, Uuid)>,
    data: Validated<NoteBody>,
) -> Response<Json<NoteDetail>> {
    let claims = authorize(
        &db_pool,
        &oid_provider,
        &token,
        target,
        target_id,
        Access::Write,
    )
    .await?
    .claims;

    let note = query::create(
        &db_pool,
        target,
        target_id,
        &data.into_inner().body,
        claims.sub,
        &author_name(&claims),
    )
    .await
    .map_err(missing_target_as_not_found)?;

    Ok(Json(note))
}

/// Load a note that belongs to the target in the path and check the caller may
/// modify it. A note of another target is a 404, so a path the caller is
/// authorized for cannot be used to reach a note on a different record.
async fn authorize_modify(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
    target: NoteTarget,
    target_id: Uuid,
    note_id: Id<Note>,
) -> Result<Uuid, ApiError> {
    let claims = authorize(
        db_pool,
        oid_provider,
        token,
        target,
        target_id,
        Access::Write,
    )
    .await?
    .claims;

    let author_sub = query::author_of(db_pool, target, target_id, note_id).await?;
    let is_manager = oid_provider.require_role(token, Role::ManageNotes).is_ok();

    if !may_modify(author_sub, claims.sub, is_manager) {
        return Err(ApiError::Status(StatusCode::FORBIDDEN));
    }

    Ok(claims.sub)
}

async fn update(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((target, target_id, note_id)): ApiPath<(NoteTarget, Uuid, Id<Note>)>,
    data: Validated<NoteBody>,
) -> Response<Json<NoteDetail>> {
    authorize_modify(&db_pool, &oid_provider, &token, target, target_id, note_id).await?;

    let note = query::update(&db_pool, note_id, &data.into_inner().body).await?;

    Ok(Json(note))
}

async fn delete(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((target, target_id, note_id)): ApiPath<(NoteTarget, Uuid, Id<Note>)>,
) -> Response<SuccessResponse> {
    let caller_sub =
        authorize_modify(&db_pool, &oid_provider, &token, target, target_id, note_id).await?;

    query::delete(&db_pool, note_id).await?;

    info!("Note {note_id} hard deleted by {caller_sub}");

    Ok(SuccessResponse::Accepted)
}

/// Hard delete every note of one target. Deletes other people's notes too, so
/// it needs `manage-notes` on top of the target's write role.
async fn clear(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((target, target_id)): ApiPath<(NoteTarget, Uuid)>,
) -> Response<SuccessResponse> {
    let claims = authorize(
        &db_pool,
        &oid_provider,
        &token,
        target,
        target_id,
        Access::Write,
    )
    .await?
    .claims;
    oid_provider.require_role(&token, Role::ManageNotes)?;

    let deleted = query::delete_all(&db_pool, target, target_id).await?;

    info!(
        "{deleted} notes of {target:?} {target_id} hard deleted by {}",
        claims.sub
    );

    Ok(SuccessResponse::Accepted)
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/{target_type}", get(list_summary))
        .route(
            "/{target_type}/{target_id}",
            get(list).post(create).delete(clear),
        )
        .route(
            "/{target_type}/{target_id}/{note_id}",
            patch(update).delete(delete),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_type_uses_snake_case_in_paths_and_json() {
        let target: NoteTarget = serde_json::from_str(r#""member_dues""#).unwrap();
        assert_eq!(target, NoteTarget::MemberDues);
        assert_eq!(
            serde_json::to_string(&NoteTarget::Application).unwrap(),
            r#""application""#
        );
        assert!(serde_json::from_str::<NoteTarget>(r#""member-dues""#).is_err());
    }

    #[test]
    fn exactly_one_column_is_set() {
        let id = Uuid::new_v4();
        assert_eq!(NoteTarget::Application.columns(id), (Some(id), None, None));
        assert_eq!(NoteTarget::Member.columns(id), (None, Some(id), None));
        assert_eq!(NoteTarget::MemberDues.columns(id), (None, Some(id), None));
        assert_eq!(NoteTarget::Workplace.columns(id), (None, None, Some(id)));
    }

    #[test]
    fn staff_roles_follow_the_plan() {
        use Access::{Read, Write};
        use NoteTarget::{Application, Member, MemberDues, Workplace};

        assert_eq!(
            staff_roles(Application, Read),
            &[Role::ListApplications, Role::ViewApplication]
        );
        assert_eq!(
            staff_roles(Application, Write),
            &[Role::ResolveApplications]
        );
        assert_eq!(staff_roles(Member, Read), &[Role::ListMembers]);
        assert_eq!(staff_roles(Member, Write), &[Role::ManageMembers]);
        assert_eq!(staff_roles(MemberDues, Read), &[Role::ListMembers]);
        assert_eq!(staff_roles(MemberDues, Write), &[Role::ListMembers]);
        assert_eq!(staff_roles(Workplace, Read), &[Role::ListWorkplaces]);
        assert_eq!(staff_roles(Workplace, Write), &[Role::ManageWorkplaces]);
    }

    #[test]
    fn reps_reach_only_dues_notes() {
        assert!(rep_may_reach(NoteTarget::MemberDues));
        assert!(!rep_may_reach(NoteTarget::Member));
        assert!(!rep_may_reach(NoteTarget::Application));
        assert!(!rep_may_reach(NoteTarget::Workplace));
    }

    #[test]
    fn rep_role_grants_no_staff_access() {
        use NoteTarget::{Application, Member, MemberDues, Workplace};

        for target in [Application, Member, MemberDues, Workplace] {
            for access in [Access::Read, Access::Write] {
                assert!(!staff_roles(target, access).contains(&Role::ListOwnWorkplaceMembers));
            }
        }
    }

    #[test]
    fn only_author_or_manager_may_modify() {
        let author = Uuid::new_v4();
        let other = Uuid::new_v4();

        assert!(may_modify(Some(author), author, false));
        assert!(!may_modify(Some(author), other, false));
        assert!(may_modify(Some(author), other, true));
        // imported notes have no author
        assert!(!may_modify(None, other, false));
        assert!(may_modify(None, other, true));
    }

    fn validate(json: &str) -> Result<NoteBody, validator::ValidationErrors> {
        let body: NoteBody = serde_json::from_str(json).unwrap();
        body.validate().map(|()| body)
    }

    #[test]
    fn body_is_trimmed_and_must_not_be_blank() {
        assert_eq!(
            validate(r#"{"body": "  called  "}"#).unwrap().body,
            "called"
        );
        assert!(validate(r#"{"body": "   "}"#).is_err());
        assert!(validate(r#"{"body": ""}"#).is_err());
    }

    #[test]
    fn body_length_is_limited_in_characters() {
        let max = "ř".repeat(5000);
        assert!(validate(&format!(r#"{{"body": "{max}"}}"#)).is_ok());
        assert!(validate(&format!(r#"{{"body": "{max}x"}}"#)).is_err());
    }
}
