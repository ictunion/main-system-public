use super::{ApiError, AppState, SuccessResponse};
use crate::api::Response;
use crate::api::extract::{ApiJson, ApiPath};
use crate::api::files::FileInfo;
use crate::api::members::MemberSummary;
use crate::api::members::query::get_status_data;
use crate::config::Config;
use crate::data::{Id, Member, MemberNumber, Workplace};
use crate::db::DbPool;
use crate::processing::workplace_documents::{EmployerInput, WorkplaceDocument};
use crate::processing::{Command, QueueSender};
use crate::server::oid::{JwtToken, Provider, Role};
use crate::validation::Validated;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, patch, put};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

pub mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkplaceSummary {
    id: Id<Workplace>,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    keycloak_group_id: Uuid,
    keycloak_executive_group_id: Option<Uuid>,
    announced_at: Option<DateTime<Utc>>,
    established_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    newsletter_id: Option<i32>,
    company_id: Option<String>,
    company_address: Option<String>,
    workplace_short_name: Option<String>,
    member_count: i64,
}

async fn list_all(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ListWorkplaces)?;

    let summaries = query::list_summaries(&db_pool).await?;

    Ok(Json(summaries))
}

/// Reduced projection of a member, for workplace executive committees.
///
/// Deliberately narrower than `MemberSummary`, which carries the freeform staff
/// `note`, `city` and company names, and narrower still than `MemberDetail`,
/// which adds date of birth, home address and postal code. `member_number` is
/// included: it is the human-readable identifier members quote for payments, so
/// a rep needs to see it.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkplaceMemberSummary {
    id: Id<Member>,
    member_number: MemberNumber,
    workplace_id: Id<Workplace>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone_number: Option<String>,
    created_at: DateTime<Utc>,
    is_representative: bool,
}

/// Resolve which workplaces the caller may act on as an executive, as bare IDs.
///
/// The Keycloak role grants only the *capability*; this resolves the *scope*,
/// and it is the single place that decides it. Returns an empty vector when the
/// caller is in no executive group, so every downstream query is bounded rather
/// than unbounded. Exposed to `api::members` so a single-member read can be
/// checked against the same scope.
pub(crate) async fn executive_workplace_id_list(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
) -> Response<Vec<Uuid>> {
    let group_ids = oid_provider.get_own_groups(token).await?;

    if group_ids.is_empty() {
        return Ok(Vec::new());
    }

    Ok(query::list_executive_workplace_ids(db_pool, &group_ids).await?)
}

/// The workplace whose executive committee the caller sits on, in full -- the
/// same projection as `GET /workplaces/<id>`. Backs both the rep-facing "My
/// Workplace" roster (which reads its title off this) and the "My Workplace
/// Settings" page (which reuses the staff workplace detail view).
///
/// No workplace ID in the path: the scope is resolved from the caller's
/// Keycloak executive group membership. By union process a rep sits on exactly
/// one committee; if the resolver ever returns several, the first by name is
/// used. A caller not linked to any workplace gets a 404.
async fn list_mine(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_any_role(&token, &[Role::ListOwnWorkplace])?;

    // each executive member should have only one workplace, we don't want someone to be rep at multiple workplaces
    let own = executive_workplace_id_list(&db_pool, &oid_provider, &token).await?;

    let Some(workplace_id) = own.first().copied() else {
        return Err(ApiError::Status(StatusCode::NOT_FOUND));
    };

    let detail = query::detail(&db_pool, Id::from(workplace_id)).await?;

    Ok(Json(detail))
}

/// Members of the workplaces the caller is an executive of -- and only those.
///
/// There is no workplace ID in the path on purpose: the caller cannot name a
/// workplace, so there is no parameter to tamper with and no way to reach a
/// roster outside their own committee.
async fn list_mine_members(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<WorkplaceMemberSummary>>> {
    oid_provider.require_role(&token, Role::ListOwnWorkplaceMembers)?;

    let workplace_ids = executive_workplace_id_list(&db_pool, &oid_provider, &token).await?;

    if workplace_ids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let members = query::list_members_of_workplaces(&db_pool, &workplace_ids).await?;

    Ok(Json(members))
}

async fn list_active(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_active_summaries(&db_pool).await?;

    Ok(Json(summaries))
}

async fn list_in_preparation(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_in_preparation_summaries(&db_pool).await?;

    Ok(Json(summaries))
}

async fn list_inactive(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_inactive_summaries(&db_pool).await?;

    Ok(Json(summaries))
}

async fn detail(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<WorkplaceSummary>> {
    // Staff (`list-workplaces`) may open any workplace. A rep with
    // `list-own-workplace` may open only workplaces whose executive committee
    // they sit on -- the same full projection, internal IDs and contact address
    // included, since those help a rep debugging their own workplace's
    // integrations. A workplace outside their scope is refused with the same
    // 403 as a caller holding no role at all.
    if let Err(staff_denial) = oid_provider.require_role(&token, Role::ListWorkplaces) {
        oid_provider.require_role(&token, Role::ListOwnWorkplace)?;

        let own = executive_workplace_id_list(&db_pool, &oid_provider, &token).await?;
        if !own.contains(&Uuid::from(workplace_id)) {
            return Err(staff_denial.into());
        }
    }

    let detail = query::detail(&db_pool, workplace_id).await?;

    Ok(Json(detail))
}

async fn list_representatives(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<Vec<WorkplaceMemberSummary>>> {
    // Same scope as `detail`: staff may look up any workplace's reps, an
    // executive only their own.
    if let Err(staff_denial) = oid_provider.require_role(&token, Role::ListWorkplaces) {
        oid_provider.require_role(&token, Role::ListOwnWorkplace)?;

        let own = executive_workplace_id_list(&db_pool, &oid_provider, &token).await?;
        if !own.contains(&Uuid::from(workplace_id)) {
            return Err(staff_denial.into());
        }
    }

    let reps = query::list_representatives(&db_pool, workplace_id).await?;

    Ok(Json(reps))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewWorkplace {
    #[validate(required)]
    name: Option<String>,
    #[validate(required)]
    email: Option<String>,
    #[validate(required)]
    keycloak_group_id: Option<Uuid>,
    #[validate(required)]
    keycloak_executive_group_id: Option<Uuid>,
    newsletter_id: Option<i32>,
    company_id: Option<String>,
    company_address: Option<String>,
    workplace_short_name: Option<String>,
}

async fn create_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    new_workplace: Validated<NewWorkplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let mut new_workplace = new_workplace.into_inner();
    new_workplace.company_id = normalize_text(new_workplace.company_id);
    new_workplace.company_address = normalize_text(new_workplace.company_address);
    new_workplace.workplace_short_name = normalize_text(new_workplace.workplace_short_name);

    // Create new workplace
    let workplace = query::create_workplace(&db_pool, &new_workplace).await?;

    Ok(Json(workplace))
}

/// Partial update: a missing or `null` field (or empty text) keeps the stored value.
/// Once set, a field can be changed but never cleared again.
#[derive(Debug, Deserialize)]
pub struct UpdateWorkplace {
    newsletter_id: Option<i32>,
    company_id: Option<String>,
    company_address: Option<String>,
    workplace_short_name: Option<String>,
}

/// Trims text and treats empty as not set
fn normalize_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

async fn update(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
    ApiJson(mut update): ApiJson<UpdateWorkplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    update.company_id = normalize_text(update.company_id);
    update.company_address = normalize_text(update.company_address);
    update.workplace_short_name = normalize_text(update.workplace_short_name);

    let workplace = query::update_workplace(&db_pool, workplace_id, &update).await?;

    Ok(Json(workplace))
}

async fn establish(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::establish_workplace(&db_pool, workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be established in its current state")
        })?;

    Ok(Json(workplace))
}

async fn announce(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::announce_workplace(&db_pool, workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be announced in its current state")
        })?;

    Ok(Json(workplace))
}

async fn cancel(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::cancel_workplace(&db_pool, workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be cancelled in its current state")
        })?;

    Ok(Json(workplace))
}

async fn assign_member_to_workplace_impl(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
    is_representative: bool,
) -> Response<SuccessResponse> {
    oid_provider.require_role(token, Role::ManageWorkplaces)?;

    let was_representative = query::get_is_representative(db_pool, workplace_id, member_id).await?;

    let rows_affected = query::create_connection_between_member_and_workplace(
        db_pool,
        workplace_id,
        member_id,
        is_representative,
    )
    .await?;

    let workplace_details = query::detail(db_pool, workplace_id).await?;

    let user_data = get_status_data(db_pool, member_id).await?;

    let keycloak_id = user_data.sub().ok_or_else(|| {
        ApiError::keycloak_push(&format!("keycloak ID for user {member_id} not assigned"))
    })?;

    oid_provider
        .connect_keycloak_user_and_group(token, keycloak_id, workplace_details.keycloak_group_id)
        .await?;

    // Only touch the executive group on an actual transition -- most PUTs here
    // are either a first-time assignment (never representative) or a
    // no-op re-assignment, and firing a Keycloak request either way would be
    // one API round-trip per member for no effect.
    if let Some(keycloak_executive_group_id) = workplace_details.keycloak_executive_group_id {
        if is_representative && !was_representative {
            oid_provider
                .connect_keycloak_user_and_group(token, keycloak_id, keycloak_executive_group_id)
                .await?;
        } else if !is_representative && was_representative {
            oid_provider
                .remove_keycloak_user_from_group(token, keycloak_id, keycloak_executive_group_id)
                .await?;
        }
    }

    if rows_affected == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Created)
    }
}

async fn assign_member_to_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((workplace_id, member_id)): ApiPath<(Id<Workplace>, Id<Member>)>,
) -> Response<SuccessResponse> {
    assign_member_to_workplace_impl(
        &db_pool,
        &oid_provider,
        &token,
        workplace_id,
        member_id,
        false,
    )
    .await
}

async fn assign_member_to_workplace_rep(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((workplace_id, member_id)): ApiPath<(Id<Workplace>, Id<Member>)>,
) -> Response<SuccessResponse> {
    assign_member_to_workplace_impl(
        &db_pool,
        &oid_provider,
        &token,
        workplace_id,
        member_id,
        true,
    )
    .await
}

async fn remove_member_from_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((workplace_id, member_id)): ApiPath<(Id<Workplace>, Id<Member>)>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let was_representative =
        query::get_is_representative(&db_pool, workplace_id, member_id).await?;

    let rows_affected =
        query::remove_connection_between_member_and_workplace(&db_pool, workplace_id, member_id)
            .await?;

    let workplace_details = query::detail(&db_pool, workplace_id).await?;

    let user_data = get_status_data(&db_pool, member_id).await?;

    let keycloak_id = user_data.sub().ok_or_else(|| {
        ApiError::keycloak_push(&format!("keycloak ID for user {member_id} not assigned"))
    })?;

    oid_provider
        .remove_keycloak_user_from_group(&token, keycloak_id, workplace_details.keycloak_group_id)
        .await?;

    if let (true, Some(keycloak_executive_group_id)) = (
        was_representative,
        workplace_details.keycloak_executive_group_id,
    ) {
        oid_provider
            .remove_keycloak_user_from_group(&token, keycloak_id, keycloak_executive_group_id)
            .await?;
    }

    if rows_affected == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Accepted)
    }
}

async fn get_all_workplace_members(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListWorkplaces)?;
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::get_all_workplace_members(&db_pool, workplace_id).await?;

    Ok(Json(summaries))
}

async fn list_files(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
) -> Response<Json<Vec<FileInfo>>> {
    oid_provider.require_role(&token, Role::ListWorkplaces)?;

    let files = query::list_workplace_files(&db_pool, workplace_id).await?;

    Ok(Json(files))
}

/// Request to generate a document (PDF) for a workplace.
/// `document_type` selects the variant, the rest of the body are its inputs:
///
/// ```json
/// { "document_type": "entrustment", "workplace_short_name": "ACME CZ", ... }
/// { "document_type": "member_confirmation", "workplace_short_name": "ACME CZ" }
/// { "document_type": "workplace_announcement", "workplace_short_name": "ACME CZ", ... }
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "document_type", rename_all = "snake_case")]
pub enum NewWorkplaceDocument {
    Entrustment(NewEntrustment),
    MemberConfirmation(NewMemberConfirmation),
    WorkplaceAnnouncement(NewWorkplaceAnnouncement),
}

// validator can't derive for enums
impl Validate for NewWorkplaceDocument {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Self::Entrustment(data) => data.validate(),
            Self::MemberConfirmation(data) => data.validate(),
            Self::WorkplaceAnnouncement(data) => data.validate(),
        }
    }
}

/// Builds employer details from fields which already passed validation.
/// `None` only when called on data which didn't.
fn employer_input(
    workplace_short_name: Option<String>,
    company_address: Option<String>,
    company_id: Option<String>,
) -> Option<EmployerInput> {
    Some(EmployerInput {
        workplace_short_name: workplace_short_name?.trim().to_string(),
        company_address: company_address?.trim().to_string(),
        company_id: company_id?.trim().to_string(),
    })
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewEntrustment {
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    workplace_short_name: Option<String>,
    #[validate(required)]
    meeting_date: Option<NaiveDate>,
    /// Members no longer on the committee, listed in an extra section when not empty
    #[serde(default)]
    removed_member_ids: Vec<Uuid>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    company_address: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    company_id: Option<String>,
}

impl NewEntrustment {
    /// Keeps the first occurrence of each id, in the order they were added
    fn dedup_removed_members(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.removed_member_ids.retain(|id| seen.insert(*id));
    }

    /// `None` only when called on data which didn't pass validation
    fn into_document(self) -> Option<WorkplaceDocument> {
        Some(WorkplaceDocument::Entrustment {
            meeting_date: self.meeting_date?,
            removed_members: self.removed_member_ids,
            employer: employer_input(
                self.workplace_short_name,
                self.company_address,
                self.company_id,
            )?,
        })
    }
}

/// Short name only goes into the file name, the rest comes from the database
#[derive(Debug, Deserialize, Validate)]
pub struct NewMemberConfirmation {
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    workplace_short_name: Option<String>,
}

impl NewMemberConfirmation {
    /// `None` only when called on data which didn't pass validation
    fn into_document(self) -> Option<WorkplaceDocument> {
        Some(WorkplaceDocument::MemberConfirmation {
            workplace_short_name: self.workplace_short_name?.trim().to_string(),
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewWorkplaceAnnouncement {
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    workplace_short_name: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    company_address: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    company_id: Option<String>,
}

impl NewWorkplaceAnnouncement {
    /// `None` only when called on data which didn't pass validation
    fn into_document(self) -> Option<WorkplaceDocument> {
        Some(WorkplaceDocument::WorkplaceAnnouncement(employer_input(
            self.workplace_short_name,
            self.company_address,
            self.company_id,
        )?))
    }
}

/// Generating happens in the processing queue, so this only checks what it can
/// up front and answers 202. The PDF shows up in `GET /<workplace_id>/files` when done.
async fn create_document(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    State(config): State<Arc<Config>>,
    State(queue): State<QueueSender>,
    token: JwtToken,
    ApiPath(workplace_id): ApiPath<Id<Workplace>>,
    data: Validated<NewWorkplaceDocument>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    // 404 for unknown workplace
    query::detail(&db_pool, workplace_id).await?;

    // Every workplace document lists representatives and is signed by the chairperson
    if config.union_chairperson_member_id.is_none() {
        error!("Can't generate workplace document: union_chairperson_member_id not configured");
        return Err(ApiError::Status(StatusCode::INTERNAL_SERVER_ERROR));
    }

    let representatives = query::list_representatives(&db_pool, workplace_id).await?;
    if representatives.is_empty() {
        return Err(ApiError::data_conflict("Workplace has no representatives"));
    }

    let document = match data.into_inner() {
        NewWorkplaceDocument::Entrustment(mut entrustment) => {
            entrustment.dedup_removed_members();
            let removed = &entrustment.removed_member_ids;
            if !removed.is_empty() {
                let known = query::count_members(&db_pool, removed).await?;
                if i64::try_from(removed.len()).ok() != Some(known) {
                    return Err(ApiError::data_conflict("Some removed members don't exist"));
                }
            }
            entrustment.into_document()
        }
        NewWorkplaceDocument::MemberConfirmation(confirmation) => confirmation.into_document(),
        NewWorkplaceDocument::WorkplaceAnnouncement(announcement) => {
            if config.union_chairperson_email.is_none() {
                error!("Can't generate announcement: union_chairperson_email not configured");
                return Err(ApiError::Status(StatusCode::INTERNAL_SERVER_ERROR));
            }
            announcement.into_document()
        }
    }
    .ok_or(ApiError::Status(StatusCode::UNPROCESSABLE_ENTITY))?;

    queue
        .send(Command::NewWorkplaceDocument(workplace_id, document))
        .await?;

    Ok(SuccessResponse::Accepted)
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_all).post(create_workplace))
        .route("/mine", get(list_mine))
        .route("/mine/members", get(list_mine_members))
        .route("/active", get(list_active))
        .route("/in_preparation", get(list_in_preparation))
        .route("/inactive", get(list_inactive))
        .route("/{workplace_id}", get(detail).patch(update))
        .route("/{workplace_id}/representatives", get(list_representatives))
        .route("/{workplace_id}/establish", patch(establish))
        .route("/{workplace_id}/announce", patch(announce))
        .route("/{workplace_id}/cancel", patch(cancel))
        .route(
            "/{workplace_id}/members/{member_id}",
            put(assign_member_to_workplace).delete(remove_member_from_workplace),
        )
        .route(
            "/{workplace_id}/members/{member_id}/is_representative",
            put(assign_member_to_workplace_rep),
        )
        .route("/{workplace_id}/members", get(get_all_workplace_members))
        .route(
            "/{workplace_id}/files",
            get(list_files).post(create_document),
        )
}

#[cfg(test)]
mod update_workplace_tests {
    use super::*;

    fn parse(json: &str) -> UpdateWorkplace {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn missing_company_fields_are_kept() {
        let update = parse(r#"{"newsletter_id": 42}"#);
        assert_eq!(update.newsletter_id, Some(42));
        assert_eq!(update.company_id, None);
        assert_eq!(update.company_address, None);
        assert_eq!(update.workplace_short_name, None);
    }

    #[test]
    fn null_company_field_is_kept_and_value_is_set() {
        let update = parse(r#"{"company_id": null, "workplace_short_name": "ACME CZ"}"#);
        assert_eq!(update.company_id, None);
        assert_eq!(update.workplace_short_name, Some("ACME CZ".to_string()));
    }

    #[test]
    fn normalize_text_trims_and_empties() {
        assert_eq!(
            normalize_text(Some("  ACME  ".to_string())),
            Some("ACME".to_string())
        );
        assert_eq!(normalize_text(Some("   ".to_string())), None);
        assert_eq!(normalize_text(None), None);
    }
}
