use jsonwebtoken::{self, TokenData};
use log::warn;
use reqwest;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::{FromForm, Responder};
use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;
use uuid::Uuid;

mod keycloak;

use super::jwk;
use crate::config;
use keycloak::KeycloakProvider;

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct User {
    id: Option<String>,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Debug, Clone, FromForm)]
pub struct JwtToken<'a> {
    #[field(name = "token")]
    string: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    ListApplications,
    ViewApplication,
    ResolveApplications,
    ListMembers,
    ViewMember,
    ManageMembers,
    ListWorkplaces,
    ManageWorkplaces,
    SuperPowers,
}

impl Role {
    fn to_json_val(self) -> &'static str {
        match self {
            Self::ListApplications => "list-applications",
            Self::ViewApplication => "view-application",
            Self::ResolveApplications => "resolve-applications",
            Self::ListMembers => "list-members",
            Self::ViewMember => "view-member",
            Self::ManageMembers => "manage-members",
            Self::ListWorkplaces => "list-workplaces",
            Self::ManageWorkplaces => "manage-workplaces",
            Self::SuperPowers => "super-powers",
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json_val())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RealmManagementRole {
    ManageUsers,
}

impl RealmManagementRole {
    fn to_json_val(self) -> &'static str {
        match self {
            Self::ManageUsers => "manage-users",
        }
    }
}

enum ProviderState {
    Keycloak(Box<KeycloakProvider>),
    Disconnected,
}

trait OidProvider {
    fn decode_jwt(
        &self,
        token: &JwtToken,
    ) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error>;

    fn require_role(&self, token: &JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error>;

    fn require_realm_role(
        &self,
        token: &JwtToken,
        role: RealmManagementRole,
    ) -> Result<TokenData<JwtClaims>, Error>;

    fn require_any_role(
        &self,
        token: &JwtToken,
        roles: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error>;

    async fn create_user<'a>(&self, token: &JwtToken<'a>, user: &User) -> Result<Uuid, Error>;
    async fn remove_user<'a>(&self, token: &JwtToken<'a>, id: Uuid) -> Result<(), Error>;
    async fn get_matching_users<'a>(
        &self,
        token: &JwtToken<'a>,
        email: String,
    ) -> Result<Vec<User>, Error>;
}

pub struct Provider(ProviderState);

impl Provider {
    pub async fn init(config: &config::Config) -> Result<Provider, Error> {
        if let (Some(host), Some(realm), Some(client_id)) = (
            &config.keycloak_host,
            &config.keycloak_realm,
            &config.keycloak_client_id,
        ) {
            let k = KeycloakProvider::fetch(host, realm, client_id.clone()).await?;
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
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_realm_role(
        &self,
        token: &JwtToken,
        role: RealmManagementRole,
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.require_realm_role(token, role),
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
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn is_connected(&self) -> bool {
        match self.0 {
            ProviderState::Keycloak(_) => true,
            ProviderState::Disconnected => false,
        }
    }

    pub async fn create_user<'a>(&self, token: &JwtToken<'a>, user: &User) -> Result<Uuid, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.create_user(token, user).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn remove_user<'a>(&self, token: &JwtToken<'a>, id: Uuid) -> Result<(), Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.remove_user(token, id).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub async fn get_matching_users<'a>(
        &self,
        token: &JwtToken<'a>,
        email: String,
    ) -> Result<Vec<User>, Error> {
        match &self.0 {
            ProviderState::Keycloak(k) => k.get_matching_users(token, email).await,
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
    #[error("Token is missing realm role {0:?}")]
    MissingRealmRole(RealmManagementRole),
    #[error("One of roles missing: {0:?}")]
    MissingOneOfRoles(Vec<Role>),
    #[error("Keycloak is disconnected")]
    Disabled,
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
    name: Option<String>,
}

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 401)]
    Unauthorized(String),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtToken<'r> {
    type Error = NetworkResponse;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("authorization") {
            None => Outcome::Error((
                Status::Unauthorized,
                NetworkResponse::Unauthorized("Expects authorization header".to_string()),
            )),
            Some(string) => {
                let token = string.trim_start_matches("Bearer").trim();
                Outcome::Success(JwtToken { string: token })
            }
        }
    }
}
