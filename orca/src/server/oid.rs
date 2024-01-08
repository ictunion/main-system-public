use std::collections::HashMap;

use jsonwebtoken::{self, TokenData};
use reqwest;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use uuid::Uuid;

use super::jwk;
use crate::config;

mod keycloak;
use self::keycloak::KeycloakProvider;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    first_name: String,
    last_name: String,
    email: String,
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
    ManageMembers,
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
            Self::SuperPowers => "super-powers",
        }
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        self.to_json_val().to_string()
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
    Keyclaok(Box<keycloak::KeycloakProvider>),
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
            return Ok(Provider(ProviderState::Keyclaok(Box::new(k))));
        }

        warn!("Keycloak authorization not configured. Authorization disabled");
        Ok(Provider(ProviderState::Disconnected))
    }

    pub fn decode_jwt(&self, token: &JwtToken) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keyclaok(k) => {
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
            ProviderState::Keyclaok(k) => k.require_role(token, role),
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_realm_role(
        &self,
        token: &JwtToken,
        role: RealmManagementRole,
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keyclaok(k) => k.require_realm_role(token, role),
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_any_role(
        &self,
        token: &JwtToken,
        role: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            ProviderState::Keyclaok(k) => k.require_any_role(token, role),
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn is_connected(&self) -> bool {
        match self.0 {
            ProviderState::Keyclaok(_) => true,
            ProviderState::Disconnected => false,
        }
    }

    pub async fn create_user<'a>(&self, token: &JwtToken<'a>, user: &User) -> Result<Uuid, Error> {
        match &self.0 {
            ProviderState::Keyclaok(k) => k.create_user(token, user).await,
            ProviderState::Disconnected => Err(Error::Disabled),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    BadKey(jwk::Error),
    BadToken(jsonwebtoken::errors::Error),
    MissingRole(Role),
    MissingRealmRole(RealmManagementRole),
    MissingOneOfRoles(Vec<Role>),
    Disabled,
    Http(reqwest::Error),
    Parsing(String),
}

impl From<jwk::Error> for Error {
    fn from(value: jwk::Error) -> Self {
        Self::BadKey(value)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::BadToken(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Roles {
    roles: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: uuid::Uuid,
    resource_access: HashMap<String, Roles>,
    pub email: String,
    name: String,
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
