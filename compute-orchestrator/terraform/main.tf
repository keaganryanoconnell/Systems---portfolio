provider "aws" {
  region = var.region
}

data "aws_ami" "amazon_linux_2" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["amzn2-ami-hvm-*-x86_64-gp2"]
  }
}

resource "aws_vpc" "cluster" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "compute-orchestrator-vpc"
  }
}

resource "aws_subnet" "cluster" {
  vpc_id     = aws_vpc.cluster.id
  cidr_block = "10.0.1.0/24"

  tags = {
    Name = "compute-orchestrator-subnet"
  }
}

resource "aws_internet_gateway" "gw" {
  vpc_id = aws_vpc.cluster.id

  tags = {
    Name = "compute-orchestrator-igw"
  }
}

resource "aws_route_table" "cluster" {
  vpc_id = aws_vpc.cluster.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.gw.id
  }
}

resource "aws_route_table_association" "cluster" {
  subnet_id      = aws_subnet.cluster.id
  route_table_id = aws_route_table.cluster.id
}

resource "aws_security_group" "cluster" {
  name        = "compute-orchestrator-sg"
  description = "Security group for compute orchestrator cluster"
  vpc_id      = aws_vpc.cluster.id

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [var.allowed_ssh_cidr]
    description = "SSH — restricted to trusted CIDR"
  }

  ingress {
    from_port   = 7946
    to_port     = 7946
    protocol    = "udp"
    cidr_blocks = [aws_vpc.cluster.cidr_block]
    description = "SWIM gossip — VPC only"
  }

  ingress {
    from_port   = 9000
    to_port     = 9100
    protocol    = "tcp"
    cidr_blocks = [aws_vpc.cluster.cidr_block]
    description = "Actor message ports — VPC only"
  }

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "HTTPS outbound for image pulls + OTLP export"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = [aws_vpc.cluster.cidr_block]
    description = "VPC-internal egress"
  }

  tags = {
    Name = "compute-orchestrator-sg"
  }
}

resource "aws_instance" "node" {
  count         = var.node_count
  ami           = data.aws_ami.amazon_linux_2.id
  instance_type = var.instance_type
  subnet_id     = aws_subnet.cluster.id
  vpc_security_group_ids = [aws_security_group.cluster.id]

  root_block_device {
    encrypted = true
    volume_size = 20
  }

  user_data = <<-EOF
    #!/bin/bash
    docker run -d \
      --name orchestrator \
      --restart always \
      -p 9000:9000 \
      -p 7946:7946/udp \
      ghcr.io/example/compute-orchestrator@${var.image_digest} \
      leader --bind 0.0.0.0:9000
  EOF

  tags = {
    Name = "compute-orchestrator-node-${count.index}"
  }
}

output "node_public_ips" {
  description = "Public IPs of compute nodes"
  value       = aws_instance.node[*].public_ip
}

output "node_private_ips" {
  description = "Private IPs of compute nodes"
  value       = aws_instance.node[*].private_ip
}
