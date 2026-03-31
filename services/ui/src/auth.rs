use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing token")]
    MissingToken,
    #[error("invalid token: {0}")]
    InvalidToken(String),
}

/// Claims extracted from a validated Cognito ID token.
#[derive(Clone, Debug)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub username: String,
}

/// Shared authentication configuration — built once at startup and stored in AppState.
#[derive(Clone)]
pub struct AuthConfig {
    pub pool_id: String,
    pub client_id: String,
    /// Full FQDN, e.g. "deploy-baba-prod.auth.us-east-1.amazoncognito.com"
    pub cognito_domain: String,
    pub region: String,
    /// App base URL, e.g. "https://sislam.com"
    pub app_domain: String,
    /// JWKS JSON string, embedded at deploy time via COGNITO_JWKS env var.
    /// No network fetch required — Lambda has no outbound HTTPS (VPC without NAT).
    jwks_json: String,
    /// When true, `validate_token` always succeeds (used when COGNITO_POOL_ID is unset).
    pub dev_mode: bool,
}

impl AuthConfig {
    /// Build from environment variables.  Falls back to dev-mode bypass when
    /// `COGNITO_POOL_ID` is absent so `just ui` works without any AWS config.
    ///
    /// No network I/O — JWKS is embedded in the `COGNITO_JWKS` env var at deploy time.
    pub fn from_env() -> Self {
        let pool_id = std::env::var("COGNITO_POOL_ID").unwrap_or_default();

        if pool_id.is_empty() {
            tracing::warn!("COGNITO_POOL_ID not set — auth dev-mode bypass active");
            return AuthConfig {
                pool_id: String::new(),
                client_id: String::new(),
                cognito_domain: String::new(),
                region: "us-east-1".to_string(),
                app_domain: "http://localhost:3000".to_string(),
                jwks_json: String::new(),
                dev_mode: true,
            };
        }

        let client_id = std::env::var("COGNITO_CLIENT_ID").unwrap_or_default();
        let cognito_domain = std::env::var("COGNITO_DOMAIN").unwrap_or_default();
        let region = std::env::var("COGNITO_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let app_domain =
            std::env::var("APP_DOMAIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let jwks_json = std::env::var("COGNITO_JWKS").unwrap_or_default();

        if jwks_json.is_empty() {
            tracing::warn!("COGNITO_JWKS not set — token validation will fail");
        }

        AuthConfig {
            pool_id,
            client_id,
            cognito_domain,
            region,
            app_domain,
            jwks_json,
            dev_mode: false,
        }
    }

    /// Validate a Cognito ID token (RS256).  Returns `Claims` on success.
    /// Kept async for interface stability even though no I/O is performed.
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        if self.dev_mode {
            return Ok(Claims {
                sub: "dev-user".to_string(),
                email: "dev@localhost".to_string(),
                username: "baba-admin".to_string(),
            });
        }

        let jwks_json = self.jwks_json.as_str();

        use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
        use serde_json::Value;

        let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("missing kid in JWT header".to_string()))?;

        let jwks: Value = serde_json::from_str(jwks_json)
            .map_err(|e| AuthError::InvalidToken(format!("JWKS parse: {}", e)))?;

        let key_entry = jwks["keys"]
            .as_array()
            .ok_or_else(|| AuthError::InvalidToken("JWKS has no keys array".to_string()))?
            .iter()
            .find(|k| k["kid"].as_str() == Some(&kid))
            .ok_or_else(|| AuthError::InvalidToken(format!("kid '{}' not in JWKS", kid)))?;

        let n = key_entry["n"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("missing n".to_string()))?;
        let e = key_entry["e"]
            .as_str()
            .ok_or_else(|| AuthError::InvalidToken("missing e".to_string()))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthError::InvalidToken(format!("RSA key: {}", e)))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&[&format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            self.region, self.pool_id
        )]);

        let token_data =
            decode::<serde_json::Map<String, Value>>(token, &decoding_key, &validation)
                .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let claims_map = token_data.claims;
        let sub = claims_map["sub"].as_str().unwrap_or("").to_string();
        let email = claims_map
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let username = claims_map
            .get("cognito:username")
            .and_then(|v| v.as_str())
            .unwrap_or(&sub)
            .to_string();

        Ok(Claims {
            sub,
            email,
            username,
        })
    }
}
