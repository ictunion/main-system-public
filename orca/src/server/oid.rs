use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::TokenData;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;
use uuid::Uuid;

#[cfg(test)]
pub(crate) mod fake;
mod keycloak;

use super::jwk;
use crate::api::ApiError;
use crate::config;
use keycloak::KeycloakProvider;

/// A Keycloak user, as the Admin API returns it. Read with Keycloak's camelCase
/// names, sent on to Melon Head in `snake_case`.
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub(crate) id: Uuid,
    pub email: String,
    #[serde(alias = "firstName")]
    pub(crate) first_name: Option<String>,
    #[serde(alias = "lastName")]
    pub(crate) last_name: Option<String>,
}

#[cfg(test)]
mod user_tests {
    use super::User;

    #[test]
    fn reads_keycloak_names_and_sends_snake_case() {
        let user: User = serde_json::from_str(
            r#"{"id": "00000000-0000-4000-8000-000000000002", "email": "jana@example.com",
                "firstName": "Jana", "lastName": "Nováková", "enabled": true}"#,
        )
        .unwrap();
        let sent = serde_json::to_value(&user).unwrap();
        assert_eq!(sent["first_name"], "Jana");
        assert_eq!(sent["last_name"], "Nováková");
    }
}

/// A Keycloak account to create for a member; Keycloak assigns the id.
#[derive(Debug)]
pub struct NewUser {
    pub email: String,
    pub(crate) first_name: Option<String>,
    pub(crate) last_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JwtToken {
    string: String,
}

impl JwtToken {
    pub fn new(string: String) -> Self {
        JwtToken { string }
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    ListApplications,
    ViewApplication,
    ResolveApplications,
    ListMembers,
    ManageMembers,
    ListWorkplaces,
    ManageWorkplaces,
    /// Scoped capability: list members of the workplaces the caller is an
    /// executive of, and nothing outside them. The role only grants the
    /// *capability*; the scope is resolved per request from the caller's
    /// Keycloak executive group membership. See `api::workplaces::mine`.
    ListOwnWorkplaceMembers,
    /// Scoped capability: open the full detail of a workplace the caller is an
    /// executive of. Grants only the *capability*; the scope is resolved per
    /// request from the caller's Keycloak executive group membership. Staff use
    /// `ListWorkplaces` and are not restricted to their own. See
    /// `api::workplaces::detail`.
    ListOwnWorkplace,
    /// Scoped capability: list applications assigned to the workplaces the
    /// caller is an executive of, and open their redacted detail. Scope is
    /// resolved per request like `ListOwnWorkplaceMembers`. Staff use
    /// `ListApplications` / `ViewApplication`. See `api::applications`.
    ListOwnWorkplaceApplications,
    /// Scoped capability: approve (recommend for acceptance) or reject
    /// applications assigned to the caller's executive workplaces. Never
    /// accept, unreject or anything else -- those need `ResolveApplications`.
    ResolveOwnWorkplaceApplications,
    /// Edit and delete notes written by other users, and clear whole note
    /// threads. Only together with the target's own write role. See `api::notes`.
    ManageNotes,
    SuperPowers,
}

impl Role {
    fn to_json_val(self) -> &'static str {
        match self {
            Self::ListApplications => "list-applications",
            Self::ViewApplication => "view-application",
            Self::ResolveApplications => "resolve-applications",
            Self::ListMembers => "list-members",
            Self::ManageMembers => "manage-members",
            Self::ListWorkplaces => "list-workplaces",
            Self::ManageWorkplaces => "manage-workplaces",
            Self::ListOwnWorkplaceMembers => "list-own-workplace-members",
            Self::ListOwnWorkplace => "list-own-workplace",
            Self::ListOwnWorkplaceApplications => "list-own-workplace-applications",
            Self::ResolveOwnWorkplaceApplications => "resolve-own-workplace-applications",
            Self::ManageNotes => "manage-notes",
            Self::SuperPowers => "super-powers",
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json_val())
    }
}

#[derive(Clone)]
enum ProviderState {
    Keycloak(Box<KeycloakProvider>),
    #[cfg(test)]
    Fake(Box<fake::FakeProvider>),
    Disconnected,
}

trait OidProvider {
    fn decode_jwt(
        &self,
        token: &JwtToken,
    ) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error>;

    fn require_role(&self, token: &JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error>;

    fn require_any_role(
        &self,
        token: &JwtToken,
        roles: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error>;

    // Admin API calls: made with Orca's own service account token, never the
    // caller's. Callers must check the caller's role before calling these.
    async fn create_user(&self, user: &NewUser) -> Result<Uuid, Error>;
    async fn remove_user(&self, id: Uuid) -> Result<(), Error>;
    async fn get_matching_users(&self, email: String) -> Result<Vec<User>, Error>;
    async fn connect_keycloak_user_and_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error>;
    async fn remove_keycloak_user_from_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error>;
    async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<Uuid>, Error>;
    async fn get_own_groups(&self, token: &JwtToken) -> Result<Vec<Uuid>, Error>;
}

#[derive(Clone)]
pub struct Provider(ProviderState);

/// Written by hand so key material never ends up in logs
impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Provider")
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl Provider {
    pub async fn init(config: &config::Config) -> Result<Provider, Error> {
        if let (Some(host), Some(realm), Some(client_id)) = (
            &config.keycloak_host,
            &config.keycloak_realm,
            &config.keycloak_client_id,
        ) {
            let (Some(id), Some(secret)) = (
                &config.keycloak_service_client_id,
                &config.keycloak_service_client_secret,
            ) else {
                return Err(Error::MissingServiceAccount);
            };
            let service_account = (id.clone(), secret.clone());
            let k =
                KeycloakProvider::fetch(host, realm, client_id.clone(), service_account).await?;
            return Ok(Provider(ProviderState::Keycloak(Box::new(k))));
        }

        warn!("Keycloak authorization not configured. Authorization disabled");
        Ok(Provider(ProviderState::Disconnected))
    }

    pub fn decode_jwt(&self, token: &JwtToken) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => {
                let res = k.decode_jwt(token)?;
                Ok(res)
            }
            #[cfg(test)]
            ProviderState::Fake(f) => {
                let res = f.decode_jwt(token)?;
                Ok(res)
            }
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_role(
        &self,
        token: &JwtToken,
        role: Role,
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.require_role(token, role),
            #[cfg(test)]
            ProviderState::Fake(f) => f.require_role(token, role),
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_any_role(
        &self,
        token: &JwtToken,
        role: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.require_any_role(token, role),
            #[cfg(test)]
            ProviderState::Fake(f) => f.require_any_role(token, role),
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    #[cfg(test)]
    pub(crate) fn fake(fake: fake::FakeProvider) -> Self {
        Provider(ProviderState::Fake(Box::new(fake)))
    }

    /// As `init` returns when Keycloak is not configured
    #[cfg(test)]
    pub(crate) fn disconnected() -> Self {
        Provider(ProviderState::Disconnected)
    }

    pub fn is_connected(&self) -> bool {
        match self.0 {
            ProviderState::Keycloak(_) => true,
            #[cfg(test)]
            ProviderState::Fake(_) => true,
            ProviderState::Disconnected => false,
        }
    }

    pub async fn create_user(&self, user: &NewUser) -> Result<Uuid, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.create_user(user).await,
            #[cfg(test)]
            ProviderState::Fake(f) => f.create_user(user).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn remove_user(&self, id: Uuid) -> Result<(), Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.remove_user(id).await,
            #[cfg(test)]
            ProviderState::Fake(f) => f.remove_user(id).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn connect_keycloak_user_and_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => {
                k.connect_keycloak_user_and_group(keycloak_user_id, keycloak_group_id)
                    .await
            }
            #[cfg(test)]
            ProviderState::Fake(f) => {
                f.connect_keycloak_user_and_group(keycloak_user_id, keycloak_group_id)
                    .await
            }
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn remove_keycloak_user_from_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => {
                k.remove_keycloak_user_from_group(keycloak_user_id, keycloak_group_id)
                    .await
            }
            #[cfg(test)]
            ProviderState::Fake(f) => {
                f.remove_keycloak_user_from_group(keycloak_user_id, keycloak_group_id)
                    .await
            }
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn get_matching_users(&self, email: String) -> Result<Vec<User>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.get_matching_users(email).await,
            #[cfg(test)]
            ProviderState::Fake(f) => f.get_matching_users(email).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<Uuid>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.get_group_members(group_id).await,
            #[cfg(test)]
            ProviderState::Fake(f) => f.get_group_members(group_id).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    /// Groups the *caller* belongs to, read live from the identity provider
    /// using the caller's own token. Deliberately not taken from a token claim:
    /// group membership drives authorization scope, so it must not be stale for
    /// the lifetime of an access token, and org structure should not be baked
    /// into a credential that gets cached and logged.
    pub async fn get_own_groups(&self, token: &JwtToken) -> Result<Vec<Uuid>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.get_own_groups(token).await,
            #[cfg(test)]
            ProviderState::Fake(f) => f.get_own_groups(token).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bad key: {0}")]
    BadKey(#[from] jwk::Error),
    #[error("Bad token: {0}")]
    BadToken(#[from] jsonwebtoken::errors::Error),
    #[error("Token is missing role: {0:?}")]
    MissingRole(Role),
    #[error("One of roles missing: {0:?}")]
    MissingOneOfRoles(Vec<Role>),
    #[error("Keycloak is disconnected")]
    Disabled,
    #[error("keycloak_service_client_id and keycloak_service_client_secret must be set")]
    MissingServiceAccount,
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parsing error: {0}")]
    Parsing(String),
    #[error("Proxy error: {0}")]
    Proxy(reqwest::StatusCode),
}

#[derive(Debug, Deserialize, Serialize)]
struct Roles {
    roles: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    resource_access: HashMap<String, Roles>,
    pub email: String,
    pub name: Option<String>,
}

/// Bearer token from the `Authorization` header; a missing header is a 401
impl<S: Send + Sync> FromRequestParts<S> for JwtToken {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::Status(StatusCode::UNAUTHORIZED))?;

        let token = header.trim_start_matches("Bearer").trim();
        Ok(JwtToken {
            string: token.to_string(),
        })
    }
}
