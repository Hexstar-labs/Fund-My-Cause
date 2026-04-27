# Contract Event Monitoring

This document covers event monitoring, indexing, and analytics for Fund-My-Cause smart contracts.

## Overview

Event monitoring provides:
- Real-time visibility into contract operations
- Event-based alerting for important actions
- Analytics dashboard for campaign metrics
- Anomaly detection for unusual patterns
- Audit trail for compliance

## Contract Events

### Event Types

| Event | Trigger | Data |
|-------|---------|------|
| `campaign:initialized` | New campaign created | creator, goal, deadline |
| `contribution:made` | User pledges funds | contributor, amount, campaign |
| `funds:withdrawn` | Creator claims funds | creator, amount, campaign |
| `refund:claimed` | Contributor claims refund | contributor, amount, campaign |
| `metadata:updated` | Campaign info updated | title, description, social_links |

### Event Structure

```json
{
  "timestamp": "2026-04-27T14:46:26Z",
  "event_type": "contribution:made",
  "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
  "data": {
    "contributor": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
    "amount": "1000000000",
    "campaign_id": "1"
  },
  "transaction_hash": "abc123...",
  "ledger_sequence": 12345
}
```

## Event Listener Setup

### Configuration

The event listener is configured in `target/events/listener-config.json`:

```json
{
  "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
  "rpc_url": "https://soroban-testnet.stellar.org",
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
```

### Starting the Listener

```bash
./scripts/monitor-events.sh --listen
```

The listener will:
1. Connect to the Soroban RPC endpoint
2. Poll for new events every 5 seconds
3. Index events in `target/events/events.json`
4. Log events to `target/events/events.log`
5. Trigger alerts based on configured rules

## Event-Based Alerts

### Alert Configuration

Alerts are defined in `target/events/alerts.json`:

```json
{
  "alerts": [
    {
      "id": "high_contribution",
      "name": "High Contribution Alert",
      "trigger": "contribution:made",
      "condition": "amount > 10000",
      "action": "notify",
      "channels": ["email", "slack"]
    }
  ]
}
```

### Alert Types

| Alert | Trigger | Condition | Action |
|-------|---------|-----------|--------|
| High Contribution | `contribution:made` | amount > 10,000 | Email + Slack |
| Goal Reached | `funds:withdrawn` | total_raised >= goal | Email + Webhook |
| New Campaign | `campaign:initialized` | Always | Log to analytics |
| Refund Claimed | `refund:claimed` | Always | Log to analytics |

### Notification Channels

- **Email**: Send to configured email addresses
- **Slack**: Post to Slack channel
- **Webhook**: POST to external webhook URL
- **Analytics**: Log to analytics service

## Event Analytics Dashboard

### Dashboard Features

The analytics dashboard (`target/events/dashboard.html`) displays:

- **Total Events**: All-time event count
- **Campaigns Created**: Number of `campaign:initialized` events
- **Contributions**: Number of `contribution:made` events
- **Withdrawals**: Number of `funds:withdrawn` events
- **Recent Events**: Live event feed

### Accessing the Dashboard

```bash
# Open in browser
open target/events/dashboard.html

# Or serve via HTTP
python3 -m http.server --directory target/events 8000
# Visit http://localhost:8000/dashboard.html
```

### Dashboard Metrics

| Metric | Description | Update Frequency |
|--------|-------------|-------------------|
| Total Events | All events since monitoring started | Real-time |
| Campaign Count | Number of campaigns created | Real-time |
| Contribution Count | Number of contributions | Real-time |
| Withdrawal Count | Number of successful withdrawals | Real-time |
| Refund Count | Number of refunds claimed | Real-time |

## Anomaly Detection

### Anomaly Types

| Anomaly | Metric | Baseline | Threshold | Severity |
|---------|--------|----------|-----------|----------|
| Contribution Spike | contributions/min | 5 | 20 | Warning |
| Rapid Refunds | refunds/min | 2 | 10 | Critical |
| High Failure Rate | failure_rate | 1% | 10% | Critical |

### Detection Configuration

Anomalies are configured in `target/events/anomaly-detection.json`:

```json
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
    }
  ],
  "detection_interval_seconds": 60,
  "notification_channels": ["email", "slack", "webhook"]
}
```

### Anomaly Response

When an anomaly is detected:

1. **Alert is triggered** with severity level
2. **Notifications sent** to configured channels
3. **Event is logged** for investigation
4. **Dashboard updated** with anomaly indicator
5. **Incident created** if severity is critical

## Event Indexing

### Index Structure

Events are indexed in `target/events/events.json`:

```json
{
  "events": [
    {
      "id": "evt_001",
      "timestamp": "2026-04-27T14:46:26Z",
      "type": "contribution:made",
      "contract_id": "CAAAA...",
      "data": { ... }
    }
  ],
  "summary": {
    "total_events": 1234,
    "by_type": {
      "campaign:initialized": 45,
      "contribution:made": 1000,
      "funds:withdrawn": 150,
      "refund:claimed": 39
    },
    "by_contract": { ... },
    "last_updated": "2026-04-27T14:46:26Z"
  }
}
```

### Querying Events

```bash
# List all events
cat target/events/events.json | jq '.events'

# Filter by type
cat target/events/events.json | jq '.events[] | select(.type == "contribution:made")'

# Get summary
cat target/events/events.json | jq '.summary'
```

## Event Export

### Export Formats

```bash
# Export as JSON
./scripts/monitor-events.sh --export json

# Export as CSV
./scripts/monitor-events.sh --export csv

# Export as HTML
./scripts/monitor-events.sh --export html
```

### Export Files

- `events-export-20260427-144626.json` — JSON format
- `events-export-20260427-144626.csv` — CSV format
- `events-export-20260427-144626.html` — HTML table

### Using Exported Data

```bash
# Analyze with jq
cat events-export-*.json | jq '.summary'

# Import to database
sqlite3 events.db < events-export-*.csv

# Generate reports
python3 analyze_events.py events-export-*.json
```

## Automated Monitoring

### CI/CD Integration

The `event-monitoring.yml` workflow runs every 5 minutes:

```yaml
schedule:
  - cron: '*/5 * * * *'
```

### Workflow Steps

1. Set up event monitoring
2. Check for new events
3. Detect anomalies
4. Upload event reports
5. Generate summary

### Accessing Reports

Reports are available as GitHub Actions artifacts:

1. Go to **Actions → Contract Event Monitoring**
2. Click the latest run
3. Download **event-reports-{run-id}** artifact

## Best Practices

1. **Monitor continuously** — Run event listener 24/7
2. **Set realistic thresholds** — Based on normal traffic patterns
3. **Alert early** — Catch issues before they escalate
4. **Review regularly** — Check dashboard daily
5. **Archive events** — Keep historical data for analysis
6. **Test alerts** — Verify notification channels work
7. **Document changes** — Log all configuration updates

## Troubleshooting

### No Events Detected

1. Verify contract ID is correct
2. Check RPC URL is accessible
3. Ensure contract has been deployed
4. Check network connectivity

### High False Positive Rate

1. Review and adjust thresholds
2. Analyze baseline metrics
3. Consider time-based patterns
4. Implement machine learning models

### Missing Events

1. Check event listener logs
2. Verify polling interval
3. Review batch size settings
4. Check for RPC rate limits

## References

- [Stellar Events](https://developers.stellar.org/docs/data/horizon/api-reference/resources/events)
- [Soroban Events](https://soroban.stellar.org/docs/learn/events)
- [Horizon Event Streaming](https://developers.stellar.org/docs/data/horizon/api-reference/streaming)
- [Event-Driven Architecture](https://martinfowler.com/articles/201701-event-driven.html)
