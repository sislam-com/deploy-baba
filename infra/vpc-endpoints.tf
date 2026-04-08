# ─── VPC Interface Endpoint: Lambda ───────────────────────────────────────────
#
# Allows the VPC-bound UI Lambda to invoke the email Lambda via SDK without
# a NAT Gateway. The Lambda API endpoint (lambda.region.amazonaws.com) resolves
# to the endpoint's private IP when private_dns_enabled = true.
#
# Cost: ~$7.30/month for 1 AZ.

# Security group: accepts HTTPS from the Lambda's security group
resource "aws_security_group" "vpc_endpoints" {
  name        = "${local.lambda_function_name}-vpc-endpoints-sg"
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
    Name = "${local.lambda_function_name}-vpc-endpoints-sg"
  }
}

# One subnet only (1 AZ) to minimise hourly cost
data "aws_subnet" "endpoint" {
  vpc_id            = data.aws_vpc.default.id
  availability_zone = data.aws_availability_zones.available.names[0]
  default_for_az    = true
}

resource "aws_vpc_endpoint" "lambda" {
  vpc_id              = data.aws_vpc.default.id
  service_name        = "com.amazonaws.${var.region}.lambda"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [data.aws_subnet.endpoint.id]
  security_group_ids  = [aws_security_group.vpc_endpoints.id]
  private_dns_enabled = true

  tags = {
    Name = "${local.lambda_function_name}-lambda-endpoint"
  }
}
