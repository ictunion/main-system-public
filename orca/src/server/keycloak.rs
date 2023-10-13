use jsonwebtoken::{self, Algorithm, DecodingKey, TokenData, Validation};
use std::collections::HashMap;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::Deserialize;
use rocket::serde::Serialize;

use super::jwk;

#[derive(FromForm)]
pub struct JwtToken<'a> {
    #[field(name = "token")]
    string: &'a str,
}

struct ConnectedKeycloak {
    key: DecodingKey,
    validation: Validation,
    client_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    ListApplications,
    ViewApplication,
    ResolveApplications,
    ListMembers,
    ManageMembers,
}

impl Role {
    fn to_json_val(self) -> &'static str {
        match self {
            Self::ListApplications => "list-applications",
            Self::ViewApplication => "view-application",
            Self::ResolveApplications => "resolve-applications",
            Self::ListMembers => "list-members",
            Self::ManageMembers => "manage-members",
        }
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        self.to_json_val().to_string()
    }
}

impl ConnectedKeycloak {
    pub async fn fetch(host: &str, realm: &str, client_id: String) -> Result<Self, Error> {
        let key = jwk::fetch_jwk(&format!(
            "{}/protocol/openid-connect/certs",
            keycloak_url(host, realm)
        ))
        .await?;

        // Configure validations
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[keycloak_url(host, realm)]);

        Ok(Self {
            key,
            validation,
            client_id,
        })
    }

    pub fn decode_jwt(
        &self,
        token: JwtToken,
    ) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error> {
        let res = jsonwebtoken::decode::<JwtClaims>(token.string, &self.key, &self.validation);
        if let Err(err) = &res {
            warn!("Faild to decode JWT token {}", token.string);
            warn!("Error: {:?}", err);
        }

        res
    }

    pub fn require_role(&self, token: JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error> {
        let token_data = self.decode_jwt(token)?;
        if self.has_role(&token_data.claims, role.to_json_val()) {
            Ok(token_data)
        } else {
            Err(Error::MissingRole(role))
        }
    }

    pub fn require_any_role(
        &self,
        token: JwtToken,
        roles: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error> {
        let token_data = self.decode_jwt(token)?;

        for role in roles {
            if self.has_role(&token_data.claims, role.to_json_val()) {
                return Ok(token_data);
            } else {
                continue;
            }
        }

        Err(Error::MissingOneOfRoles(roles.to_vec()))
    }

    fn has_role(&self, claims: &JwtClaims, role: &str) -> bool {
        match claims.resource_access.get(&self.client_id) {
            Some(r) => r.roles.iter().any(|x| *x == role),
            None => false,
        }
    }
}

enum KeycloakState {
    Connected(Box<ConnectedKeycloak>),
    Disconnected,
}

pub struct Keycloak(KeycloakState);

impl Keycloak {
    pub async fn fetch(host: &str, realm: &str, client_id: String) -> Result<Keycloak, Error> {
        let k = ConnectedKeycloak::fetch(host, realm, client_id).await?;
        Ok(Keycloak(KeycloakState::Connected(Box::new(k))))
    }

    pub fn disable() -> Self {
        warn!("Keycloak authorization not configured. Authorization disabled");
        Keycloak(KeycloakState::Disconnected)
    }

    pub fn decode_jwt(&self, token: JwtToken) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            KeycloakState::Connected(k) => {
                let res = k.decode_jwt(token)?;
                Ok(res)
            }
            KeycloakState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_role(&self, token: JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            KeycloakState::Connected(k) => k.require_role(token, role),
            KeycloakState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn require_any_role(
        &self,
        token: JwtToken,
        role: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            KeycloakState::Connected(k) => k.require_any_role(token, role),
            KeycloakState::Disconnected => Err(Error::Disabled),
        }
    }

    pub fn is_connected(&self) -> bool {
        match self.0 {
            KeycloakState::Connected(_) => true,
            KeycloakState::Disconnected => false,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    BadKey(jwk::Error),
    BadToken(jsonwebtoken::errors::Error),
    MissingRole(Role),
    MissingOneOfRoles(Vec<Role>),
    Disabled,
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

#[derive(Debug, Deserialize, Serialize)]
struct Roles {
    roles: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: uuid::Uuid,
    realm_access: Roles,
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
            None => Outcome::Failure((
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

fn keycloak_url(host: &str, realm: &str) -> String {
    format!("{host}/realms/{realm}")
}
