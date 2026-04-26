/// Data types and structures for the crowdfund contract.
///
/// This module contains all `#[contracttype]` definitions including enums and structs
/// used throughout the contract for state management and function signatures.

use soroban_sdk::{contracttype, Address, String, Vec};

/// Campaign status enumeration.
///
/// Represents the lifecycle state of a crowdfunding campaign.
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum Status {
    /// Campaign is accepting contributions
    Active,
    /// Campaign deadline passed and goal was reached
    Successful,
    /// Campaign deadline passed and goal was not reached (refunds available)
    Refunded,
    /// Campaign was cancelled by creator (refunds available)
    Cancelled,
    /// Campaign is temporarily paused (no new contributions allowed)
    Paused,
}

/// Campaign statistics snapshot.
///
/// Contains aggregated metrics about campaign progress and contributor activity.
#[derive(Clone)]
#[contracttype]
pub struct CampaignStats {
    /// Total amount raised in stroops
    pub total_raised: i128,
    /// Campaign funding goal in stroops
    pub goal: i128,
    /// Progress as basis points (0-10000, where 10000 = 100%)
    pub progress_bps: u32,
    /// Number of unique contributors
    pub contributor_count: u32,
    /// Average contribution amount in stroops (total_raised / contributor_count)
    pub average_contribution: i128,
    /// Largest single contribution amount in stroops
    pub largest_contribution: i128,
}

/// Platform fee configuration.
///
/// Specifies the address that receives platform fees and the fee percentage.
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub struct PlatformConfig {
    /// Address that receives platform fees
    pub address: Address,
    /// Fee percentage in basis points (e.g., 250 = 2.5%)
    pub fee_bps: u32,
}

/// Complete campaign information.
///
/// Contains all metadata and configuration for a campaign.
#[derive(Clone)]
#[contracttype]
pub struct CampaignInfo {
    /// Campaign creator's Stellar address
    pub creator: Address,
    /// Token address for contributions
    pub token: Address,
    /// Funding goal in stroops
    pub goal: i128,
    /// Campaign deadline as Unix timestamp (seconds)
    pub deadline: u64,
    /// Minimum contribution amount in stroops
    pub min_contribution: i128,
    /// Maximum contribution amount per contributor in stroops (0 = no limit)
    pub max_contribution: i128,
    /// Campaign title
    pub title: String,
    /// Campaign description
    pub description: String,
    /// Current campaign status
    pub status: Status,
    /// Whether a platform fee is configured
    pub has_platform_config: bool,
    /// Platform fee in basis points (0 if no config)
    pub platform_fee_bps: u32,
    /// Platform fee recipient address
    pub platform_address: Address,
    /// Campaign category
    pub category: Category,
}

/// Campaign update entry with IPFS hash and timestamp.
#[derive(Clone)]
#[contracttype]
pub struct CampaignUpdate {
    /// IPFS hash of the update content
    pub ipfs_hash: String,
    /// Timestamp when update was posted
    pub timestamp: u64,
}

/// Milestone tracking for campaigns.
#[derive(Clone)]
#[contracttype]
pub struct Milestone {
    /// Target amount in stroops
    pub amount: i128,
    /// Milestone description
    pub description: String,
    /// Whether this milestone has been reached
    pub reached: bool,
}

/// Matching configuration for sponsor contributions.
#[derive(Clone)]
#[contracttype]
pub struct MatchingConfig {
    /// Sponsor address providing matching funds
    pub sponsor: Address,
    /// Match ratio in basis points (e.g., 10000 = 1:1 match)
    pub match_ratio: u32,
    /// Maximum total matching amount in stroops
    pub max_match: i128,
}

/// Campaign template type.
#[derive(Clone, Copy, PartialEq, Debug)]
#[contracttype]
pub enum TemplateType {
    /// Charity/nonprofit fundraising
    Charity,
    /// Product launch or development
    Product,
    /// Event or conference
    Event,
    /// Personal cause
    Personal,
    /// Custom template
    Custom,
}

/// Campaign category.
#[derive(Clone, Copy, PartialEq, Debug)]
#[contracttype]
pub enum Category {
    /// Charity/nonprofit
    Charity,
    /// Technology
    Technology,
    /// Creative
    Creative,
    /// Event
    Event,
    /// Personal
    Personal,
    /// Other
    Other,
}

/// Campaign template configuration.
#[derive(Clone)]
#[contracttype]
pub struct CampaignTemplate {
    /// Template type
    pub template_type: TemplateType,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Suggested minimum contribution
    pub suggested_min: i128,
    /// Suggested goal multiplier (e.g., 10000 = 1x)
    pub goal_multiplier: u32,
}

/// Delegation configuration.
#[derive(Clone)]
#[contracttype]
pub struct Delegation {
    /// Delegated amount in stroops
    pub amount: i128,
    /// Delegate address
    pub delegate: Address,
    /// Whether delegation is active
    pub active: bool,
}

/// Storage key variants for contract data.
///
/// Used to organize persistent and instance storage in the contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Contribution amount for a specific address
    Contribution(Address),
    /// Whether an address has contributed (presence flag)
    ContributorPresence(Address),
    /// Total number of unique contributors
    ContributorCount,
    /// Largest single contribution amount
    LargestContribution,
    /// Whitelist of accepted token addresses
    AcceptedTokens,
    /// Contribution message for a specific address
    ContributionMessage(Address),
    /// Recurring contribution plan for a specific address
    RecurringPlan(Address),
    /// Recurring contribution history for a specific address
    RecurringHistory(Address),
    /// Extension proposal data
    ExtensionProposal,
    /// Extension votes for a specific address
    ExtensionVote(Address),
    /// Partial refund amount for a specific address
    PartialRefund(Address),
    /// Insurance fee paid by a specific contributor
    InsuranceFee(Address),
}

/// Recurring contribution plan.
///
/// Defines a scheduled recurring contribution.
#[derive(Clone)]
#[contracttype]
pub struct RecurringPlan {
    /// Amount to contribute each interval in stroops
    pub amount: i128,
    /// Interval in seconds between contributions
    pub interval: u64,
    /// End date for recurring contributions (Unix timestamp)
    pub end_date: u64,
    /// Timestamp of last execution
    pub last_executed: u64,
}

/// Extension proposal for deadline voting.
///
/// Tracks a proposed deadline extension and voting results.
#[derive(Clone)]
#[contracttype]
pub struct ExtensionProposal {
    /// Proposed new deadline (Unix timestamp)
    pub new_deadline: u64,
    /// Total votes in favor
    pub votes_for: i128,
    /// Total votes against
    pub votes_against: i128,
    /// Proposal creation timestamp
    pub created_at: u64,
    /// Voting period end timestamp
    pub voting_ends_at: u64,
    /// Whether the proposal has been executed
    pub executed: bool,
}

/// Insurance configuration for campaign protection.
///
/// Defines optional insurance parameters for contributor protection.
#[derive(Clone)]
#[contracttype]
pub struct InsuranceConfig {
    /// Insurance fee in basis points (e.g., 100 = 1%)
    pub fee_bps: u32,
    /// Insurance provider address
    pub provider: Address,
    /// Whether insurance is enabled for this campaign
    pub enabled: bool,
}
