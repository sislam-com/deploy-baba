use aws_sdk_cognitoidentityprovider::Client as CognitoClient;

/// Shared application state for the auth service.
#[derive(Clone)]
pub struct AppState {
    pub config: AuthConfig,
    pub cognito: CognitoClient,
}

/// Authentication configuration — built once at startup.
#[derive(Clone)]
pub struct AuthConfig {
    pub client_id: String,
    /// When true, all auth calls succeed with a synthetic token.
    pub dev_mode: bool,
}

impl AppState {
    /// Build from environment variables.
    pub async fn from_env() -> Self {
        let pool_id = std::env::var("COGNITO_POOL_ID").unwrap_or_default();
        let dev_mode = pool_id.is_empty();

        let config = AuthConfig {
            client_id: std::env::var("COGNITO_CLIENT_ID").unwrap_or_default(),
            dev_mode,
        };

        if dev_mode {
            tracing::warn!("COGNITO_POOL_ID not set — auth dev-mode bypass active");
        }

        let aws_cfg = aws_config::load_from_env().await;
        let cognito = CognitoClient::new(&aws_cfg);

        Self { config, cognito }
    }
}
