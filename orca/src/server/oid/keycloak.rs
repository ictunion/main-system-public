/// These implementations are very keycloak specific
/// We're not using keycloak library from crates io because we want these to wrork differently
use super::{Error, JwtClaims, JwtToken, OidProvider, RealmManagementRole, Role};
use jsonwebtoken::{self, Algorithm, DecodingKey, TokenData, Validation};
use reqwest;
use rocket::serde::json::json;
use url::Url;
use uuid::Uuid;

use crate::server::jwk;

fn keycloak_url(host: &str, realm: &str) -> String {
    format!("{host}/realms/{realm}")
}

pub struct KeycloakProvider {
    key: DecodingKey,
    validation: Validation,
    client_id: String,
    host: String,
    realm: String,
}

impl KeycloakProvider {
    pub async fn fetch(host: &str, realm: &str, client_id: String) -> Result<Self, Error> {
        let url = keycloak_url(host, realm);
        let key = jwk::fetch_jwk(&format!("{}/protocol/openid-connect/certs", url)).await?;

        // Configure validations
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[keycloak_url(host, realm)]);

        Ok(Self {
            key,
            validation,
            client_id,
            host: host.into(),
            realm: realm.into(),
        })
    }

    fn has_role(&self, claims: &JwtClaims, role: &str) -> bool {
        match claims.resource_access.get(&self.client_id) {
            Some(r) => r.roles.iter().any(|x| *x == role),
            None => false,
        }
    }

    fn has_realm_role(&self, claims: &JwtClaims, role: &str) -> bool {
        match claims.resource_access.get("realm-management") {
            Some(r) => r.roles.iter().any(|x| *x == role),
            None => false,
        }
    }
}

impl OidProvider for KeycloakProvider {
    fn decode_jwt(
        &self,
        token: &JwtToken,
    ) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error> {
        let res = jsonwebtoken::decode::<JwtClaims>(token.string, &self.key, &self.validation);
        if let Err(err) = &res {
            warn!("Faild to decode JWT token {}", token.string);
            warn!("Error: {:?}", err);
        }

        res
    }

    fn require_role(&self, token: &JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error> {
        let token_data = self.decode_jwt(token)?;
        if self.has_role(&token_data.claims, role.to_json_val()) {
            Ok(token_data)
        } else {
            Err(Error::MissingRole(role))
        }
    }

    fn require_realm_role(
        &self,
        token: &JwtToken,
        role: RealmManagementRole,
    ) -> Result<TokenData<JwtClaims>, Error> {
        let token_data = self.decode_jwt(token)?;
        if self.has_realm_role(&token_data.claims, role.to_json_val()) {
            Ok(token_data)
        } else {
            Err(Error::MissingRealmRole(role))
        }
    }

    fn require_any_role(
        &self,
        token: &JwtToken,
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

    async fn create_user<'a>(
        &self,
        token: &JwtToken<'a>,
        user: &super::User,
    ) -> Result<uuid::Uuid, Error> {
        // Keycalok expects json body with data about user
        let json = json!({
            "firstName": user.first_name,
            "lastName": user.last_name,
            "email": user.email,
            "enabled": true,
            "username": user.email,
            "emailVerified": true
        });

        // Send request to create a new user
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/admin/realms/{}/users", self.host, self.realm))
            .json(&json)
            .header("Authorization", format!("Bearer {}", token.string))
            .send()
            .await?;

        let status = response.status();
        debug!("Keycloak response status: {}", status);
        debug!("Keycloak response: {:?}", response);

        if status.is_success() {
            // Keycloak responds with empty body but it includes `Location` with full
            // api path to new user resource.
            // Last segment of this path is uuid identifying new user so we can parse it out of this header.
            match response.headers().get("Location") {
                Some(header) => {
                    let string = header
                        .to_str()
                        .map_err(|_| Error::Parsing(format!("Bad header {:?}", header)))?;
                    let url = Url::parse(string)
                        .map_err(|_| Error::Parsing(format!("Expected URL got {}", string)))?;
                    let uuid = url
                        .path_segments()
                        .ok_or(Error::Parsing(format!("Bad url {url}")))?
                        .last()
                        .ok_or(Error::Parsing(format!("Bad url {url}")))?;

                    Uuid::parse_str(uuid)
                        .map_err(|_| Error::Parsing(format!("Canot parse UUID from {}", uuid)))
                }
                None => Err(Error::Parsing("Missing Location header".to_string())),
            }
        } else {
            Err(Error::Proxy(status))
        }
    }

    async fn remove_user<'a>(&self, token: &JwtToken<'a>, id: uuid::Uuid) -> Result<(), Error> {
        // Send request to create a new user
        let client = reqwest::Client::new();
        let response = client
            .delete(&format!(
                "{}/admin/realms/{}/users/{}",
                self.host, self.realm, id
            ))
            .header("Authorization", format!("Bearer {}", token.string))
            .send()
            .await?;

        let status = response.status();

        debug!("Keycloak response status: {}", status);
        debug!("Keycloak response: {:?}", response);

        if status.is_success() {
            Ok(())
        } else {
            Err(Error::Proxy(status))
        }
    }
}
