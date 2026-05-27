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

/// Per-address rate limit configuration for contributions.
///
/// Limits the total amount a single address can contribute within a configurable
/// time window. The window is tracked per-address from the moment of the first
/// contribution in the window; once `window_seconds` have elapsed without a new
/// contribution, the period resets.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct RateLimit {
    /// Maximum total contribution amount per address within `window_seconds`.
    pub max_amount: i128,
    /// Length of the per-address window in seconds.
    pub window_seconds: u64,
}

/// Campaign visibility level.
///
/// Controls who can contribute and whether the campaign is publicly discoverable.
/// `Private` campaigns restrict contributions to whitelisted addresses; `Public`
/// and `Unlisted` campaigns place no extra access restriction here, but `Unlisted`
/// signals to frontends that the campaign should not appear in discovery feeds.
#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum Visibility {
    /// Listed publicly; anyone may contribute.
    Public,
    /// Whitelist-only contributions; not listed in discovery.
    Private,
    /// Anyone may contribute; not listed in discovery.
    Unlisted,
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

/// Reward tier for contribution amounts.
///
/// Defines a named reward tier that contributors reach based on their total
/// cumulative contribution.  Tiers should be stored sorted by `min_amount`
/// in ascending order.
///
/// # Example
/// ```ignore
/// // Bronze: ≥ 100 stroops, Silver: ≥ 1_000, Gold: ≥ 10_000
/// ```
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub struct RewardTier {
    /// Minimum cumulative contribution required to qualify (in stroops)
    pub min_amount: i128,
    /// Short display name for the tier (e.g. "Bronze", "Silver", "Gold")
    pub name: String,
    /// Human-readable description of what this tier unlocks
    pub description: String,
}

/// An immutable record of a single contribution.
///
/// Appended to each contributor's persistent history every time they call
/// [`contribute`](crate::CrowdfundContract::contribute).
#[derive(Clone)]
#[contracttype]
pub struct ContributionRecord {
    /// Amount transferred in this contribution (in stroops)
    pub amount: i128,
    /// Ledger timestamp at the moment the contribution was accepted
    pub timestamp: u64,
    /// Contributor's cumulative total after this contribution
    pub running_total: i128,
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
    /// Whitelist flag for a specific address
    Whitelist(Address),
    /// Blacklist flag for a specific address
    Blacklist(Address),
    /// Whitelist-only mode flag
    WhitelistOnly,
    /// Delegation for a specific address
    Delegation(Address),
    /// Delegated contribution amount for a specific address
    DelegatedContribution(Address),
    /// Campaign template
    Template,
    /// Emergency lock time
    EmergencyLockTime,
    /// Rate limit timestamp for a specific address
    RateLimitTimestamp(Address),
    /// Rate limit amount for a specific address
    RateLimitAmount(Address),
    /// Matching configuration
    MatchingConfig,
    /// Total matched amount
    TotalMatched,
    /// Penalty basis points
    PenaltyBps,
    /// Ordered list of reward tiers configured by the creator
    RewardTiers,
    /// Best reward tier currently assigned to a specific contributor
    ContributorTier(Address),
    /// Full contribution history for a specific contributor
    ContributionHistory(Address),
    /// Required number of approvals for emergency withdrawal multi-sig
    EmergencyApproversRequired,
    /// Running approval count for the active emergency withdrawal session
    EmergencyApprovalCount,
    /// Session token (lock_until timestamp) that a specific address last approved
    EmergencyApproval(Address),
    /// Authorized approver addresses for emergency multi-sig
    EmergencyApproversList,
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

// ── Missing types referenced in lib.rs ───────────────────────────────────────

/// Vesting schedule for campaign withdrawal.
///
/// Defines a cliff-and-duration vesting schedule for creator payouts.
#[derive(Clone)]
#[contracttype]
pub struct VestingSchedule {
    /// Cliff timestamp — no withdrawal before this point (Unix seconds)
    pub cliff: u64,
    /// Duration in seconds over which funds vest linearly after the cliff
    pub duration: u64,
}

/// Records a single goal adjustment for audit history.
#[derive(Clone)]
#[contracttype]
pub struct GoalAdjustment {
    /// Goal value before the adjustment
    pub previous_goal: i128,
    /// Goal value after the adjustment
    pub new_goal: i128,
    /// Ledger timestamp when the adjustment occurred
    pub timestamp: u64,
}

// ── Structured event payloads ─────────────────────────────────────────────────
//
// Each event emitted by the contract carries one of these structs as its data
// payload.  Using typed structs instead of raw tuples makes events indexable,
// self-documenting, and forward-compatible.

/// Emitted when a campaign is successfully initialized.
///
/// Event topic: `("campaign", "initialized")`
#[derive(Clone)]
#[contracttype]
pub struct EventInitialized {
    pub creator: Address,
    pub goal: i128,
    pub deadline: u64,
}

/// Emitted when a contribution is accepted.
///
/// Event topic: `("campaign", "contributed")`
#[derive(Clone)]
#[contracttype]
pub struct EventContributed {
    pub contributor: Address,
    pub amount: i128,
    /// New running total for this contributor after this contribution
    pub new_total: i128,
    /// Matched amount added by a sponsor (0 if no matching configured)
    pub matched_amount: i128,
}

/// Emitted when the creator withdraws funds after a successful campaign.
///
/// Event topic: `("campaign", "withdrawn")`
#[derive(Clone)]
#[contracttype]
pub struct EventWithdrawn {
    pub creator: Address,
    /// Total raised at the time of withdrawal (before fee deduction)
    pub total: i128,
    /// Platform fee deducted (0 if no platform config)
    pub fee: i128,
    /// Net amount transferred to the creator
    pub payout: i128,
}

/// Emitted when a contributor claims a full refund.
///
/// Event topic: `("campaign", "refunded")`
#[derive(Clone)]
#[contracttype]
pub struct EventRefunded {
    pub contributor: Address,
    pub amount: i128,
}

/// Emitted when a contributor claims a partial refund before the deadline.
///
/// Event topic: `("campaign", "partial_refund")`
#[derive(Clone)]
#[contracttype]
pub struct EventPartialRefund {
    pub contributor: Address,
    pub amount: i128,
    /// Remaining contribution balance after the partial refund
    pub remaining: i128,
}

/// Emitted when the campaign status changes.
///
/// Event topic: `("campaign", "status_changed")`
#[derive(Clone)]
#[contracttype]
pub struct EventStatusChanged {
    pub old_status: Status,
    pub new_status: Status,
}

/// Emitted when campaign metadata is updated.
///
/// Event topic: `("campaign", "metadata_updated")`
#[derive(Clone)]
#[contracttype]
pub struct EventMetadataUpdated {
    pub updated_title: bool,
    pub updated_description: bool,
    pub updated_social_links: bool,
}

/// Emitted when the campaign deadline is extended directly by the creator.
///
/// Event topic: `("campaign", "deadline_extended")`
#[derive(Clone)]
#[contracttype]
pub struct EventDeadlineExtended {
    pub old_deadline: u64,
    pub new_deadline: u64,
}

/// Emitted when a deadline extension proposal is created.
///
/// Event topic: `("campaign", "extension_proposed")`
#[derive(Clone)]
#[contracttype]
pub struct EventExtensionProposed {
    pub new_deadline: u64,
    pub voting_ends_at: u64,
}

/// Emitted when a contributor votes on a deadline extension.
///
/// Event topic: `("campaign", "extension_voted")`
#[derive(Clone)]
#[contracttype]
pub struct EventExtensionVoted {
    pub contributor: Address,
    pub approve: bool,
    pub vote_weight: i128,
}

/// Emitted when a deadline extension is executed after successful voting.
///
/// Event topic: `("campaign", "extension_executed")`
#[derive(Clone)]
#[contracttype]
pub struct EventExtensionExecuted {
    pub new_deadline: u64,
    pub votes_for: i128,
    pub votes_against: i128,
}

/// Emitted when a recurring contribution plan is set up.
///
/// Event topic: `("campaign", "recurring_setup")`
#[derive(Clone)]
#[contracttype]
pub struct EventRecurringSetup {
    pub contributor: Address,
    pub amount: i128,
    pub interval: u64,
    pub end_date: u64,
}

/// Emitted when a recurring contribution is executed.
///
/// Event topic: `("campaign", "recurring_executed")`
#[derive(Clone)]
#[contracttype]
pub struct EventRecurringExecuted {
    pub contributor: Address,
    pub amount: i128,
}

/// Emitted when a recurring plan is cancelled.
///
/// Event topic: `("campaign", "recurring_cancelled")`
#[derive(Clone)]
#[contracttype]
pub struct EventRecurringCancelled {
    pub contributor: Address,
}

/// Emitted when a delegation is created.
///
/// Event topic: `("campaign", "delegation_created")`
#[derive(Clone)]
#[contracttype]
pub struct EventDelegationCreated {
    pub delegator: Address,
    pub delegate: Address,
    pub amount: i128,
}

/// Emitted when a delegated contribution is made.
///
/// Event topic: `("campaign", "delegated_contribution")`
#[derive(Clone)]
#[contracttype]
pub struct EventDelegatedContribution {
    pub delegator: Address,
    pub delegate: Address,
    pub amount: i128,
}

/// Emitted when a delegation is revoked.
///
/// Event topic: `("campaign", "delegation_revoked")`
#[derive(Clone)]
#[contracttype]
pub struct EventDelegationRevoked {
    pub delegator: Address,
}

/// Emitted when an address is added to the whitelist.
///
/// Event topic: `("campaign", "whitelisted")`
#[derive(Clone)]
#[contracttype]
pub struct EventWhitelisted {
    pub address: Address,
}

/// Emitted when an address is removed from the whitelist.
///
/// Event topic: `("campaign", "whitelist_removed")`
#[derive(Clone)]
#[contracttype]
pub struct EventWhitelistRemoved {
    pub address: Address,
}

/// Emitted when an address is added to the blacklist.
///
/// Event topic: `("campaign", "blacklisted")`
#[derive(Clone)]
#[contracttype]
pub struct EventBlacklisted {
    pub address: Address,
}

/// Emitted when an address is removed from the blacklist.
///
/// Event topic: `("campaign", "blacklist_removed")`
#[derive(Clone)]
#[contracttype]
pub struct EventBlacklistRemoved {
    pub address: Address,
}

/// Emitted when whitelist-only mode is toggled.
///
/// Event topic: `("campaign", "whitelist_only_set")`
#[derive(Clone)]
#[contracttype]
pub struct EventWhitelistOnlySet {
    pub enabled: bool,
}

/// Emitted when the rate limit is updated.
///
/// Event topic: `("campaign", "rate_limit_updated")`
#[derive(Clone)]
#[contracttype]
pub struct EventRateLimitUpdated {
    /// Maximum total contribution amount per address within `window_seconds`.
    pub max_amount: i128,
    /// Window length in seconds (0 when the rate limit is cleared).
    pub window_seconds: u64,
}

/// Emitted when a contribution is rejected because it would exceed the rate limit.
///
/// Event topic: `("campaign", "rate_limit_hit")`
#[derive(Clone)]
#[contracttype]
pub struct EventRateLimitHit {
    pub contributor: Address,
    /// Amount the contributor attempted to add.
    pub attempted: i128,
    /// Amount already counted toward the contributor's current window.
    pub period_amount: i128,
    /// Configured maximum for the window.
    pub max_amount: i128,
}

/// Emitted when a campaign's visibility level is changed.
///
/// Event topic: `("campaign", "visibility_changed")`
#[derive(Clone)]
#[contracttype]
pub struct EventVisibilityChanged {
    pub old_visibility: Visibility,
    pub new_visibility: Visibility,
}

/// Emitted when an emergency withdrawal is initiated.
///
/// Event topic: `("campaign", "emergency_initiated")`
#[derive(Clone)]
#[contracttype]
pub struct EventEmergencyInitiated {
    pub lock_until: u64,
}

/// Emitted when an emergency withdrawal is executed.
///
/// Event topic: `("campaign", "emergency_executed")`
#[derive(Clone)]
#[contracttype]
pub struct EventEmergencyExecuted {
    pub amount: i128,
}

/// Emitted when insurance is enabled for the campaign.
///
/// Event topic: `("insurance", "enabled")`
#[derive(Clone)]
#[contracttype]
pub struct EventInsuranceEnabled {
    pub fee_bps: u32,
    pub provider: Address,
}

/// Emitted when an insurance payout is processed.
///
/// Event topic: `("insurance", "payout")`
#[derive(Clone)]
#[contracttype]
pub struct EventInsurancePayout {
    pub contributor: Address,
    pub amount: i128,
}

/// Emitted when emergency withdrawal multi-sig is configured.
///
/// Event topic: `("campaign", "multisig_configured")`
#[derive(Clone)]
#[contracttype]
pub struct EventMultiSigConfigured {
    /// Minimum number of approvals required to execute the emergency withdrawal
    pub required_approvals: u32,
    /// Total number of authorised approver addresses
    pub approver_count: u32,
}

/// Emitted when an emergency withdrawal approval is submitted by an approver.
///
/// Event topic: `("campaign", "emergency_approved")`
#[derive(Clone)]
#[contracttype]
pub struct EventEmergencyApproved {
    /// Address of the approver who submitted this approval
    pub approver: Address,
    /// Running approval count for the current session after this approval
    pub approval_count: u32,
}

/// Emitted when a contribution matching pool is configured.
///
/// Event topic: `("campaign", "matching_setup")`
#[derive(Clone)]
#[contracttype]
pub struct EventMatchingSetup {
    /// Sponsor address funding the matching pool
    pub sponsor: Address,
    /// Match ratio in basis points (e.g. 10 000 = 1 : 1)
    pub match_ratio: u32,
    /// Maximum total matching amount in stroops
    pub max_match: i128,
}

/// Emitted when a campaign is initialised via a template.
///
/// Event topic: `("campaign", "template_applied")`
#[derive(Clone)]
#[contracttype]
pub struct EventTemplateApplied {
    /// Template type used to initialise the campaign
    pub template_type: TemplateType,
    /// Minimum contribution derived from the template
    pub suggested_min: i128,
}

/// Emitted when the campaign category is updated by the creator.
///
/// Event topic: `("campaign", "category_updated")`
#[derive(Clone)]
#[contracttype]
pub struct EventCategoryUpdated {
    /// Previous category before the update
    pub old_category: Category,
    /// New category after the update
    pub new_category: Category,
}
