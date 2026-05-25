use aws_sdk_cognitoidentityprovider::{
    types::{AuthFlowType, ChallengeNameType},
    Client as CognitoClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AuthError;
use crate::state::AuthConfig;

/// Successful authentication result from Cognito.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthTokens {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i32,
}

/// Challenge response from Cognito requiring additional user input.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthChallenge {
    pub challenge_name: String,
    pub session: String,
    pub challenge_parameters: HashMap<String, String>,
}

/// Result of an auth attempt — either tokens or a challenge.
#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum AuthResult {
    Success { tokens: AuthTokens },
    Challenge { challenge: AuthChallenge },
}

/// Sign in with username and password.
pub async fn sign_in(
    client: &CognitoClient,
    config: &AuthConfig,
    username: &str,
    password: &str,
) -> Result<AuthResult, AuthError> {
    if config.dev_mode {
        return Ok(AuthResult::Success {
            tokens: dev_tokens(),
        });
    }

    let mut auth_params = HashMap::new();
    auth_params.insert("USERNAME".to_string(), username.to_string());
    auth_params.insert("PASSWORD".to_string(), password.to_string());

    let resp = client
        .initiate_auth()
        .client_id(&config.client_id)
        .auth_flow(AuthFlowType::UserPasswordAuth)
        .set_auth_parameters(Some(auth_params))
        .send()
        .await
        .map_err(|e| AuthError::Cognito(format!("{e:?}")))?;

    parse_initiate_auth_response(resp)
}

/// Respond to an auth challenge (e.g. NEW_PASSWORD_REQUIRED, SMS_MFA).
pub async fn respond_to_challenge(
    client: &CognitoClient,
    config: &AuthConfig,
    challenge_name: &str,
    session: &str,
    challenge_responses: HashMap<String, String>,
) -> Result<AuthResult, AuthError> {
    if config.dev_mode {
        return Ok(AuthResult::Success {
            tokens: dev_tokens(),
        });
    }

    let challenge = match challenge_name {
        "NEW_PASSWORD_REQUIRED" => ChallengeNameType::NewPasswordRequired,
        "SMS_MFA" => ChallengeNameType::SmsMfa,
        "SOFTWARE_TOKEN_MFA" => ChallengeNameType::SoftwareTokenMfa,
        _ => {
            return Err(AuthError::InvalidInput(format!(
                "unknown challenge: {}",
                challenge_name
            )))
        }
    };

    let resp = client
        .respond_to_auth_challenge()
        .client_id(&config.client_id)
        .challenge_name(challenge)
        .session(session)
        .set_challenge_responses(Some(challenge_responses))
        .send()
        .await
        .map_err(|e| AuthError::Cognito(format!("{e:?}")))?;

    parse_challenge_response(resp)
}

/// Initiate forgot password flow.
pub async fn forgot_password(
    client: &CognitoClient,
    config: &AuthConfig,
    username: &str,
) -> Result<(), AuthError> {
    if config.dev_mode {
        return Ok(());
    }

    client
        .forgot_password()
        .client_id(&config.client_id)
        .username(username)
        .send()
        .await
        .map_err(|e| AuthError::Cognito(format!("{e:?}")))?;

    Ok(())
}

/// Confirm forgot password with code and new password.
pub async fn confirm_forgot_password(
    client: &CognitoClient,
    config: &AuthConfig,
    username: &str,
    confirmation_code: &str,
    new_password: &str,
) -> Result<(), AuthError> {
    if config.dev_mode {
        return Ok(());
    }

    client
        .confirm_forgot_password()
        .client_id(&config.client_id)
        .username(username)
        .confirmation_code(confirmation_code)
        .password(new_password)
        .send()
        .await
        .map_err(|e| AuthError::Cognito(format!("{e:?}")))?;

    Ok(())
}

/// Global sign-out (revoke all tokens).
pub async fn global_sign_out(
    client: &CognitoClient,
    _config: &AuthConfig,
    access_token: &str,
) -> Result<(), AuthError> {
    client
        .global_sign_out()
        .access_token(access_token)
        .send()
        .await
        .map_err(|e| AuthError::Cognito(format!("{e:?}")))?;

    Ok(())
}

// ── Response parsers ─────────────────────────────────────────────────────────

fn parse_initiate_auth_response(
    resp: aws_sdk_cognitoidentityprovider::operation::initiate_auth::InitiateAuthOutput,
) -> Result<AuthResult, AuthError> {
    if let Some(challenge) = resp.challenge_name {
        return Ok(AuthResult::Challenge {
            challenge: AuthChallenge {
                challenge_name: challenge.as_str().to_string(),
                session: resp.session.unwrap_or_default(),
                challenge_parameters: resp
                    .challenge_parameters
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            },
        });
    }

    let result = resp
        .authentication_result
        .ok_or_else(|| AuthError::AuthFailed("no authentication result".to_string()))?;

    Ok(AuthResult::Success {
        tokens: AuthTokens {
            id_token: result.id_token.unwrap_or_default(),
            access_token: result.access_token.unwrap_or_default(),
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
        },
    })
}

fn parse_challenge_response(
    resp: aws_sdk_cognitoidentityprovider::operation::respond_to_auth_challenge::RespondToAuthChallengeOutput,
) -> Result<AuthResult, AuthError> {
    if let Some(challenge) = resp.challenge_name {
        return Ok(AuthResult::Challenge {
            challenge: AuthChallenge {
                challenge_name: challenge.as_str().to_string(),
                session: resp.session.unwrap_or_default(),
                challenge_parameters: resp
                    .challenge_parameters
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            },
        });
    }

    let result = resp
        .authentication_result
        .ok_or_else(|| AuthError::AuthFailed("no authentication result".to_string()))?;

    Ok(AuthResult::Success {
        tokens: AuthTokens {
            id_token: result.id_token.unwrap_or_default(),
            access_token: result.access_token.unwrap_or_default(),
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
        },
    })
}

fn dev_tokens() -> AuthTokens {
    AuthTokens {
        id_token: "dev-id-token".to_string(),
        access_token: "dev-access-token".to_string(),
        refresh_token: Some("dev-refresh-token".to_string()),
        expires_in: 3600,
    }
}
