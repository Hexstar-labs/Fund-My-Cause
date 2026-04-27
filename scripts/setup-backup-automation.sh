#!/bin/bash

# Automated Backup Scheduler
# Sets up automated contract state backups using cron
# Usage: ./setup-backup-automation.sh <contract_id> [frequency]

set -e

CONTRACT_ID="${1:-}"
FREQUENCY="${2:-daily}"  # daily, weekly, or hourly
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_DIR="${SCRIPT_DIR}/../.backups"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Validate inputs
if [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Error: Contract ID is required${NC}"
    echo "Usage: $0 <contract_id> [frequency]"
    echo "Frequency options: hourly, daily, weekly"
    exit 1
fi

# Validate frequency
case "$FREQUENCY" in
    hourly|daily|weekly)
        ;;
    *)
        echo -e "${RED}Error: Invalid frequency. Use: hourly, daily, or weekly${NC}"
        exit 1
        ;;
esac

echo -e "${YELLOW}Setting up automated backup for contract: $CONTRACT_ID${NC}"
echo "Frequency: $FREQUENCY"
echo "Backup directory: $BACKUP_DIR"

# Make scripts executable
chmod +x "${SCRIPT_DIR}/backup-contract-state.sh"
chmod +x "${SCRIPT_DIR}/restore-contract-state.sh"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Determine cron schedule
case "$FREQUENCY" in
    hourly)
        CRON_SCHEDULE="0 * * * *"
        DESCRIPTION="every hour"
        ;;
    daily)
        CRON_SCHEDULE="0 2 * * *"
        DESCRIPTION="daily at 2 AM"
        ;;
    weekly)
        CRON_SCHEDULE="0 2 * * 0"
        DESCRIPTION="weekly on Sunday at 2 AM"
        ;;
esac

# Create cron job entry
CRON_JOB="$CRON_SCHEDULE cd $SCRIPT_DIR && ./backup-contract-state.sh $CONTRACT_ID $BACKUP_DIR >> $BACKUP_DIR/backup.log 2>&1"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "backup-contract-state.sh.*$CONTRACT_ID"; then
    echo -e "${YELLOW}Cron job already exists for this contract${NC}"
    read -p "Replace existing cron job? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Setup cancelled${NC}"
        exit 0
    fi
    # Remove existing job
    crontab -l | grep -v "backup-contract-state.sh.*$CONTRACT_ID" | crontab -
fi

# Add new cron job
(crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -

echo -e "${GREEN}✓ Cron job created${NC}"
echo -e "${GREEN}✓ Backups will run $DESCRIPTION${NC}"

# Create environment file for cron
ENV_FILE="${SCRIPT_DIR}/.backup-env"
cat > "$ENV_FILE" << EOF
# Backup Environment Configuration
export CONTRACT_ID="$CONTRACT_ID"
export BACKUP_DIR="$BACKUP_DIR"
export SOROBAN_RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
export STELLAR_NETWORK="${STELLAR_NETWORK:-testnet}"
export SECONDARY_BACKUP_DIR="${SECONDARY_BACKUP_DIR:-}"
EOF

echo -e "${GREEN}✓ Environment file created: $ENV_FILE${NC}"

# Create backup verification script
VERIFY_SCRIPT="${SCRIPT_DIR}/verify-backups.sh"
cat > "$VERIFY_SCRIPT" << 'VERIFY_EOF'
#!/bin/bash

# Backup Verification Script
# Verifies the integrity of all backups

BACKUP_DIR="${1:-.backups}"

if [ ! -d "$BACKUP_DIR" ]; then
    echo "Backup directory not found: $BACKUP_DIR"
    exit 1
fi

echo "Verifying backups in: $BACKUP_DIR"
echo ""

TOTAL=0
VALID=0
INVALID=0

for backup in "$BACKUP_DIR"/contract_state_*.json; do
    if [ -f "$backup" ]; then
        TOTAL=$((TOTAL + 1))
        CHECKSUM_FILE="${backup}.sha256"
        
        if [ -f "$CHECKSUM_FILE" ]; then
            if sha256sum -c "$CHECKSUM_FILE" > /dev/null 2>&1; then
                echo "✓ $(basename "$backup")"
                VALID=$((VALID + 1))
            else
                echo "✗ $(basename "$backup") - CORRUPTED"
                INVALID=$((INVALID + 1))
            fi
        else
            echo "? $(basename "$backup") - No checksum"
        fi
    fi
done

echo ""
echo "Summary: $TOTAL total, $VALID valid, $INVALID invalid"

if [ $INVALID -gt 0 ]; then
    exit 1
fi
VERIFY_EOF

chmod +x "$VERIFY_SCRIPT"
echo -e "${GREEN}✓ Verification script created: $VERIFY_SCRIPT${NC}"

# Display cron job list
echo ""
echo -e "${YELLOW}Current cron jobs:${NC}"
crontab -l | grep -E "backup-contract-state|restore-contract-state" || echo "No backup jobs found"

echo ""
echo -e "${GREEN}Automated backup setup completed!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Verify the cron job: crontab -l"
echo "2. Check backup logs: tail -f $BACKUP_DIR/backup.log"
echo "3. Verify backups: $VERIFY_SCRIPT $BACKUP_DIR"
echo "4. To restore: $SCRIPT_DIR/restore-contract-state.sh <backup_file> <contract_id>"
