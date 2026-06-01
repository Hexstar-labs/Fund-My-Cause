# Staging Environment Infrastructure

This document covers the complete staging environment setup for Fund-My-Cause.

## Overview

The staging environment mirrors production with:
- Stellar Testnet deployment
- Automated contract deployment
- Staging-specific configuration
- Monitoring and alerting
- Performance baseline collection

## Infrastructure as Code

### Terraform Configuration

```hcl
# terraform/staging.tf

resource "aws_ecs_cluster" "staging" {
  name = "fund-my-cause-staging"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

resource "aws_ecs_task_definition" "staging_frontend" {
  family                   = "fund-my-cause-staging-frontend"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "256"
  memory                   = "512"

  container_definitions = jsonencode([{
    name      = "frontend"
    image     = "fund-my-cause:staging-latest"
    essential = true
    portMappings = [{
      containerPort = 3000
      hostPort      = 3000
      protocol      = "tcp"
    }]
    environment = [
      {
        name  = "NEXT_PUBLIC_NETWORK_PASSPHRASE"
        value = "Test SDF Network ; September 2015"
      },
      {
        name  = "NEXT_PUBLIC_RPC_URL"
        value = "https://soroban-testnet.stellar.org"
      }
    ]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.staging.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
  }])
}

resource "aws_ecs_service" "staging_frontend" {
  name            = "fund-my-cause-staging-frontend"
  cluster         = aws_ecs_cluster.staging.id
  task_definition = aws_ecs_task_definition.staging_frontend.arn
  desired_count   = 2
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = var.private_subnets
    security_groups  = [aws_security_group.staging_ecs.id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.staging.arn
    container_name   = "frontend"
    container_port   = 3000
  }
}

resource "aws_cloudwatch_log_group" "staging" {
  name              = "/ecs/fund-my-cause-staging"
  retention_in_days = 7
}

resource "aws_cloudwatch_dashboard" "staging" {
  dashboard_name = "fund-my-cause-staging"

  dashboard_body = jsonencode({
    widgets = [
      {
        type = "metric"
        properties = {
          metrics = [
            ["AWS/ECS", "CPUUtilization", { stat = "Average" }],
            [".", "MemoryUtilization", { stat = "Average" }]
          ]
          period = 300
          stat   = "Average"
          region = var.aws_region
          title  = "ECS Metrics"
        }
      }
    ]
  })
}
```

## Deployment Automation

### Staging Deployment Workflow

```yaml
# .github/workflows/deploy-staging.yml (enhanced)

name: Deploy to Staging

on:
  push:
    branches: [develop]
  pull_request:
    branches: [develop]

jobs:
  deploy-staging:
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: arn:aws:iam::${{ secrets.AWS_ACCOUNT_ID }}:role/GitHubActionsRole
          aws-region: us-east-1

      - name: Build and push Docker image
        run: |
          aws ecr get-login-password --region us-east-1 | \
            docker login --username AWS --password-stdin ${{ secrets.ECR_REGISTRY }}
          
          docker build -f apps/interface/Dockerfile \
            -t ${{ secrets.ECR_REGISTRY }}/fund-my-cause:staging-${{ github.sha }} \
            -t ${{ secrets.ECR_REGISTRY }}/fund-my-cause:staging-latest \
            .
          
          docker push ${{ secrets.ECR_REGISTRY }}/fund-my-cause:staging-${{ github.sha }}
          docker push ${{ secrets.ECR_REGISTRY }}/fund-my-cause:staging-latest

      - name: Deploy contracts to testnet
        run: |
          ./scripts/deploy.sh \
            ${{ secrets.STELLAR_TESTNET_ACCOUNT }} \
            ${{ secrets.STELLAR_TESTNET_TOKEN }} \
            1000 \
            $(date -d "+30 days" +%s) \
            10 \
            "Staging Campaign" \
            "Testing environment" \
            null \
            ${{ secrets.REGISTRY_CONTRACT_ID }}

      - name: Update ECS service
        run: |
          aws ecs update-service \
            --cluster fund-my-cause-staging \
            --service fund-my-cause-staging-frontend \
            --force-new-deployment

      - name: Wait for deployment
        run: |
          aws ecs wait services-stable \
            --cluster fund-my-cause-staging \
            --services fund-my-cause-staging-frontend

      - name: Run staging tests
        run: |
          npm run test:staging

      - name: Post deployment info
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## 🚀 Staging Deployment
              
              ✅ Deployment successful!
              
              **Staging URL:** https://staging.fund-my-cause.example.com
              **Contracts:** 
              - Crowdfund: ${{ env.CROWDFUND_CONTRACT_ID }}
              - Registry: ${{ env.REGISTRY_CONTRACT_ID }}
              
              **Test Instructions:**
              1. Connect Freighter wallet to Testnet
              2. Fund account at https://laboratory.stellar.org
              3. Visit staging URL and test campaign creation
              `
            })
```

## Staging Monitoring

### CloudWatch Monitoring

```bash
#!/usr/bin/env bash
# scripts/setup-staging-monitoring.sh

set -euo pipefail

# Create alarms for staging environment
aws cloudwatch put-metric-alarm \
  --alarm-name staging-high-cpu \
  --alarm-description "Alert when staging CPU exceeds 80%" \
  --metric-name CPUUtilization \
  --namespace AWS/ECS \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2

aws cloudwatch put-metric-alarm \
  --alarm-name staging-high-memory \
  --alarm-description "Alert when staging memory exceeds 80%" \
  --metric-name MemoryUtilization \
  --namespace AWS/ECS \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2

aws cloudwatch put-metric-alarm \
  --alarm-name staging-deployment-failures \
  --alarm-description "Alert on deployment failures" \
  --metric-name DeploymentFailures \
  --namespace Fund-My-Cause \
  --statistic Sum \
  --period 300 \
  --threshold 1 \
  --comparison-operator GreaterThanOrEqualToThreshold \
  --evaluation-periods 1

echo "✓ Staging monitoring alarms created"
```

## Staging Testing

### Automated Test Suite

```bash
#!/usr/bin/env bash
# scripts/test-staging.sh

set -euo pipefail

STAGING_URL="${STAGING_URL:-https://staging.fund-my-cause.example.com}"

echo "Running staging tests against $STAGING_URL"

# Health check
echo "Testing health endpoint..."
curl -sf "$STAGING_URL/health" > /dev/null || exit 1

# API connectivity
echo "Testing API endpoints..."
curl -sf "$STAGING_URL/api/campaigns" > /dev/null || exit 1

# Contract interaction
echo "Testing contract interaction..."
curl -sf "$STAGING_URL/api/contract/stats" > /dev/null || exit 1

# Performance baseline
echo "Collecting performance baseline..."
ab -n 100 -c 10 "$STAGING_URL/" > /tmp/staging_perf.txt

# Extract metrics
MEAN_TIME=$(grep "Time per request:" /tmp/staging_perf.txt | head -1 | awk '{print $4}')
echo "Mean response time: ${MEAN_TIME}ms"

# Store baseline
mkdir -p .staging
echo "$MEAN_TIME" > .staging/baseline_response_time.txt

echo "✓ Staging tests completed"
```

## Staging Documentation

### Environment Configuration

```bash
# apps/interface/.env.staging

NEXT_PUBLIC_CONTRACT_ID=CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
NEXT_PUBLIC_REGISTRY_CONTRACT_ID=CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
NEXT_PUBLIC_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Test SDF Network ; September 2015
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_ENVIRONMENT=staging
```

## Staging Runbook

### Common Issues

**Deployment Stuck**
```bash
# Check ECS service status
aws ecs describe-services \
  --cluster fund-my-cause-staging \
  --services fund-my-cause-staging-frontend

# View task logs
aws logs tail /ecs/fund-my-cause-staging --follow
```

**Contract Deployment Failed**
```bash
# Check Stellar testnet status
curl https://horizon-testnet.stellar.org/health

# Verify account funding
stellar account info --account <ACCOUNT_ID> --network testnet
```

**Performance Degradation**
```bash
# Check CloudWatch metrics
aws cloudwatch get-metric-statistics \
  --namespace AWS/ECS \
  --metric-name CPUUtilization \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 300 \
  --statistics Average
```

## References

- [AWS ECS Documentation](https://docs.aws.amazon.com/ecs/)
- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Stellar Testnet](https://developers.stellar.org/docs/learn/networks)
