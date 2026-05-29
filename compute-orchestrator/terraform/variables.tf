variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t3.medium"
}

variable "node_count" {
  description = "Number of compute nodes"
  type        = number
  default     = 3
}

variable "image_digest" {
  description = "Docker image SHA256 digest to pin deployment"
  type        = string
  default     = "latest"
}

variable "allowed_ssh_cidr" {
  description = "CIDR block allowed to SSH into instances"
  type        = string
  default     = "10.0.0.0/8"
}

terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}
