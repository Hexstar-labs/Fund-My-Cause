#!/usr/bin/env bash
# Staging environment monitoring setup
# Configures CloudWatch alarms and dashboards for staging environment

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
CLUSTER_NAME="${CLUSTER_NAME:-fund-my-cause-staging}"
SERVICE_NAME="${SERVICE_NAME:-fund-my-cause-staging-frontend}"
AWS_REGION="${AWS_REGION:-us-east-1}"
SNS_TOPIC_ARN="${SNS_TOPIC_ARN:-}"

setup_cloudwatch_alarms() {
  echo -e "${BLUE}Setting up CloudWatch alarms...${NC}"
  
  # High CPU utilization alarm
  aws cloudwatch put-metric-alarm \
    --alarm-name "$CLUSTER_NAME-high-cpu" \
    --alarm-description "Alert when staging CPU exceeds 80%" \
    --metric-name CPUUtilization \
    --namespace AWS/ECS \
    --statistic Average \
    --period 300 \
    --threshold 80 \
    --comparison-operator GreaterThanThreshold \
    --evaluation-periods 2 \
    --region "$AWS_REGION" \
    $([ -n "$SNS_TOPIC_ARN" ] && echo "--alarm-actions $SNS_TOPIC_ARN" || echo "")
  
  echo -e "${GREEN}✓${NC} CPU alarm created"
  
  # High memory utilization alarm
  aws cloudwatch put-metric-alarm \
    --alarm-name "$CLUSTER_NAME-high-memory" \
    --alarm-description "Alert when staging memory exceeds 80%" \
    --metric-name MemoryUtilization \
    --namespace AWS/ECS \
    --statistic Average \
    --period 300 \
    --threshold 80 \
    --comparison-operator GreaterThanThreshold \
    --evaluation-periods 2 \
    --region "$AWS_REGION" \
    $([ -n "$SNS_TOPIC_ARN" ] && echo "--alarm-actions $SNS_TOPIC_ARN" || echo "")
  
  echo -e "${GREEN}✓${NC} Memory alarm created"
  
  # Task count alarm
  aws cloudwatch put-metric-alarm \
    --alarm-name "$CLUSTER_NAME-low-task-count" \
    --alarm-description "Alert when running task count drops below desired" \
    --metric-name RunningCount \
    --namespace AWS/ECS \
    --statistic Average \
    --period 300 \
    --threshold 1 \
    --comparison-operator LessThanThreshold \
    --evaluation-periods 2 \
    --region "$AWS_REGION" \
    $([ -n "$SNS_TOPIC_ARN" ] && echo "--alarm-actions $SNS_TOPIC_ARN" || echo "")
  
  echo -e "${GREEN}✓${NC} Task count alarm created"
}

setup_cloudwatch_dashboard() {
  echo -e "${BLUE}Setting up CloudWatch dashboard...${NC}"
  
  local dashboard_body=$(cat <<'EOF'
{
  "widgets": [
    {
      "type": "metric",
      "properties": {
        "metrics": [
          ["AWS/ECS", "CPUUtilization", {"stat": "Average"}],
          [".", "MemoryUtilization", {"stat": "Average"}],
          [".", "RunningCount", {"stat": "Average"}],
          [".", "DesiredCount", {"stat": "Average"}]
        ],
        "period": 300,
        "stat": "Average",
        "region": "AWS_REGION_PLACEHOLDER",
        "title": "ECS Service Metrics",
        "yAxis": {"left": {"min": 0, "max": 100}}
      }
    },
    {
      "type": "log",
      "properties": {
        "query": "fields @timestamp, @message | stats count() by @message | sort count() desc",
        "region": "AWS_REGION_PLACEHOLDER",
        "title": "Error Log Analysis"
      }
    },
    {
      "type": "metric",
      "properties": {
        "metrics": [
          ["AWS/ApplicationELB", "TargetResponseTime", {"stat": "Average"}],
          [".", "RequestCount", {"stat": "Sum"}],
          [".", "HTTPCode_Target_5XX_Count", {"stat": "Sum"}]
        ],
        "period": 300,
        "stat": "Average",
        "region": "AWS_REGION_PLACEHOLDER",
        "title": "Load Balancer Metrics"
      }
    }
  ]
}
EOF
)
  
  # Replace placeholder with actual region
  dashboard_body="${dashboard_body//AWS_REGION_PLACEHOLDER/$AWS_REGION}"
  
  aws cloudwatch put-dashboard \
    --dashboard-name "$CLUSTER_NAME-dashboard" \
    --dashboard-body "$dashboard_body" \
    --region "$AWS_REGION"
  
  echo -e "${GREEN}✓${NC} Dashboard created"
}

setup_log_groups() {
  echo -e "${BLUE}Setting up CloudWatch log groups...${NC}"
  
  local log_group="/ecs/$CLUSTER_NAME"
  
  # Create log group if it doesn't exist
  aws logs create-log-group \
    --log-group-name "$log_group" \
    --region "$AWS_REGION" 2>/dev/null || true
  
  # Set retention policy
  aws logs put-retention-policy \
    --log-group-name "$log_group" \
    --retention-in-days 7 \
    --region "$AWS_REGION"
  
  echo -e "${GREEN}✓${NC} Log group configured with 7-day retention"
}

setup_metric_filters() {
  echo -e "${BLUE}Setting up metric filters...${NC}"
  
  local log_group="/ecs/$CLUSTER_NAME"
  
  # Error count metric filter
  aws logs put-metric-filter \
    --log-group-name "$log_group" \
    --filter-name "ErrorCount" \
    --filter-pattern "[ERROR]" \
    --metric-transformations \
      metricName=ErrorCount,metricNamespace=Fund-My-Cause,metricValue=1 \
    --region "$AWS_REGION" 2>/dev/null || true
  
  # Warning count metric filter
  aws logs put-metric-filter \
    --log-group-name "$log_group" \
    --filter-name "WarningCount" \
    --filter-pattern "[WARN]" \
    --metric-transformations \
      metricName=WarningCount,metricNamespace=Fund-My-Cause,metricValue=1 \
    --region "$AWS_REGION" 2>/dev/null || true
  
  echo -e "${GREEN}✓${NC} Metric filters created"
}

setup_sns_topic() {
  echo -e "${BLUE}Setting up SNS topic for alerts...${NC}"
  
  if [ -z "$SNS_TOPIC_ARN" ]; then
    local topic_name="fund-my-cause-staging-alerts"
    
    SNS_TOPIC_ARN=$(aws sns create-topic \
      --name "$topic_name" \
      --region "$AWS_REGION" \
      --query 'TopicArn' \
      --output text)
    
    echo -e "${GREEN}✓${NC} SNS topic created: $SNS_TOPIC_ARN"
  else
    echo -e "${GREEN}✓${NC} Using existing SNS topic: $SNS_TOPIC_ARN"
  fi
}

verify_setup() {
  echo -e "${BLUE}Verifying setup...${NC}"
  
  # Check alarms
  local alarm_count=$(aws cloudwatch describe-alarms \
    --alarm-name-prefix "$CLUSTER_NAME" \
    --region "$AWS_REGION" \
    --query 'MetricAlarms | length(@)' \
    --output text)
  
  echo -e "${GREEN}✓${NC} $alarm_count alarms configured"
  
  # Check dashboard
  local dashboard_exists=$(aws cloudwatch get-dashboard \
    --dashboard-name "$CLUSTER_NAME-dashboard" \
    --region "$AWS_REGION" 2>/dev/null && echo "true" || echo "false")
  
  if [ "$dashboard_exists" = "true" ]; then
    echo -e "${GREEN}✓${NC} Dashboard verified"
  fi
  
  # Check log group
  local log_group_exists=$(aws logs describe-log-groups \
    --log-group-name-prefix "/ecs/$CLUSTER_NAME" \
    --region "$AWS_REGION" \
    --query 'logGroups | length(@)' \
    --output text)
  
  if [ "$log_group_exists" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Log group verified"
  fi
}

main() {
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${BLUE}Staging Environment Monitoring Setup${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo ""
  
  echo "Configuration:"
  echo "  Cluster: $CLUSTER_NAME"
  echo "  Service: $SERVICE_NAME"
  echo "  Region: $AWS_REGION"
  echo ""
  
  setup_sns_topic
  setup_cloudwatch_alarms
  setup_cloudwatch_dashboard
  setup_log_groups
  setup_metric_filters
  verify_setup
  
  echo ""
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${GREEN}✓ Staging monitoring setup completed${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
}

main
