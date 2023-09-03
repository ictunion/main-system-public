use jsonwebtoken::{self, Algorithm, DecodingKey, TokenData, Validation};
use std::collections::HashMap;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::Deserialize;
use rocket::serde::Serialize;

use super::jwk;

struct ConnectedKeycloak {
    key: DecodingKey,
    validation: Validation,
    client_id: String,
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
        jsonwebtoken::decode::<JwtClaims>(token.0, &self.key, &self.validation)
    }

    pub fn require_role(&self, token: JwtToken, role: &str) -> Result<TokenData<JwtClaims>, Error> {
        let token_data = self.decode_jwt(token)?;
        if self.has_role(&token_data.claims, role) {
            Ok(token_data)
        } else {
            Err(Error::MissingRole(role.to_string()))
        }
    }

    fn has_role(&self, claims: &JwtClaims, role: &str) -> bool {
        match claims.resource_access.get(&self.client_id) {
            Some(r) => r.roles.iter().any(|x| *x == role),
            None => false,
        }
    }
}

pub enum KeycloakState {
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

    pub fn require_role(&self, token: JwtToken, role: &str) -> Result<TokenData<JwtClaims>, Error> {
        match &self.0 {
            KeycloakState::Connected(k) => k.require_role(token, role),
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
    MissingRole(String),
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
    sub: uuid::Uuid,
    realm_access: Roles,
    resource_access: HashMap<String, Roles>,
}

pub struct JwtToken<'a>(&'a str);

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
                Outcome::Success(JwtToken(token))
            }
        }
    }
}

fn keycloak_url(host: &str, realm: &str) -> String {
    format!("{host}/realms/{realm}")
}
