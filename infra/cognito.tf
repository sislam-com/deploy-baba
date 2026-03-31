# Cognito User Pool for admin authentication
resource "aws_cognito_user_pool" "baba" {
  name = "${var.project_name}-${var.environment}"

  # Disable self-registration — only admin-created users can log in
  admin_create_user_config {
    allow_admin_create_user_only = true
  }

  password_policy {
    minimum_length                   = 12
    require_uppercase                = true
    require_lowercase                = true
    require_numbers                  = true
    require_symbols                  = true
    temporary_password_validity_days = 7
  }

  auto_verified_attributes = ["email"]

  account_recovery_setting {
    recovery_mechanism {
      name     = "verified_email"
      priority = 1
    }
  }

  deletion_protection = "ACTIVE"

  tags = {
    Name = "${local.lambda_function_name}-cognito"
  }
}

# Cognito hosted UI domain prefix
resource "aws_cognito_user_pool_domain" "baba" {
  domain       = "${var.project_name}-${var.environment}"
  user_pool_id = aws_cognito_user_pool.baba.id
}

# Public app client — implicit grant flow, no client secret, no server-side token exchange
resource "aws_cognito_user_pool_client" "baba_web" {
  name         = "${var.project_name}-web"
  user_pool_id = aws_cognito_user_pool.baba.id

  generate_secret                      = false
  allowed_oauth_flows_user_pool_client = true
  allowed_oauth_flows                  = ["implicit"]
  allowed_oauth_scopes                 = ["openid", "email", "profile"]

  supported_identity_providers = ["COGNITO"]

  callback_urls = [
    "https://${var.domain_name}/auth/callback",
    "http://localhost:3000/auth/callback",
  ]

  logout_urls = [
    "https://${var.domain_name}",
    "http://localhost:3000",
  ]

  explicit_auth_flows = [
    "ALLOW_REFRESH_TOKEN_AUTH",
    "ALLOW_USER_SRP_AUTH",
  ]
}

# Bootstrap admin user — password reset required on first login
resource "aws_cognito_user" "baba_admin" {
  user_pool_id = aws_cognito_user_pool.baba.id
  username     = "baba-admin"

  attributes = {
    email          = var.admin_email
    email_verified = "true"
  }

  temporary_password = var.cognito_temp_password

  lifecycle {
    ignore_changes = [temporary_password]
  }
}

# Fetch JWKS at deploy time so Lambda can validate tokens without outbound network calls
# (Lambda runs in VPC without NAT Gateway — no outbound HTTPS to Cognito endpoints)
data "http" "cognito_jwks" {
  url = "https://cognito-idp.${var.region}.amazonaws.com/${aws_cognito_user_pool.baba.id}/.well-known/jwks.json"
}
