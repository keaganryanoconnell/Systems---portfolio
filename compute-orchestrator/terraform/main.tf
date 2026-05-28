provider "aws" {
  region = var.region
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
  vpc_id                  = aws_vpc.cluster.id
  cidr_block              = "10.0.1.0/24"
  map_public_ip_on_launch = true

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
    cidr_blocks = ["0.0.0.0/0"]
    description = "SSH"
  }

  ingress {
    from_port   = 7946
    to_port     = 7946
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "SWIM gossip"
  }

  ingress {
    from_port   = 9000
    to_port     = 9100
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Actor message ports"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "compute-orchestrator-sg"
  }
}

resource "aws_instance" "node" {
  count         = var.node_count
  ami           = "ami-0c02fb55956c7d316"
  instance_type = var.instance_type
  subnet_id     = aws_subnet.cluster.id
  vpc_security_group_ids = [aws_security_group.cluster.id]

  user_data = <<-EOF
    #!/bin/bash
    docker run -d \
      --name orchestrator \
      --restart always \
      -p 9000:9000 \
      -p 7946:7946/udp \
      ghcr.io/example/compute-orchestrator:${var.image_tag} \
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
