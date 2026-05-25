use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod routing;
pub use routing::{ServiceRouter, TargetService};

/// Authentication context extracted from Cognito JWT by the api-gateway.
/// Passed to backend services so they can enforce authorization without
/// re-validating the token.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AuthContext {
    pub sub: String,
    pub email: String,
    pub groups: Vec<String>,
}

/// Request payload sent from api-gateway to backend services via Lambda SDK invoke.
/// Mirrors an HTTP request in a serializable form.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ServiceRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
    pub auth_context: Option<AuthContext>,
}

/// Response payload returned by backend services to api-gateway.
/// Mirrors an HTTP response in a serializable form.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ServiceResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ServiceRequest {
    /// Build a request from an Axum HTTP request.
    /// Only available when the `axum` feature is enabled (in the api-gateway crate).
    #[cfg(feature = "axum")]
    pub async fn from_axum(req: axum::extract::Request) -> anyhow::Result<Self> {
        use axum::body::to_bytes;

        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let query = req
            .uri()
            .query()
            .map(|q| {
                q.split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        Some((
                            parts.next()?.to_string(),
                            parts.next().unwrap_or("").to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut headers = HashMap::new();
        for (name, value) in req.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.as_str().to_lowercase(), v.to_string());
            }
        }

        let body = if method == "GET" || method == "HEAD" {
            None
        } else {
            let bytes = to_bytes(req.into_body(), 1024 * 1024).await?;
            if bytes.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&bytes).to_string())
            }
        };

        Ok(Self {
            method,
            path,
            headers,
            query,
            body,
            auth_context: None,
        })
    }

    /// Shortcut for creating a GET request in tests.
    #[cfg(test)]
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.to_string(),
            ..Default::default()
        }
    }

    /// Shortcut for creating a POST request in tests.
    #[cfg(test)]
    pub fn post(path: &str, body: impl Serialize) -> Self {
        Self {
            method: "POST".to_string(),
            path: path.to_string(),
            body: Some(serde_json::to_string(&body).unwrap_or_default()),
            ..Default::default()
        }
    }
}

impl ServiceResponse {
    /// Create a successful JSON response.
    pub fn ok(body: impl Serialize) -> Self {
        let body = serde_json::to_string(&body).unwrap_or_default();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Self {
            status_code: 200,
            headers,
            body,
        }
    }

    /// Create an error response.
    pub fn error(status_code: u16, message: &str) -> Self {
        let body = serde_json::json!({"error": message}).to_string();
        Self {
            status_code,
            headers: HashMap::new(),
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_roundtrip() {
        let ctx = AuthContext {
            sub: "user-123".to_string(),
            email: "test@example.com".to_string(),
            groups: vec!["admin".to_string()],
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let decoded: AuthContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, decoded);
    }

    #[test]
    fn test_service_request_roundtrip() {
        let req = ServiceRequest {
            method: "POST".to_string(),
            path: "/api/v1/portfolio/jobs".to_string(),
            headers: [("content-type".to_string(), "application/json".to_string())]
                .into_iter()
                .collect(),
            query: HashMap::new(),
            body: Some(r#"{"slug":"scala"}"#.to_string()),
            auth_context: Some(AuthContext {
                sub: "user-123".to_string(),
                email: "test@example.com".to_string(),
                groups: vec![],
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: ServiceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn test_service_response_ok() {
        let resp = ServiceResponse::ok(serde_json::json!({"id": 1}));
        assert_eq!(resp.status_code, 200);
        assert_eq!(
            resp.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_service_response_error() {
        let resp = ServiceResponse::error(404, "not found");
        assert_eq!(resp.status_code, 404);
        assert!(resp.body.contains("not found"));
    }
}
