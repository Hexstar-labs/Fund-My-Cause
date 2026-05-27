/// Storage management and constants for the crowdfund contract.
///
/// This module provides storage keys and helper utilities for managing contract state.
use soroban_sdk::Symbol;

/// Contract version for upgrades and compatibility tracking
pub const CONTRACT_VERSION: u32 = 4;

/// Maximum number of updates per campaign
pub const MAX_UPDATES: u32 = 100;

/// Maximum number of milestones per campaign
pub const MAX_MILESTONES: u32 = 20;

// ── Storage Keys ──────────────────────────────────────────────────────────────
/// Storage key for campaign creator address
pub const KEY_CREATOR: Symbol = soroban_sdk::symbol_short!("CREATOR");
/// Storage key for contribution token address
pub const KEY_TOKEN: Symbol = soroban_sdk::symbol_short!("TOKEN");
/// Storage key for campaign funding goal
pub const KEY_GOAL: Symbol = soroban_sdk::symbol_short!("GOAL");
/// Storage key for campaign deadline timestamp
pub const KEY_DEADLINE: Symbol = soroban_sdk::symbol_short!("DEADLINE");
/// Storage key for total amount raised
pub const KEY_TOTAL: Symbol = soroban_sdk::symbol_short!("TOTAL");
/// Storage key for list of contributors
pub const KEY_CONTRIBS: Symbol = soroban_sdk::symbol_short!("CONTRIBS");
/// Storage key for campaign status
pub const KEY_STATUS: Symbol = soroban_sdk::symbol_short!("STATUS");
/// Storage key for minimum contribution amount
pub const KEY_MIN: Symbol = soroban_sdk::symbol_short!("MIN");
/// Storage key for maximum contribution amount per contributor (0 = no limit)
pub const KEY_MAX: Symbol = soroban_sdk::symbol_short!("MAX");
/// Storage key for campaign title
pub const KEY_TITLE: Symbol = soroban_sdk::symbol_short!("TITLE");
/// Storage key for campaign description
pub const KEY_DESC: Symbol = soroban_sdk::symbol_short!("DESC");
/// Storage key for campaign social links
pub const KEY_SOCIAL: Symbol = soroban_sdk::symbol_short!("SOCIAL");
/// Storage key for platform fee configuration
pub const KEY_PLATFORM: Symbol = soroban_sdk::symbol_short!("PLATFORM");
/// Storage key for contract administrator
pub const KEY_ADMIN: Symbol = soroban_sdk::symbol_short!("ADMIN");
/// Storage key for rate limit configuration (max amount per hour)
pub const KEY_RATE_LIMIT: Symbol = soroban_sdk::symbol_short!("RATELIMIT");
/// Storage key for insurance pool configuration
pub const KEY_INSURANCE: Symbol = soroban_sdk::symbol_short!("INSURE");
/// Storage key for total insurance fees collected
pub const KEY_INSURANCE_POOL: Symbol = soroban_sdk::symbol_short!("INSPOOL");
/// Storage key for campaign category
pub const KEY_CATEGORY: Symbol = soroban_sdk::symbol_short!("CATEGORY");
/// Storage key for vesting schedule
pub const KEY_VESTING: Symbol = soroban_sdk::symbol_short!("VESTING");
/// Storage key for goal adjustment history
pub const KEY_GOAL_HISTORY: Symbol = soroban_sdk::symbol_short!("GHIST");
/// Storage key for campaign visibility level
pub const KEY_VISIBILITY: Symbol = soroban_sdk::symbol_short!("VIS");
