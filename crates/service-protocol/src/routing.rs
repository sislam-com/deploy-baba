/// Target backend service for a given API path.
/// Used by the api-gateway to determine which Lambda to invoke.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetService {
    Portfolio,
    Rag,
    Admin,
    Auth,
    Contact,
    Agent,
    Metrics,
    Health,
}

impl TargetService {
    /// Map an HTTP path to the target service that should handle it.
    pub fn from_path(path: &str) -> Option<Self> {
        let path = path.trim_end_matches('/');

        if path == "/health" {
            return Some(TargetService::Health);
        }

        if path == "/api/v1/metrics" || path.starts_with("/api/v1/metrics/") {
            return Some(TargetService::Metrics);
        }

        if path == "/api/contact" || path == "/api/contact/challenge" {
            return Some(TargetService::Contact);
        }

        if path.starts_with("/api/v1/portfolio/") {
            return Some(TargetService::Portfolio);
        }

        if path.starts_with("/api/v1/rag/") || path == "/api/ask" {
            return Some(TargetService::Rag);
        }

        if path.starts_with("/api/v1/admin/") {
            return Some(TargetService::Admin);
        }

        if path.starts_with("/api/v1/auth/") || path.starts_with("/auth/") {
            return Some(TargetService::Auth);
        }

        if path.starts_with("/api/v1/agent/") {
            return Some(TargetService::Agent);
        }

        None
    }

    /// Build the Lambda function name for this service in a given environment.
    pub fn lambda_name(&self, project: &str, env: &str) -> String {
        let suffix = match self {
            TargetService::Portfolio => "portfolio",
            TargetService::Rag => "rag",
            TargetService::Admin => "admin",
            TargetService::Auth => "auth",
            TargetService::Contact => "contact",
            TargetService::Agent => "agent",
            TargetService::Metrics => "ui",
            TargetService::Health => "ui",
        };
        format!("{}-{}-{}", project, env, suffix)
    }

    /// Returns true if this service should be invoked via Lambda SDK
    /// (i.e., it has its own function, not handled inline by api-gateway).
    pub fn is_backend(&self) -> bool {
        !matches!(self, TargetService::Metrics | TargetService::Health)
    }
}

/// Trait for backend services to implement internal routing.
/// Each service Lambda receives a `ServiceRequest` and must produce a `ServiceResponse`.
pub trait ServiceRouter {
    /// Route the request to the appropriate handler and return a response.
    fn route(
        &self,
        req: super::ServiceRequest,
    ) -> impl std::future::Future<Output = super::ServiceResponse> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_paths() {
        assert_eq!(
            TargetService::from_path("/api/v1/portfolio/jobs"),
            Some(TargetService::Portfolio)
        );
        assert_eq!(
            TargetService::from_path("/api/v1/portfolio/competencies"),
            Some(TargetService::Portfolio)
        );
    }

    #[test]
    fn test_rag_paths() {
        assert_eq!(
            TargetService::from_path("/api/v1/rag/ask"),
            Some(TargetService::Rag)
        );
        assert_eq!(
            TargetService::from_path("/api/ask"),
            Some(TargetService::Rag)
        );
    }

    #[test]
    fn test_admin_paths() {
        assert_eq!(
            TargetService::from_path("/api/v1/admin/jobs"),
            Some(TargetService::Admin)
        );
    }

    #[test]
    fn test_auth_paths() {
        assert_eq!(
            TargetService::from_path("/api/v1/auth/me"),
            Some(TargetService::Auth)
        );
        assert_eq!(
            TargetService::from_path("/auth/callback"),
            Some(TargetService::Auth)
        );
    }

    #[test]
    fn test_contact_paths() {
        assert_eq!(
            TargetService::from_path("/api/contact"),
            Some(TargetService::Contact)
        );
        assert_eq!(
            TargetService::from_path("/api/contact/challenge"),
            Some(TargetService::Contact)
        );
    }

    #[test]
    fn test_inline_paths() {
        assert_eq!(
            TargetService::from_path("/health"),
            Some(TargetService::Health)
        );
        assert_eq!(
            TargetService::from_path("/api/v1/metrics"),
            Some(TargetService::Metrics)
        );
    }

    #[test]
    fn test_agent_paths() {
        assert_eq!(
            TargetService::from_path("/api/v1/agent/cover-letter"),
            Some(TargetService::Agent)
        );
        assert_eq!(
            TargetService::from_path("/api/v1/agent/status"),
            Some(TargetService::Agent)
        );
    }

    #[test]
    fn test_unknown_paths() {
        assert_eq!(TargetService::from_path("/api/v1/unknown"), None);
        assert_eq!(TargetService::from_path("/"), None);
    }

    #[test]
    fn test_lambda_name() {
        assert_eq!(
            TargetService::Portfolio.lambda_name("deploy-baba", "prod"),
            "deploy-baba-prod-portfolio"
        );
        assert_eq!(
            TargetService::Auth.lambda_name("deploy-baba", "dev"),
            "deploy-baba-dev-auth"
        );
        assert_eq!(
            TargetService::Agent.lambda_name("deploy-baba", "prod"),
            "deploy-baba-prod-agent"
        );
    }

    #[test]
    fn test_is_backend() {
        assert!(TargetService::Portfolio.is_backend());
        assert!(TargetService::Rag.is_backend());
        assert!(TargetService::Agent.is_backend());
        assert!(!TargetService::Metrics.is_backend());
        assert!(!TargetService::Health.is_backend());
    }
}
