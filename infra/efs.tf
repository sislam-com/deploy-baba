# Data source for default VPC
data "aws_vpc" "default" {
  default = true
}

# Data source for default VPC subnets
data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

# Data source for availability zones
data "aws_availability_zones" "available" {
  state = "available"
}

# EFS File System for SQLite database storage
resource "aws_efs_file_system" "baba_db" {
  encrypted           = true
  performance_mode    = "generalPurpose"
  throughput_mode     = "bursting"

  tags = {
    Name = "${local.lambda_function_name}-efs"
  }
}

# EFS Mount Targets in default VPC (one per availability zone)
resource "aws_efs_mount_target" "baba_db" {
  count          = length(data.aws_availability_zones.available.names)
  file_system_id = aws_efs_file_system.baba_db.id
  subnet_id      = data.aws_subnets.default.ids[count.index]
  security_groups = [aws_security_group.efs.id]
}

# EFS Access Point for Lambda with specific mount path and POSIX user
resource "aws_efs_access_point" "db" {
  file_system_id = aws_efs_file_system.baba_db.id
  root_directory {
    path = "/mnt/db"

    creation_info {
      owner_gid   = 1000
      owner_uid   = 1000
      permissions = "0755"
    }
  }

  posix_user {
    gid = 1000
    uid = 1000
  }

  tags = {
    Name = "${local.lambda_function_name}-access-point"
  }

  depends_on = [aws_efs_mount_target.baba_db]
}

# Security group for EFS
resource "aws_security_group" "efs" {
  name        = "${local.lambda_function_name}-efs-sg"
  description = "Security group for EFS access"
  vpc_id      = data.aws_vpc.default.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.lambda_function_name}-efs-sg"
  }
}

# Security group for Lambda EFS access
resource "aws_security_group" "lambda_efs" {
  name        = "${local.lambda_function_name}-lambda-efs-sg"
  description = "Security group for Lambda to access EFS"
  vpc_id      = data.aws_vpc.default.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.lambda_function_name}-lambda-efs-sg"
  }
}

# Cross-SG rules extracted to break the cycle
resource "aws_security_group_rule" "efs_ingress_from_lambda" {
  type                     = "ingress"
  from_port                = 2049
  to_port                  = 2049
  protocol                 = "tcp"
  security_group_id        = aws_security_group.efs.id
  source_security_group_id = aws_security_group.lambda_efs.id
}

