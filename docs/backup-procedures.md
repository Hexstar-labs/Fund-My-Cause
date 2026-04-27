# Contract State Backup and Recovery Guide

This guide explains how to set up, manage, and restore contract state backups for the Fund-My-Cause crowdfunding contract.

## Overview

The backup system provides:
- **Automated backups** via cron scheduling
- **Multiple backup locations** for redundancy
- **Integrity verification** using SHA-256 checksums
- **Easy restoration** from backup files
- **Backup retention policies** to manage disk space

## Quick Start

### 1. Set Up Automated Backups

```bash
# Daily backups
./scripts/setup-backup-automation.sh <CONTRACT_ID> daily

# Hourly backups
./scripts/setup-backup-automation.sh <CONTRACT_ID> hourly

# Weekly backups
./scripts/setup-backup-automation.sh <CONTRACT_ID> weekly
```

### 2. Manual Backup

```bash
./scripts/backup-contract-state.sh <CONTRACT_ID> [backup_dir]
```

### 3. Verify Backups

```bash
./scripts/verify-backups.sh .backups
```

### 4. Restore from Backup

```bash
./scripts/restore-contract-state.sh <backup_file> <CONTRACT_ID>
```

## Backup File Structure

Each backup file contains:
- **backup_timestamp**: When the backup was created
- **contract_id**: The contract being backed up
- **network**: The Stellar network (testnet/mainnet)
- **rpc_url**: The RPC endpoint used
- **backup_version**: Format version for compatibility
- **state_data**: The exported contract state

Example:
```json
{
  "backup_timestamp": "2026-04-27T14:58:19Z",
  "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
  "network": "testnet",
  "rpc_url": "https://soroban-testnet.stellar.org",
  "backup_version": "1.0",
  "state_data": {
    "exported_at": "2026-04-27T14:58:19Z"
  }
}
```

## Backup Locations

### Primary Location
- **Path**: `.backups/`
- **Retention**: 30 days
- **Frequency**: Configurable (hourly/daily/weekly)

### Secondary Location (Optional)
Set the `SECONDARY_BACKUP_DIR` environment variable to enable:

```bash
export SECONDARY_BACKUP_DIR="/mnt/backup-storage"
./scripts/backup-contract-state.sh <CONTRACT_ID>
```

### Cloud Storage (Optional)
For production, consider syncing backups to cloud storage:

```bash
# AWS S3
aws s3 sync .backups/ s3://my-backup-bucket/contract-backups/

# Google Cloud Storage
gsutil -m cp -r .backups/* gs://my-backup-bucket/contract-backups/

# Azure Blob Storage
az storage blob upload-batch -d backups -s .backups/
```

## Verification

### Check Backup Integrity

```bash
# Verify all backups
./scripts/verify-backups.sh .backups

# Verify specific backup
sha256sum -c contract_state_20260427_145819.json.sha256
```

### Monitor Backup Logs

```bash
# View backup logs
tail -f .backups/backup.log

# Check cron execution
grep CRON /var/log/syslog  # Linux
log stream --predicate 'process == "cron"'  # macOS
```

## Restoration Procedures

### Full State Restoration

```bash
# 1. Identify the backup to restore
ls -lh .backups/contract_state_*.json

# 2. Restore the state
./scripts/restore-contract-state.sh .backups/contract_state_20260427_145819.json <CONTRACT_ID>

# 3. Verify restoration
./scripts/verify-backups.sh .backups
```

### Partial State Recovery

For recovering specific data (e.g., contributor list):

```bash
# Extract specific data from backup
jq '.state_data.contributors' .backups/contract_state_20260427_145819.json
```

## Backup Schedule Recommendations

### Development
- **Frequency**: Daily
- **Retention**: 7 days
- **Locations**: 1 (local)

### Staging
- **Frequency**: Daily
- **Retention**: 30 days
- **Locations**: 2 (local + secondary)

### Production
- **Frequency**: Hourly
- **Retention**: 90 days
- **Locations**: 3+ (local + multiple cloud regions)

## Environment Variables

Configure backup behavior with environment variables:

```bash
# Contract to backup
export CONTRACT_ID="CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4"

# Backup directory
export BACKUP_DIR=".backups"

# Soroban RPC endpoint
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"

# Stellar network
export STELLAR_NETWORK="testnet"

# Secondary backup location
export SECONDARY_BACKUP_DIR="/mnt/backup-storage"
```

## Troubleshooting

### Backup Fails with "Stellar CLI not found"

Install the Stellar CLI:
```bash
# macOS
brew install stellar-cli

# Linux
curl -L https://github.com/stellar/stellar-cli/releases/download/v21.0.0/stellar-cli-21.0.0-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv stellar /usr/local/bin/

# Windows
# Download from: https://github.com/stellar/stellar-cli/releases
```

### Backup Integrity Check Fails

```bash
# Regenerate checksum
sha256sum contract_state_*.json > contract_state_*.json.sha256

# Or delete corrupted backup and create new one
rm contract_state_*.json*
./scripts/backup-contract-state.sh <CONTRACT_ID>
```

### Cron Job Not Running

```bash
# Check cron is running
sudo service cron status  # Linux
sudo launchctl list | grep cron  # macOS

# Verify cron job exists
crontab -l

# Check cron logs
sudo tail -f /var/log/syslog | grep CRON  # Linux
log stream --predicate 'process == "cron"'  # macOS
```

### Restoration Fails

```bash
# Verify backup file integrity
sha256sum -c contract_state_*.json.sha256

# Check contract ID matches
jq '.contract_id' contract_state_*.json

# Verify network matches
jq '.network' contract_state_*.json
```

## Best Practices

1. **Test Restores Regularly**: Verify backups work by testing restoration in a test environment
2. **Monitor Backup Logs**: Set up alerts for backup failures
3. **Use Multiple Locations**: Store backups in at least 2 different locations
4. **Encrypt Sensitive Data**: Use encryption for backups containing sensitive information
5. **Document Procedures**: Keep restoration procedures documented and accessible
6. **Automate Verification**: Run verification scripts regularly
7. **Maintain Audit Trail**: Log all backup and restoration operations

## Disaster Recovery Plan

### RTO (Recovery Time Objective): 1 hour
### RPO (Recovery Point Objective): 1 hour

**Steps**:
1. Identify the issue and determine recovery point
2. Locate the appropriate backup file
3. Verify backup integrity
4. Restore state to contract
5. Verify restoration success
6. Monitor contract operation
7. Document incident and recovery

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review backup logs in `.backups/backup.log`
3. Verify Stellar CLI installation and configuration
4. Check environment variables are set correctly
