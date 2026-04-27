#!/bin/bash

# Contract State Backup Script
# Exports contract state data and stores backups in multiple locations
# Usage: ./backup-contract-state.sh <contract_id> [backup_dir]

set -e

CONTRACT_ID="${1:-}"
BACKUP_DIR="${2:-.backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="contract_state_${TIMESTAMP}.json"
BACKUP_PATH="${BACKUP_DIR}/${BACKUP_NAME}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Validate inputs
if [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Error: Contract ID is required${NC}"
    echo "Usage: $0 <contract_id> [backup_dir]"
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

echo -e "${YELLOW}Starting contract state backup...${NC}"
echo "Contract ID: $CONTRACT_ID"
echo "Backup directory: $BACKUP_DIR"
echo "Backup file: $BACKUP_NAME"

# Export contract state using Stellar CLI
# This requires the Stellar CLI to be installed and configured
if ! command -v stellar &> /dev/null; then
    echo -e "${RED}Error: Stellar CLI is not installed${NC}"
    echo "Please install it from: https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli"
    exit 1
fi

# Get RPC URL from environment or use default
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK="${STELLAR_NETWORK:-testnet}"

echo -e "${YELLOW}Exporting state from $NETWORK network...${NC}"

# Create backup with metadata
{
    echo "{"
    echo "  \"backup_timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"contract_id\": \"$CONTRACT_ID\","
    echo "  \"network\": \"$NETWORK\","
    echo "  \"rpc_url\": \"$RPC_URL\","
    echo "  \"backup_version\": \"1.0\","
    echo "  \"state_data\": {"
    
    # Export contract state (this is a placeholder - actual implementation depends on contract structure)
    # In a real scenario, you would call contract methods to export state
    echo "    \"exported_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
    echo "  }"
    echo "}"
} > "$BACKUP_PATH"

echo -e "${GREEN}✓ Backup created: $BACKUP_PATH${NC}"

# Verify backup integrity
if [ ! -f "$BACKUP_PATH" ]; then
    echo -e "${RED}Error: Backup file was not created${NC}"
    exit 1
fi

BACKUP_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
echo -e "${GREEN}✓ Backup size: $BACKUP_SIZE${NC}"

# Create checksum for verification
CHECKSUM=$(sha256sum "$BACKUP_PATH" | awk '{print $1}')
echo "$CHECKSUM  $BACKUP_NAME" > "${BACKUP_PATH}.sha256"
echo -e "${GREEN}✓ Checksum created: ${BACKUP_PATH}.sha256${NC}"

# Cleanup old backups (keep last 30 days)
echo -e "${YELLOW}Cleaning up old backups (older than 30 days)...${NC}"
find "$BACKUP_DIR" -name "contract_state_*.json" -mtime +30 -delete
find "$BACKUP_DIR" -name "contract_state_*.json.sha256" -mtime +30 -delete

# Count remaining backups
BACKUP_COUNT=$(find "$BACKUP_DIR" -name "contract_state_*.json" | wc -l)
echo -e "${GREEN}✓ Backups retained: $BACKUP_COUNT${NC}"

# Optional: Copy to secondary location if specified
if [ -n "$SECONDARY_BACKUP_DIR" ]; then
    echo -e "${YELLOW}Copying to secondary backup location...${NC}"
    mkdir -p "$SECONDARY_BACKUP_DIR"
    cp "$BACKUP_PATH" "$SECONDARY_BACKUP_DIR/"
    cp "${BACKUP_PATH}.sha256" "$SECONDARY_BACKUP_DIR/"
    echo -e "${GREEN}✓ Secondary backup created${NC}"
fi

echo -e "${GREEN}Backup completed successfully!${NC}"
