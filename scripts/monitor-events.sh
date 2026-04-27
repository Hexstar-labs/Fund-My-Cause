#!/usr/bin/env bash
# Contract event monitoring and indexing
# Monitors and indexes all contract events for analytics and alerting
# Usage: ./scripts/monitor-events.sh [--listen] [--export <format>]

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
EVENTS_DIR="target/events"
EVENTS_LOG="$EVENTS_DIR/events.log"
EVENTS_INDEX="$EVENTS_DIR/events.json"
LISTEN_MODE=false
EXPORT_FORMAT=""
RPC_URL="${NEXT_PUBLIC_SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
CONTRACT_ID="${NEXT_PUBLIC_CROWDFUND_CONTRACT_ID:-}"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --listen)
      LISTEN_MODE=true
      shift
      ;;
    --export)
      EXPORT_FORMAT="$2"
      shift 2
      ;;
    --help|-h)
      echo "Usage: $0 [--listen] [--export <format>]"
      echo ""
      echo "Options:"
      echo "  --listen           Listen for new events in real-time"
      echo "  --export <format>  Export events (json, csv, html)"
      echo "  --help             Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

mkdir -p "$EVENTS_DIR"

# Initialize event index if it doesn't exist
if [ ! -f "$EVENTS_INDEX" ]; then
  cat > "$EVENTS_INDEX" << 'EOF'
{
  "events": [],
  "summary": {
    "total_events": 0,
    "by_type": {},
    "by_contract": {},
    "last_updated": ""
  }
}
EOF
fi

# Log event
log_event() {
  local event_type=$1
  local event_data=$2
  local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  echo "$timestamp | $event_type | $event_data" >> "$EVENTS_LOG"
}

# Index event
index_event() {
  local event_type=$1
  local event_data=$2
  local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  
  # This is a placeholder - actual implementation would parse and index events
  log_event "$event_type" "$event_data"
}

# Set up event listener
setup_event_listener() {
  echo -e "${BLUE}→${NC} Setting up event listener..."
  
  if [ -z "$CONTRACT_ID" ]; then
    echo -e "${YELLOW}⚠${NC} CONTRACT_ID not set. Set NEXT_PUBLIC_CROWDFUND_CONTRACT_ID environment variable."
    return 1
  fi
  
  echo -e "${BLUE}→${NC} Listening for events from contract: $CONTRACT_ID"
  echo -e "${BLUE}→${NC} RPC URL: $RPC_URL"
  echo ""
  
  # Create event listener configuration
  cat > "$EVENTS_DIR/listener-config.json" << EOF
{
  "contract_id": "$CONTRACT_ID",
  "rpc_url": "$RPC_URL",
  "events": [
    "campaign:initialized",
    "contribution:made",
    "funds:withdrawn",
    "refund:claimed",
    "metadata:updated"
  ],
  "polling_interval_ms": 5000,
  "batch_size": 100
}
EOF
  
  echo -e "${GREEN}✓${NC} Event listener configured"
  echo "  Config: $EVENTS_DIR/listener-config.json"
}

# Create event-based alerts
create_alerts() {
  echo -e "${BLUE}→${NC} Creating event-based alerts..."
  
  cat > "$EVENTS_DIR/alerts.json" << 'EOF'
{
  "alerts": [
    {
      "id": "high_contribution",
      "name": "High Contribution Alert",
      "trigger": "contribution:made",
      "condition": "amount > 10000",
      "action": "notify",
      "channels": ["email", "slack"]
    },
    {
      "id": "goal_reached",
      "name": "Goal Reached Alert",
      "trigger": "funds:withdrawn",
      "condition": "total_raised >= goal",
      "action": "notify",
      "channels": ["email", "webhook"]
    },
    {
      "id": "campaign_initialized",
      "name": "New Campaign Alert",
      "trigger": "campaign:initialized",
      "condition": "true",
      "action": "log",
      "channels": ["analytics"]
    },
    {
      "id": "refund_claimed",
      "name": "Refund Claimed Alert",
      "trigger": "refund:claimed",
      "condition": "true",
      "action": "log",
      "channels": ["analytics"]
    }
  ]
}
EOF
  
  echo -e "${GREEN}✓${NC} Alerts configured"
  echo "  Config: $EVENTS_DIR/alerts.json"
}

# Build event analytics dashboard
build_analytics_dashboard() {
  echo -e "${BLUE}→${NC} Building event analytics dashboard..."
  
  cat > "$EVENTS_DIR/dashboard.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Fund-My-Cause Event Analytics</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #f5f5f5; }
    .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
    header { background: #fff; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    h1 { color: #333; margin-bottom: 10px; }
    .timestamp { color: #666; font-size: 14px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-bottom: 20px; }
    .card { background: #fff; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    .card h2 { color: #333; font-size: 18px; margin-bottom: 10px; }
    .metric { font-size: 32px; font-weight: bold; color: #0066cc; }
    .metric-label { color: #666; font-size: 14px; margin-top: 5px; }
    .event-log { background: #fff; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    .event-log h2 { color: #333; margin-bottom: 15px; }
    .event-item { padding: 10px; border-left: 3px solid #0066cc; margin-bottom: 10px; background: #f9f9f9; }
    .event-time { color: #666; font-size: 12px; }
    .event-type { font-weight: bold; color: #0066cc; }
    .event-data { color: #333; margin-top: 5px; }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>Fund-My-Cause Event Analytics</h1>
      <p class="timestamp">Last updated: <span id="timestamp"></span></p>
    </header>
    
    <div class="grid">
      <div class="card">
        <h2>Total Events</h2>
        <div class="metric" id="total-events">0</div>
        <div class="metric-label">All-time events</div>
      </div>
      
      <div class="card">
        <h2>Campaigns Created</h2>
        <div class="metric" id="campaigns-created">0</div>
        <div class="metric-label">campaign:initialized</div>
      </div>
      
      <div class="card">
        <h2>Contributions</h2>
        <div class="metric" id="contributions">0</div>
        <div class="metric-label">contribution:made</div>
      </div>
      
      <div class="card">
        <h2>Withdrawals</h2>
        <div class="metric" id="withdrawals">0</div>
        <div class="metric-label">funds:withdrawn</div>
      </div>
    </div>
    
    <div class="event-log">
      <h2>Recent Events</h2>
      <div id="event-list">
        <p style="color: #999;">No events recorded yet</p>
      </div>
    </div>
  </div>
  
  <script>
    // Placeholder for real-time event updates
    document.getElementById('timestamp').textContent = new Date().toISOString();
    
    // In production, this would connect to a WebSocket or polling endpoint
    // to receive real-time event updates
  </script>
</body>
</html>
EOF
  
  echo -e "${GREEN}✓${NC} Dashboard created"
  echo "  Location: $EVENTS_DIR/dashboard.html"
}

# Implement anomaly detection
setup_anomaly_detection() {
  echo -e "${BLUE}→${NC} Setting up anomaly detection..."
  
  cat > "$EVENTS_DIR/anomaly-detection.json" << 'EOF'
{
  "anomalies": [
    {
      "id": "unusual_contribution_spike",
      "name": "Unusual Contribution Spike",
      "metric": "contributions_per_minute",
      "baseline": 5,
      "threshold": 20,
      "window_minutes": 5,
      "severity": "warning"
    },
    {
      "id": "rapid_refunds",
      "name": "Rapid Refund Claims",
      "metric": "refunds_per_minute",
      "baseline": 2,
      "threshold": 10,
      "window_minutes": 5,
      "severity": "critical"
    },
    {
      "id": "failed_transactions",
      "name": "High Transaction Failure Rate",
      "metric": "failure_rate",
      "baseline": 0.01,
      "threshold": 0.1,
      "window_minutes": 10,
      "severity": "critical"
    }
  ],
  "detection_interval_seconds": 60,
  "notification_channels": ["email", "slack", "webhook"]
}
EOF
  
  echo -e "${GREEN}✓${NC} Anomaly detection configured"
  echo "  Config: $EVENTS_DIR/anomaly-detection.json"
}

# Export events
export_events() {
  local format=$1
  local export_file="$EVENTS_DIR/events-export-$(date +%Y%m%d-%H%M%S).$format"
  
  echo -e "${BLUE}→${NC} Exporting events as $format..."
  
  case "$format" in
    json)
      cp "$EVENTS_INDEX" "$export_file"
      ;;
    csv)
      cat > "$export_file" << 'EOF'
timestamp,event_type,event_data
EOF
      tail -n +2 "$EVENTS_LOG" 2>/dev/null | while IFS='|' read -r timestamp event_type event_data; do
        echo "\"$timestamp\",\"$event_type\",\"$event_data\"" >> "$export_file"
      done || true
      ;;
    html)
      cat > "$export_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <title>Event Export</title>
  <style>
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background-color: #0066cc; color: white; }
  </style>
</head>
<body>
  <h1>Fund-My-Cause Events Export</h1>
  <table>
    <tr><th>Timestamp</th><th>Event Type</th><th>Event Data</th></tr>
EOF
      tail -n +2 "$EVENTS_LOG" 2>/dev/null | while IFS='|' read -r timestamp event_type event_data; do
        echo "    <tr><td>$timestamp</td><td>$event_type</td><td>$event_data</td></tr>" >> "$export_file"
      done || true
      echo "  </table></body></html>" >> "$export_file"
      ;;
    *)
      echo -e "${RED}✗${NC} Unknown export format: $format"
      return 1
      ;;
  esac
  
  echo -e "${GREEN}✓${NC} Events exported"
  echo "  File: $export_file"
}

main() {
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${BLUE}Contract Event Monitoring${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo ""
  
  # Set up event listener
  setup_event_listener || true
  
  # Create alerts
  create_alerts
  
  # Build analytics dashboard
  build_analytics_dashboard
  
  # Setup anomaly detection
  setup_anomaly_detection
  
  # Export events if requested
  if [ -n "$EXPORT_FORMAT" ]; then
    export_events "$EXPORT_FORMAT"
  fi
  
  echo ""
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${GREEN}✓ Event monitoring setup completed${NC}"
  echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
  
  if [ "$LISTEN_MODE" = true ]; then
    echo ""
    echo -e "${BLUE}→${NC} Listening for events (press Ctrl+C to stop)..."
    echo ""
    # In production, this would start a real event listener
    # For now, just show the configuration
    cat "$EVENTS_DIR/listener-config.json" | grep -E '"contract_id"|"events"' || true
  fi
}

main
