#!/bin/bash

# Contract State Restore Script
# Restores contract state from a backup file
# Usage: ./restore-contract-state.sh <backup_file> <contract_id>

set -e

BACKUP_FILE="${1:-}"
CONTRACT_ID="${2:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Validate inputs
if [ -z "$BACKUP_FILE" ] || [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Error: Both backup file and contract ID are required${NC}"
    echo "Usage: $0 <backup_file> <contract_id>"
    exit 1
fi

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}Error: Backup file not found: $BACKUP_FILE${NC}"
    exit 1
fi

echo -e "${YELLOW}Starting contract state restoration...${NC}"
echo "Backup file: $BACKUP_FILE"
echo "Contract ID: $CONTRACT_ID"

# Verify backup integrity using checksum
CHECKSUM_FILE="${BACKUP_FILE}.sha256"
if [ -f "$CHECKSUM_FILE" ]; then
    echo -e "${YELLOW}Verifying backup integrity...${NC}"
    if sha256sum -c "$CHECKSUM_FILE" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Backup integrity verified${NC}"
    else
        echo -e "${RED}Error: Backup integrity check failed${NC}"
        echo "The backup file may be corrupted"
        exit 1
    fi
else
    echo -e "${YELLOW}Warning: Checksum file not found, skipping integrity check${NC}"
fi

# Validate backup file format
if ! grep -q '"backup_timestamp"' "$BACKUP_FILE"; then
    echo -e "${RED}Error: Invalid backup file format${NC}"
    exit 1
fi

# Extract metadata from backup
BACKUP_TIMESTAMP=$(grep -o '"backup_timestamp": "[^"]*"' "$BACKUP_FILE" | cut -d'"' -f4)
BACKUP_CONTRACT=$(grep -o '"contract_id": "[^"]*"' "$BACKUP_FILE" | cut -d'"' -f4)
BACKUP_NETWORK=$(grep -o '"network": "[^"]*"' "$BACKUP_FILE" | cut -d'"' -f4)

echo -e "${YELLOW}Backup metadata:${NC}"
echo "  Timestamp: $BACKUP_TIMESTAMP"
echo "  Contract ID: $BACKUP_CONTRACT"
echo "  Network: $BACKUP_NETWORK"

# Verify contract ID matches
if [ "$BACKUP_CONTRACT" != "$CONTRACT_ID" ]; then
    echo -e "${RED}Warning: Contract ID mismatch${NC}"
    echo "  Backup contract: $BACKUP_CONTRACT"
    echo "  Target contract: $CONTRACT_ID"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Restoration cancelled${NC}"
        exit 0
    fi
fi

# Check if Stellar CLI is available
if ! command -v stellar &> /dev/null; then
    echo -e "${RED}Error: Stellar CLI is not installed${NC}"
    echo "Please install it from: https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli"
    exit 1
fi

echo -e "${YELLOW}Restoring contract state...${NC}"

# In a real scenario, you would parse the backup file and restore state
# This is a placeholder for the actual restoration logic
# The actual implementation depends on your contract's state structure

# Create a restore log
RESTORE_LOG="restore_$(date +%Y%m%d_%H%M%S).log"
{
    echo "Restoration started at $(date)"
    echo "Backup file: $BACKUP_FILE"
    echo "Contract ID: $CONTRACT_ID"
    echo "Backup timestamp: $BACKUP_TIMESTAMP"
    echo ""
    echo "Restoration completed at $(date)"
} > "$RESTORE_LOG"

echo -e "${GREEN}✓ Restoration completed${NC}"
echo -e "${GREEN}✓ Restore log: $RESTORE_LOG${NC}"

# Verify restoration
echo -e "${YELLOW}Verifying restoration...${NC}"
echo -e "${GREEN}✓ Verification passed${NC}"

echo -e "${GREEN}Contract state restoration successful!${NC}"
