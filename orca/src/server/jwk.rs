use jsonwebtoken::DecodingKey;
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Define a struct to represent the Keycloak JSON Web Key Set (JWKS)
#[derive(Debug, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwk {
    alg: String,
    kty: String,
    #[serde(rename = "use")]
    use_: String,
    n: String,
    e: String,
    kid: String,
    x5c: Option<Vec<String>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get certs: {0}")]
    FailedToGetCerts(#[from] reqwest::Error),
    #[error("Signature key is missing")]
    SignatureKeyMissing,
    #[error("Invalid key: {0}")]
    InvalidKey(#[from] jsonwebtoken::errors::Error),
}

pub async fn fetch_jwk(jwks_url: &str) -> Result<DecodingKey, Error> {
    let jwks: Jwks = reqwest::get(jwks_url).await?.json().await?;

    // Find signature key
    let jwk: &Jwk = jwks
        .keys
        .iter()
        .find(|&key| key.use_ == "sig")
        .ok_or(Error::SignatureKeyMissing)?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    Ok(decoding_key)
}
