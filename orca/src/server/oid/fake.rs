//! Identity provider for tests: real token checks against an HS256 test key,
//! and an in-memory stand-in for the Keycloak Admin and Account APIs.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use jsonwebtoken::{EncodingKey, Header, TokenData};
use serde_json::json;
use uuid::Uuid;

use super::keycloak::KeycloakProvider;
use super::{Error, JwtClaims, JwtToken, NewUser, OidProvider, Role, User};

const SECRET: &[u8] = b"orca-test-secret";
const ISSUER: &str = "orca-test";

/// Clones share state, so a test keeps one to arrange group membership and to
/// inspect admin calls made by the handlers.
#[derive(Clone)]
pub struct FakeProvider {
    tokens: KeycloakProvider,
    /// Keycloak user id → ids of the groups the user is in
    groups: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
    /// Admin API calls in order, e.g. `remove_user <uuid>`
    calls: Arc<Mutex<Vec<String>>>,
}

impl std::fmt::Debug for FakeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeProvider").finish_non_exhaustive()
    }
}

impl Default for FakeProvider {
    fn default() -> Self {
        Self {
            tokens: KeycloakProvider::for_test(SECRET, ISSUER),
            groups: Arc::default(),
            calls: Arc::default(),
        }
    }
}

impl FakeProvider {
    /// Group Keycloak answers 404 for
    pub const MISSING_GROUP: Uuid = Uuid::from_u128(0x404);

    /// Signed token for `sub` holding `roles` on the `orca` client
    pub fn token(sub: Uuid, roles: &[Role]) -> String {
        let roles: Vec<&str> = roles.iter().map(|role| role.to_json_val()).collect();
        let claims = json!({
            "sub": sub,
            "iss": ISSUER,
            "aud": "orca",
            "exp": chrono::Utc::now().timestamp() + 3600,
            "email": format!("{sub}@example.com"),
            "name": "Test User",
            "resource_access": { "orca": { "roles": roles } },
        });
        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET),
        )
        .expect("test claims always encode")
    }

    pub fn add_to_group(&self, user: Uuid, group: Uuid) {
        self.groups_mut().entry(user).or_default().push(group);
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn record(&self, call: String) {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call);
    }

    fn groups_mut(&self) -> std::sync::MutexGuard<'_, HashMap<Uuid, Vec<Uuid>>> {
        self.groups.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl OidProvider for FakeProvider {
    fn decode_jwt(
        &self,
        token: &JwtToken,
    ) -> Result<TokenData<JwtClaims>, jsonwebtoken::errors::Error> {
        self.tokens.decode_jwt(token)
    }

    fn require_role(&self, token: &JwtToken, role: Role) -> Result<TokenData<JwtClaims>, Error> {
        self.tokens.require_role(token, role)
    }

    fn require_any_role(
        &self,
        token: &JwtToken,
        roles: &[Role],
    ) -> Result<TokenData<JwtClaims>, Error> {
        self.tokens.require_any_role(token, roles)
    }

    async fn create_user(&self, user: &NewUser) -> Result<Uuid, Error> {
        self.record(format!("create_user {}", user.email));
        Ok(Uuid::new_v4())
    }

    async fn remove_user(&self, id: Uuid) -> Result<(), Error> {
        self.record(format!("remove_user {id}"));
        self.groups_mut().remove(&id);
        Ok(())
    }

    async fn get_matching_users(&self, email: String) -> Result<Vec<User>, Error> {
        self.record(format!("get_matching_users {email}"));
        Ok(Vec::new())
    }

    async fn connect_keycloak_user_and_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error> {
        self.record(format!("connect {keycloak_user_id} {keycloak_group_id}"));
        self.add_to_group(keycloak_user_id, keycloak_group_id);
        Ok(())
    }

    async fn remove_keycloak_user_from_group(
        &self,
        keycloak_user_id: Uuid,
        keycloak_group_id: Uuid,
    ) -> Result<(), Error> {
        self.record(format!("disconnect {keycloak_user_id} {keycloak_group_id}"));
        if let Some(groups) = self.groups_mut().get_mut(&keycloak_user_id) {
            groups.retain(|group| *group != keycloak_group_id);
        }
        Ok(())
    }

    async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<Uuid>, Error> {
        if group_id == Self::MISSING_GROUP {
            return Err(Error::Proxy(reqwest::StatusCode::NOT_FOUND));
        }
        Ok(self
            .groups_mut()
            .iter()
            .filter(|(_, groups)| groups.contains(&group_id))
            .map(|(user, _)| *user)
            .collect())
    }

    async fn get_own_groups(&self, token: &JwtToken) -> Result<Vec<Uuid>, Error> {
        let sub = self.decode_jwt(token)?.claims.sub;
        Ok(self.groups_mut().get(&sub).cloned().unwrap_or_default())
    }
}
