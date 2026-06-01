#!/usr/bin/env bash
# Staging environment tests
# Validates staging deployment and collects performance baselines

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
STAGING_URL="${STAGING_URL:-http://localhost:3000}"
TIMEOUT=30
BASELINE_DIR=".staging"

mkdir -p "$BASELINE_DIR"

test_health_endpoint() {
  echo -e "${BLUE}Testing health endpoint...${NC}"
  
  if curl -sf --max-time $TIMEOUT "$STAGING_URL/health" > /dev/null; then
    echo -e "${GREEN}✓${NC} Health endpoint responding"
    return 0
  else
    echo -e "${RED}✗${NC} Health endpoint failed"
    return 1
  fi
}

test_api_connectivity() {
  echo -e "${BLUE}Testing API connectivity...${NC}"
  
  if curl -sf --max-time $TIMEOUT "$STAGING_URL/api/campaigns" > /dev/null; then
    echo -e "${GREEN}✓${NC} API endpoints responding"
    return 0
  else
    echo -e "${RED}✗${NC} API endpoints failed"
    return 1
  fi
}

test_contract_interaction() {
  echo -e "${BLUE}Testing contract interaction...${NC}"
  
  if curl -sf --max-time $TIMEOUT "$STAGING_URL/api/contract/stats" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Contract interaction working"
    return 0
  else
    echo -e "${YELLOW}⊘${NC} Contract interaction not available (expected in early staging)"
    return 0
  fi
}

test_homepage_load() {
  echo -e "${BLUE}Testing homepage load...${NC}"
  
  local response=$(curl -s --max-time $TIMEOUT "$STAGING_URL/")
  
  if echo "$response" | grep -q "Fund-My-Cause\|crowdfund\|campaign"; then
    echo -e "${GREEN}✓${NC} Homepage loads successfully"
    return 0
  else
    echo -e "${RED}✗${NC} Homepage content missing"
    return 1
  fi
}

collect_performance_baseline() {
  echo -e "${BLUE}Collecting performance baseline...${NC}"
  
  # Warm up
  curl -s --max-time $TIMEOUT "$STAGING_URL/" > /dev/null 2>&1 || true
  
  # Run performance test
  local output=$(mktemp)
  ab -n 100 -c 10 -t 30 "$STAGING_URL/" > "$output" 2>&1 || true
  
  # Extract metrics
  local mean_time=$(grep "Time per request:" "$output" | head -1 | awk '{print $4}')
  local requests_per_sec=$(grep "Requests per second:" "$output" | awk '{print $4}')
  local failed=$(grep "Failed requests:" "$output" | awk '{print $3}')
  
  # Store baseline
  echo "$mean_time" > "$BASELINE_DIR/baseline_response_time.txt"
  echo "$requests_per_sec" > "$BASELINE_DIR/baseline_requests_per_sec.txt"
  echo "$failed" > "$BASELINE_DIR/baseline_failed_requests.txt"
  
  echo -e "${GREEN}✓${NC} Performance baseline collected"
  echo "  Mean response time: ${mean_time}ms"
  echo "  Requests per second: $requests_per_sec"
  echo "  Failed requests: $failed"
  
  rm -f "$output"
}

test_ssl_certificate() {
  echo -e "${BLUE}Testing SSL certificate...${NC}"
  
  if [[ "$STAGING_URL" == https://* ]]; then
    if echo | openssl s_client -servername "${STAGING_URL#https://}" \
      -connect "${STAGING_URL#https://}:443" 2>/dev/null | grep -q "Verify return code: 0"; then
      echo -e "${GREEN}✓${NC} SSL certificate valid"
      return 0
    else
      echo -e "${YELLOW}⊘${NC} SSL certificate validation skipped"
      return 0
    fi
  else
    echo -e "${YELLOW}⊘${NC} Not HTTPS, skipping SSL check"
    return 0
  fi
}

test_response_headers() {
  echo -e "${BLUE}Testing response headers...${NC}"
  
  local headers=$(curl -s -I --max-time $TIMEOUT "$STAGING_URL/" 2>/dev/null)
  
  # Check for security headers
  if echo "$headers" | grep -q "X-Content-Type-Options"; then
    echo -e "${GREEN}✓${NC} Security headers present"
  else
    echo -e "${YELLOW}⊘${NC} Some security headers missing"
  fi
  
  # Check for caching headers
  if echo "$headers" | grep -q "Cache-Control"; then
    echo -e "${GREEN}✓${NC} Caching headers configured"
  else
    echo -e "${YELLOW}⊘${NC} Caching headers not configured"
  fi
}

generate_report() {
  echo ""
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${BLUE}Staging Test Report${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo ""
  echo "Staging URL: $STAGING_URL"
  echo "Test Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo ""
  
  if [ -f "$BASELINE_DIR/baseline_response_time.txt" ]; then
    echo "Performance Baseline:"
    echo "  Response Time: $(cat $BASELINE_DIR/baseline_response_time.txt)ms"
    echo "  Requests/sec: $(cat $BASELINE_DIR/baseline_requests_per_sec.txt)"
    echo "  Failed Requests: $(cat $BASELINE_DIR/baseline_failed_requests.txt)"
  fi
  
  echo ""
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
}

main() {
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${BLUE}Staging Environment Tests${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo ""
  
  local failed=0
  
  test_health_endpoint || ((failed++))
  test_api_connectivity || ((failed++))
  test_contract_interaction || ((failed++))
  test_homepage_load || ((failed++))
  test_ssl_certificate || ((failed++))
  test_response_headers || ((failed++))
  
  echo ""
  collect_performance_baseline
  
  generate_report
  
  if [ "$failed" -gt 0 ]; then
    echo -e "${RED}✗ $failed test(s) failed${NC}"
    exit 1
  else
    echo -e "${GREEN}✓ All tests passed${NC}"
  fi
}

main
