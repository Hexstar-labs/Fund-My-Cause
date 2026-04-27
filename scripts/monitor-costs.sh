#!/usr/bin/env bash
# Monitor and track infrastructure costs
# Usage: ./scripts/monitor-costs.sh [--report] [--alert-threshold <amount>]

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Defaults
REPORT_MODE=false
ALERT_THRESHOLD=1000
COST_LOG_FILE="cost-tracking.log"
COST_REPORT_FILE="cost-report-$(date +%Y-%m-%d).json"

# Helper functions
log_info() { echo -e "${BLUE}➜${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1" >&2; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --report)
      REPORT_MODE=true
      shift
      ;;
    --alert-threshold)
      ALERT_THRESHOLD="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

log_info "Starting cost monitoring..."

# Initialize cost tracking
if [ ! -f "$COST_LOG_FILE" ]; then
  echo "timestamp,category,amount,description" > "$COST_LOG_FILE"
fi

# Function to log cost
log_cost() {
  local category=$1
  local amount=$2
  local description=$3
  echo "$(date -u +%Y-%m-%dT%H:%M:%SZ),$category,$amount,$description" >> "$COST_LOG_FILE"
}

# 1. Track RPC call costs (estimate based on call frequency)
log_info "Analyzing RPC call patterns..."
RPC_CALLS=$(grep -r "soroban-testnet.stellar.org\|soroban-mainnet.stellar.org" . --include="*.ts" --include="*.tsx" --include="*.js" 2>/dev/null | wc -l || echo "0")
RPC_COST=$(echo "$RPC_CALLS * 0.0001" | bc 2>/dev/null || echo "0")
log_cost "RPC_CALLS" "$RPC_COST" "Estimated RPC call costs"
log_success "RPC calls tracked: $RPC_CALLS calls (~\$$RPC_COST)"

# 2. Track storage costs
log_info "Analyzing storage usage..."
STORAGE_SIZE=$(du -sh . 2>/dev/null | awk '{print $1}' || echo "0")
STORAGE_COST=$(du -sb . 2>/dev/null | awk '{printf "%.4f", $1 * 0.000001}' || echo "0")
log_cost "STORAGE" "$STORAGE_COST" "Storage usage: $STORAGE_SIZE"
log_success "Storage tracked: $STORAGE_SIZE (~\$$STORAGE_COST)"

# 3. Track contract deployment costs
log_info "Analyzing contract deployment costs..."
WASM_SIZE=$(stat -f%z target/wasm32-unknown-unknown/release/crowdfund.wasm 2>/dev/null || stat -c%s target/wasm32-unknown-unknown/release/crowdfund.wasm 2>/dev/null || echo "0")
DEPLOY_COST=$(echo "$WASM_SIZE * 0.00001" | bc 2>/dev/null || echo "0")
log_cost "DEPLOYMENT" "$DEPLOY_COST" "Contract deployment (crowdfund.wasm: $WASM_SIZE bytes)"
log_success "Deployment costs tracked: ~\$$DEPLOY_COST"

# 4. Calculate total costs
TOTAL_COST=$(echo "$RPC_COST + $STORAGE_COST + $DEPLOY_COST" | bc 2>/dev/null || echo "0")

# 5. Check alert threshold
if (( $(echo "$TOTAL_COST > $ALERT_THRESHOLD" | bc -l 2>/dev/null || echo "0") )); then
  log_warning "Cost alert: Total costs (\$$TOTAL_COST) exceed threshold (\$$ALERT_THRESHOLD)"
fi

# 6. Generate report if requested
if [ "$REPORT_MODE" = true ]; then
  log_info "Generating cost report..."
  
  # Calculate monthly projection
  DAILY_COST=$TOTAL_COST
  MONTHLY_COST=$(echo "$DAILY_COST * 30" | bc 2>/dev/null || echo "0")
  YEARLY_COST=$(echo "$DAILY_COST * 365" | bc 2>/dev/null || echo "0")
  
  cat > "$COST_REPORT_FILE" << EOF
{
  "report_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "costs": {
    "rpc_calls": {
      "amount": $RPC_COST,
      "description": "RPC call costs",
      "calls_count": $RPC_CALLS
    },
    "storage": {
      "amount": $STORAGE_COST,
      "description": "Storage usage",
      "size_bytes": $WASM_SIZE
    },
    "deployment": {
      "amount": $DEPLOY_COST,
      "description": "Contract deployment"
    }
  },
  "summary": {
    "daily_cost": $TOTAL_COST,
    "monthly_projection": $MONTHLY_COST,
    "yearly_projection": $YEARLY_COST,
    "alert_threshold": $ALERT_THRESHOLD,
    "alert_triggered": $([ $(echo "$TOTAL_COST > $ALERT_THRESHOLD" | bc -l 2>/dev/null || echo "0") -eq 1 ] && echo "true" || echo "false")
  }
}
EOF
  
  log_success "Cost report generated: $COST_REPORT_FILE"
fi

# Print summary
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}                    ${GREEN}COST MONITORING SUMMARY${NC}                    ${BLUE}║${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC} RPC Calls:         ${GREEN}\$$RPC_COST${NC}"
echo -e "${BLUE}║${NC} Storage:           ${GREEN}\$$STORAGE_COST${NC}"
echo -e "${BLUE}║${NC} Deployment:        ${GREEN}\$$DEPLOY_COST${NC}"
echo -e "${BLUE}║${NC} Total Daily Cost:  ${GREEN}\$$TOTAL_COST${NC}"
echo -e "${BLUE}║${NC} Monthly Proj:      ${GREEN}\$$(echo "$TOTAL_COST * 30" | bc 2>/dev/null || echo "0")${NC}"
echo -e "${BLUE}║${NC} Cost Log:          ${GREEN}$COST_LOG_FILE${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
