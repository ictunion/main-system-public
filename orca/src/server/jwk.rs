use jsonwebtoken::DecodingKey;
use reqwest;
use serde::{Deserialize, Serialize};

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

#[derive(Debug)]
pub enum Error {
    FaildToGetCerts(reqwest::Error),
    SignatureKeyMissing,
    InvalidKey(jsonwebtoken::errors::Error),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::FaildToGetCerts(value)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidKey(value)
    }
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
