# ─── VPC Interface Endpoints (prod-only singletons) ──────────────────────────
#
# Allows the VPC-bound UI Lambda to invoke helper Lambdas and Secrets Manager
# without a NAT Gateway. Shared per-VPC — managed in the prod workspace only.
# Dev Lambda reuses the same VPC endpoints (same VPC, same account).
#
# Cost: ~$7.30/month per interface endpoint (1 AZ).

locals {
  is_prod_vpc = var.environment == "prod"
}

# Security group: accepts HTTPS from the Lambda's security group
resource "aws_security_group" "vpc_endpoints" {
  count       = local.is_prod_vpc ? 1 : 0
  name        = "${var.project_name}-vpc-endpoints-sg"
  description = "Security group for VPC Interface endpoints"
  vpc_id      = data.aws_vpc.default.id

  ingress {
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    security_groups = [aws_security_group.lambda_efs.id]
    description     = "HTTPS from Lambda security group"
  }

  tags = {
    Name = "${var.project_name}-vpc-endpoints-sg"
  }
}

# One subnet only (1 AZ) to minimise hourly cost
data "aws_subnet" "endpoint" {
  count             = local.is_prod_vpc ? 1 : 0
  vpc_id            = data.aws_vpc.default.id
  availability_zone = data.aws_availability_zones.available.names[0]
  default_for_az    = true
}

resource "aws_vpc_endpoint" "lambda" {
  count               = local.is_prod_vpc ? 1 : 0
  vpc_id              = data.aws_vpc.default.id
  service_name        = "com.amazonaws.${var.region}.lambda"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [data.aws_subnet.endpoint[0].id]
  security_group_ids  = [aws_security_group.vpc_endpoints[0].id]
  private_dns_enabled = true

  tags = {
    Name = "${var.project_name}-lambda-endpoint"
  }
}

# ─── VPC Interface Endpoint: Secrets Manager ───────────────────────────────────

resource "aws_vpc_endpoint" "secretsmanager" {
  count               = local.is_prod_vpc ? 1 : 0
  vpc_id              = data.aws_vpc.default.id
  service_name        = "com.amazonaws.${var.region}.secretsmanager"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [data.aws_subnet.endpoint[0].id]
  security_group_ids  = [aws_security_group.vpc_endpoints[0].id]
  private_dns_enabled = true

  tags = {
    Name = "${var.project_name}-secretsmanager-endpoint"
  }
}

# ─── VPC Gateway Endpoint: S3 ─────────────────────────────────────────────────
#
# Free Gateway endpoint — S3 traffic through the AWS backbone.

data "aws_route_tables" "main" {
  count  = local.is_prod_vpc ? 1 : 0
  vpc_id = data.aws_vpc.default.id

  filter {
    name   = "association.main"
    values = ["true"]
  }
}

resource "aws_vpc_endpoint" "s3" {
  count             = local.is_prod_vpc ? 1 : 0
  vpc_id            = data.aws_vpc.default.id
  service_name      = "com.amazonaws.${var.region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = data.aws_route_tables.main[0].ids

  tags = {
    Name = "${var.project_name}-s3-endpoint"
  }
}
