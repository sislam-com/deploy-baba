use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use mcp_rs::{build_server, config, initialize_workspace};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<config::Config> = OnceLock::new();

#[derive(Clone, Debug)]
struct Claims {
    sub: String,
    email: String,
}

#[derive(Clone)]
struct AuthConfig {
    pool_id: String,
    client_id: String,
    region: String,
    jwks_json: String,
}

impl AuthConfig {
    fn from_env() -> Self {
        Self {
            pool_id: std::env::var("COGNITO_POOL_ID").unwrap_or_default(),
            client_id: std::env::var("COGNITO_CLIENT_ID").unwrap_or_default(),
            region: std::env::var("COGNITO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            jwks_json: std::env::var("COGNITO_JWKS").unwrap_or_default(),
        }
    }

    fn validate(&self, token: &str) -> Result<Claims, String> {
        if self.pool_id.is_empty() || self.client_id.is_empty() || self.jwks_json.is_empty() {
            return Err("Cognito auth is not configured".to_string());
        }

        let header = decode_header(token).map_err(|e| e.to_string())?;
        let kid = header
            .kid
            .ok_or_else(|| "missing kid in JWT header".to_string())?;
        let jwks: Value = serde_json::from_str(&self.jwks_json).map_err(|e| e.to_string())?;
        let key_entry = jwks["keys"]
            .as_array()
            .ok_or_else(|| "JWKS has no keys array".to_string())?
            .iter()
            .find(|key| key["kid"].as_str() == Some(&kid))
            .ok_or_else(|| format!("kid '{}' not in JWKS", kid))?;
        let n = key_entry["n"]
            .as_str()
            .ok_or_else(|| "missing RSA modulus".to_string())?;
        let e = key_entry["e"]
            .as_str()
            .ok_or_else(|| "missing RSA exponent".to_string())?;
        let decoding_key = DecodingKey::from_rsa_components(n, e).map_err(|e| e.to_string())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&[&format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            self.region, self.pool_id
        )]);

        let token_data =
            decode::<serde_json::Map<String, Value>>(token, &decoding_key, &validation)
                .map_err(|e| e.to_string())?;
        let claims = token_data.claims;
        Ok(Claims {
            sub: claims["sub"].as_str().unwrap_or("").to_string(),
            email: claims["email"].as_str().unwrap_or("").to_string(),
        })
    }
}

fn config_path() -> PathBuf {
    std::env::var("MCP_RS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/task/mcp-rs.toml"))
}

fn mcp_config() -> &'static config::Config {
    CONFIG.get_or_init(|| {
        let loaded = config::load_config_from(Some(config_path()));
        initialize_workspace(&loaded);
        loaded
    })
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let path = req.uri().path();
    let method = req.method().as_str();

    if method == "GET" && path == "/mcp/health" {
        return json_response(
            200,
            json!({
                "status": "ok",
                "service": "mcp-gateway",
                "mode": "private-cognito",
            }),
        );
    }

    if method != "POST" || path != "/mcp" {
        return text_response(404, "not found");
    }

    let auth = AuthConfig::from_env();
    let claims = match bearer_token(&req).and_then(|token| auth.validate(token).ok()) {
        Some(claims) => claims,
        None => return text_response(401, "unauthorized"),
    };

    let body = match req.body() {
        Body::Text(text) => text.clone(),
        Body::Binary(bytes) => String::from_utf8(bytes.clone()).unwrap_or_default(),
        Body::Empty => String::new(),
    };

    tracing::info!(
        sub = %claims.sub,
        email = %claims.email,
        "authorized MCP request"
    );

    let server = build_server(mcp_config());
    let response = server.handle_request("lambda_req", &body);
    json_response(200, response)
}

fn bearer_token(req: &Request) -> Option<&str> {
    let value = req.headers().get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ")
}

fn json_response(status: u16, value: Value) -> Result<Response<Body>, Error> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(value.to_string()))
        .map_err(Into::into)
}

fn text_response(status: u16, text: &str) -> Result<Response<Body>, Error> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::Text(text.to_string()))
        .map_err(Into::into)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    run(service_fn(handler)).await
}
