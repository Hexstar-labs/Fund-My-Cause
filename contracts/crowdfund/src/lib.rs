//! # Fund-My-Cause Crowdfund Contract
//!
//! A Soroban smart contract for decentralised crowdfunding on the Stellar network.
//!
//! ## Overview
//!
//! Each deployed instance of [`CrowdfundContract`] represents a single crowdfunding
//! campaign. The contract lifecycle is:
//!
//! 1. **Initialise** — creator calls [`initialize`](CrowdfundContract::initialize) once.
//! 2. **Contribute** — backers call [`contribute`](CrowdfundContract::contribute) before the deadline.
//! 3. **Withdraw** — if goal is met after deadline, creator calls [`withdraw`](CrowdfundContract::withdraw).
//! 4. **Refund** — if goal is not met (or campaign is cancelled), contributors call
//!    [`refund_single`](CrowdfundContract::refund_single) to reclaim their funds.
//!
//! ## Advanced Features
//!
//! - **Recurring contributions** — [`setup_recurring`](CrowdfundContract::setup_recurring) /
//!   [`execute_recurring`](CrowdfundContract::execute_recurring)
//! - **Delegation** — [`delegate_contribution`](CrowdfundContract::delegate_contribution) /
//!   [`contribute_on_behalf`](CrowdfundContract::contribute_on_behalf)
//! - **Deadline extension voting** — [`propose_extension`](CrowdfundContract::propose_extension) /
//!   [`vote_on_extension`](CrowdfundContract::vote_on_extension) /
//!   [`execute_extension`](CrowdfundContract::execute_extension)
//! - **Whitelist / Blacklist** — [`add_to_whitelist`](CrowdfundContract::add_to_whitelist) /
//!   [`add_to_blacklist`](CrowdfundContract::add_to_blacklist)
//! - **Partial refunds** — [`refund_partial`](CrowdfundContract::refund_partial)
//! - **Emergency withdrawal** — [`initiate_emergency_withdrawal`](CrowdfundContract::initiate_emergency_withdrawal) /
//!   [`execute_emergency_withdrawal`](CrowdfundContract::execute_emergency_withdrawal)
//! - **Insurance** — [`enable_insurance`](CrowdfundContract::enable_insurance)
//! - **Vesting** — configurable cliff + linear vesting on withdrawal
//! - **Matching** — sponsor-funded contribution matching
//!
//! ## Storage Model
//!
//! - **Instance storage** — campaign-wide state (status, goal, deadline, totals).
//! - **Persistent storage** — per-contributor data (balances, plans, flags).
//!
//! ## Error Handling
//!
//! All mutating functions return `Result<_, ContractError>`. See [`errors::ContractError`]
//! for the full list of error codes.
//!
//! ## Events
//!
//! Every state-changing function publishes a structured event. See the `Event*` types
//! in [`types`] for the full list of event payloads.

#![no_std]
#![allow(clippy::too_many_arguments)]

mod errors;
mod storage;
mod types;
mod validation;

pub use errors::ContractError;
pub use storage::{
    CONTRACT_VERSION, KEY_ADMIN, KEY_CATEGORY, KEY_CONTRIBS, KEY_CREATOR, KEY_DEADLINE, KEY_DESC,
    KEY_GOAL, KEY_GOAL_HISTORY, KEY_INSURANCE, KEY_INSURANCE_POOL, KEY_MAX, KEY_MIN, KEY_PLATFORM,
    KEY_RATE_LIMIT, KEY_SOCIAL, KEY_STATUS, KEY_TITLE, KEY_TOKEN, KEY_TOTAL, KEY_VESTING,
    KEY_VISIBILITY,
};
pub use types::{
    CampaignInfo,
    CampaignStats,
    CampaignTemplate,
    Category,
    DataKey,
    Delegation,
    EventBlacklistRemoved,
    EventBlacklisted,
    EventContributed,
    EventDeadlineExtended,
    EventDelegatedContribution,
    EventDelegationCreated,
    EventDelegationRevoked,
    EventEmergencyExecuted,
    EventEmergencyInitiated,
    EventExtensionExecuted,
    EventExtensionProposed,
    EventExtensionVoted,
    // Event payload types
    EventInitialized,
    EventInsuranceEnabled,
    EventInsurancePayout,
    EventMetadataUpdated,
    EventPartialRefund,
    EventRateLimitHit,
    EventRateLimitUpdated,
    EventRecurringCancelled,
    EventRecurringExecuted,
    EventRecurringSetup,
    EventRefunded,
    EventStatusChanged,
    EventVisibilityChanged,
    EventWhitelistOnlySet,
    EventWhitelistRemoved,
    EventWhitelisted,
    EventWithdrawn,
    ExtensionProposal,
    GoalAdjustment,
    InsuranceConfig,
    MatchingConfig,
    PlatformConfig,
    RateLimit,
    RecurringPlan,
    Status,
    TemplateType,
    VestingSchedule,
    Visibility,
};
pub use validation::*;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct CrowdfundContract;

#[contractimpl]
impl CrowdfundContract {
    /// Initializes a new crowdfunding campaign.
    ///
    /// Creates a campaign with the specified parameters. Can only be called once per contract instance.
    /// The creator must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The campaign creator's Stellar address (must authorize)
    /// * `token` - The token address for contributions (e.g., native XLM or custom token)
    /// * `goal` - The funding goal in stroops (must be > 0)
    /// * `deadline` - Unix timestamp (seconds) when the campaign ends (must be > current ledger time)
    /// * `min_contribution` - Minimum contribution amount in stroops (must be >= 0)
    /// * `title` - Campaign title
    /// * `description` - Campaign description
    /// * `social_links` - Optional list of social media URLs
    /// * `platform_config` - Optional platform fee configuration (address and fee_bps)
    /// * `accepted_tokens` - Optional whitelist of accepted token addresses
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::AlreadyInitialized)` if campaign already initialized
    /// * `Err(ContractError::InvalidGoal)` if goal <= 0
    /// * `Err(ContractError::InvalidDeadline)` if deadline <= current time
    /// * `Err(ContractError::InvalidFee)` if platform fee_bps > 10,000
    ///
    /// # Example
    /// ```ignore
    /// initialize(
    ///     env,
    ///     creator_address,
    ///     token_address,
    ///     1_000_000_000,  // 100 XLM goal
    ///     1704067200,     // deadline timestamp
    ///     1_000_000,      // 0.1 XLM minimum
    ///     String::from_str(&env, "My Campaign"),
    ///     String::from_str(&env, "Help fund my project"),
    ///     None,
    ///     None,
    ///     None,
    /// )
    /// ```
    pub fn initialize(
        env: Env,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        max_contribution: i128,
        title: String,
        description: String,
        social_links: Option<Vec<String>>,
        platform_config: Option<PlatformConfig>,
        accepted_tokens: Option<Vec<Address>>,
        category: Category,
        vesting: Option<VestingSchedule>,
        penalty_bps: Option<u32>,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&KEY_CREATOR) {
            return Err(ContractError::AlreadyInitialized);
        }
        creator.require_auth();

        if goal <= 0 {
            return Err(ContractError::InvalidGoal);
        }
        validate_goal_not_overflow(goal)?;
        if deadline <= env.ledger().timestamp() {
            return Err(ContractError::InvalidDeadline);
        }
        if min_contribution < 0 {
            return Err(ContractError::BelowMinimum);
        }
        if max_contribution < 0 || (max_contribution > 0 && max_contribution < min_contribution) {
            return Err(ContractError::ExceedsMaximum);
        }
        validate_string_length(&title, 64)?;
        validate_string_length(&description, 512)?;

        if let Some(ref config) = platform_config {
            validate_fee_bps(config.fee_bps)?;
            validate_address_not_self(&creator, &config.address)?;
            env.storage().instance().set(&KEY_PLATFORM, config);
        }

        // ── Batch all instance writes in one block to minimise storage overhead ──
        let storage = env.storage().instance();
        storage.set(&KEY_ADMIN, &creator);
        storage.set(&KEY_CREATOR, &creator);
        storage.set(&KEY_TOKEN, &token);
        storage.set(&KEY_GOAL, &goal);
        storage.set(&KEY_DEADLINE, &deadline);
        storage.set(&KEY_MIN, &min_contribution);
        storage.set(&KEY_MAX, &max_contribution);
        storage.set(&KEY_TITLE, &title);
        storage.set(&KEY_DESC, &description);
        storage.set(&KEY_TOTAL, &0i128);
        storage.set(&KEY_STATUS, &Status::Active);
        storage.set(&KEY_CATEGORY, &category);
        storage.set(&KEY_VISIBILITY, &Visibility::Public);
        storage.set(&DataKey::ContributorCount, &0u32);
        storage.set(&DataKey::LargestContribution, &0i128);

        if let Some(links) = social_links {
            storage.set(&KEY_SOCIAL, &links);
        }
        if let Some(tokens) = accepted_tokens {
            storage.set(&DataKey::AcceptedTokens, &tokens);
        }
        if let Some(v) = vesting {
            storage.set(&KEY_VESTING, &v);
        }
        if let Some(p) = penalty_bps {
            storage.set(&DataKey::PenaltyBps, &p);
        }

        // Persistent writes (separate storage tier)
        let empty: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&KEY_CONTRIBS, &empty);

        let mut history: Vec<GoalAdjustment> = Vec::new(&env);
        history.push_back(GoalAdjustment {
            previous_goal: 0,
            new_goal: goal,
            timestamp: env.ledger().timestamp(),
        });
        env.storage().persistent().set(&KEY_GOAL_HISTORY, &history);

        env.events().publish(
            ("campaign", "initialized"),
            EventInitialized {
                creator,
                goal,
                deadline,
            },
        );
        Ok(())
    }

    /// Submits a contribution to the campaign.
    ///
    /// Allows a contributor to pledge tokens before the campaign deadline.
    /// The contributor must authorize this transaction and have sufficient token balance.
    /// Uses a pull-based refund model: contributors claim refunds individually if the goal is not met.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address (must authorize)
    /// * `amount` - Contribution amount in stroops (must be >= min_contribution)
    /// * `token` - The token address being contributed (must match campaign token or be in whitelist)
    /// * `message` - Optional message/memo attached to the contribution (max 256 chars)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::BelowMinimum)` if amount < min_contribution
    /// * `Err(ContractError::CampaignPaused)` if campaign is paused
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::CampaignEnded)` if current time >= deadline
    /// * `Err(ContractError::TokenNotAccepted)` if token not in whitelist
    /// * `Err(ContractError::Overflow)` if total raised would overflow
    /// * `Err(ContractError::MessageTooLong)` if message exceeds 256 characters
    ///
    /// # Side Effects
    /// - Transfers tokens from contributor to contract
    /// - Updates contributor's total contribution amount
    /// - Stores contribution message if provided
    /// - Increments contributor count if this is their first contribution
    /// - Updates largest contribution if applicable
    /// - Stores anonymity flag if anonymous=true
    /// - Publishes "contributed" event
    pub fn contribute(
        env: Env,
        contributor: Address,
        amount: i128,
        token: Address,
        message: Option<String>,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        validate_positive_amount(amount)?;

        if let Some(ref msg) = message {
            if msg.len() > 256 {
                return Err(ContractError::MessageTooLong);
            }
        }

        // ── Batch all instance reads up-front to avoid repeated storage lookups ──
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        let min: i128 = inst.get(&KEY_MIN).unwrap();
        let max: i128 = inst.get(&KEY_MAX).unwrap_or(0);
        let deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
        let default_token: Address = inst.get(&KEY_TOKEN).unwrap();
        let rate_limit: Option<RateLimit> = inst.get(&KEY_RATE_LIMIT);
        let visibility: Visibility = inst.get(&KEY_VISIBILITY).unwrap_or(Visibility::Public);
        let accepted_tokens: Option<Vec<Address>> = inst.get(&DataKey::AcceptedTokens);
        let total: i128 = inst.get(&KEY_TOTAL).unwrap();
        let count: u32 = inst.get(&DataKey::ContributorCount).unwrap();
        let largest: i128 = inst.get(&DataKey::LargestContribution).unwrap();

        // ── Validate status ───────────────────────────────────────────────────
        if status == Status::Paused {
            return Err(ContractError::CampaignPaused);
        }
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }

        // ── Validate deadline ─────────────────────────────────────────────────
        let now = env.ledger().timestamp();
        if now >= deadline {
            return Err(ContractError::CampaignEnded);
        }

        // ── Check blacklist / whitelist (persistent, per-address) ─────────────
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Blacklist(contributor.clone()))
            .unwrap_or(false)
        {
            return Err(ContractError::Blacklisted);
        }
        let whitelist_only: bool = inst.get(&DataKey::WhitelistOnly).unwrap_or(false);
        let needs_whitelist = whitelist_only || visibility == Visibility::Private;
        if needs_whitelist
            && !env
                .storage()
                .persistent()
                .get::<_, bool>(&DataKey::Whitelist(contributor.clone()))
                .unwrap_or(false)
        {
            return Err(ContractError::NotWhitelisted);
        }

        // ── Validate amount ───────────────────────────────────────────────────
        if amount < min {
            return Err(ContractError::BelowMinimum);
        }

        // Read contributor's existing balance once; reuse below
        let contrib_key = DataKey::Contribution(contributor.clone());
        let prev_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);

        if max > 0 {
            let new_total = prev_contrib
                .checked_add(amount)
                .ok_or(ContractError::Overflow)?;
            if new_total > max {
                return Err(ContractError::ExceedsMaximum);
            }
        }

        // ── Rate limit check (reuse cached `now`) ─────────────────────────────
        if let Some(rl) = rate_limit {
            if rl.max_amount > 0 && rl.window_seconds > 0 {
                let ts_key = DataKey::RateLimitTimestamp(contributor.clone());
                let amt_key = DataKey::RateLimitAmount(contributor.clone());
                let last_ts: u64 = env.storage().persistent().get(&ts_key).unwrap_or(0);

                let in_window = last_ts > 0 && now.saturating_sub(last_ts) < rl.window_seconds;
                let period_amount: i128 = if in_window {
                    env.storage().persistent().get(&amt_key).unwrap_or(0)
                } else {
                    0
                };
                let new_period = period_amount
                    .checked_add(amount)
                    .ok_or(ContractError::Overflow)?;
                if new_period > rl.max_amount {
                    env.events().publish(
                        ("campaign", "rate_limit_hit"),
                        EventRateLimitHit {
                            contributor: contributor.clone(),
                            attempted: amount,
                            period_amount,
                            max_amount: rl.max_amount,
                        },
                    );
                    return Err(ContractError::RateLimitExceeded);
                }
                if in_window {
                    env.storage().persistent().set(&amt_key, &new_period);
                } else {
                    env.storage().persistent().set(&ts_key, &now);
                    env.storage().persistent().set(&amt_key, &amount);
                }
            }
        }

        // ── Validate token (use cached accepted_tokens) ───────────────────────
        if let Some(ref whitelist) = accepted_tokens {
            if !whitelist.contains(&token) {
                return Err(ContractError::TokenNotAccepted);
            }
        } else if token != default_token {
            return Err(ContractError::TokenNotAccepted);
        }

        // ── Transfer tokens ───────────────────────────────────────────────────
        token::Client::new(&env, &token).transfer(
            &contributor,
            &env.current_contract_address(),
            &amount,
        );

        // ── Update contributor balance (single write) ─────────────────────────
        let new_contrib = prev_contrib
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&contrib_key, &new_contrib);
        env.storage()
            .persistent()
            .extend_ttl(&contrib_key, 100, 100);

        if let Some(msg) = message {
            let msg_key = DataKey::ContributionMessage(contributor.clone());
            env.storage().persistent().set(&msg_key, &msg);
            env.storage().persistent().extend_ttl(&msg_key, 100, 100);
        }

        // ── Apply matching (cached instance read) ─────────────────────────────
        let new_running_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;
        let mut matched_amount = 0i128;
        if let Some(config) = inst.get::<_, MatchingConfig>(&DataKey::MatchingConfig) {
            let match_amount = (amount * config.match_ratio as i128) / 10_000;
            let total_matched: i128 = inst.get(&DataKey::TotalMatched).unwrap_or(0);
            let available_match = config.max_match - total_matched;
            matched_amount = match_amount.min(available_match).max(0);
            if matched_amount > 0 {
                inst.set(&DataKey::TotalMatched, &(total_matched + matched_amount));
            }
        }

        let final_total = new_running_total
            .checked_add(matched_amount)
            .ok_or(ContractError::Overflow)?;
        inst.set(&KEY_TOTAL, &final_total);

        // ── First-time contributor bookkeeping ────────────────────────────────
        let presence_key = DataKey::ContributorPresence(contributor.clone());
        let is_present: bool = env
            .storage()
            .persistent()
            .get(&presence_key)
            .unwrap_or(false);
        if !is_present {
            env.storage().persistent().set(&presence_key, &true);
            env.storage()
                .persistent()
                .extend_ttl(&presence_key, 100, 100);
            // Use cached `count` — single write
            inst.set(&DataKey::ContributorCount, &(count + 1));

            let mut contributors: Vec<Address> = env
                .storage()
                .persistent()
                .get(&KEY_CONTRIBS)
                .unwrap_or_else(|| Vec::new(&env));
            contributors.push_back(contributor.clone());
            env.storage().persistent().set(&KEY_CONTRIBS, &contributors);
            env.storage()
                .persistent()
                .extend_ttl(&KEY_CONTRIBS, 100, 100);
        }

        // Use cached `largest` — conditional single write
        if new_contrib > largest {
            inst.set(&DataKey::LargestContribution, &new_contrib);
        }

        inst.extend_ttl(17280, 518400);

        env.events().publish(
            ("campaign", "contributed"),
            EventContributed {
                contributor,
                amount,
                new_total: new_contrib,
                matched_amount,
            },
        );
        Ok(())
    }

    /// Withdraws raised funds to the campaign creator after a successful campaign.
    ///
    /// Can only be called after the deadline has passed and the goal has been reached.
    /// The creator must authorize this transaction.
    /// If a platform fee is configured, it is deducted from the total before payout.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::CampaignStillActive)` if current time < deadline
    /// * `Err(ContractError::GoalNotReached)` if total_raised < goal
    ///
    /// # Side Effects
    /// - Transfers platform fee to platform address (if configured)
    /// - Transfers remaining funds to creator
    /// - Sets campaign status to Successful
    /// - Resets total_raised to 0
    /// - Publishes "withdrawn" event
    ///
    /// # Platform Fee Calculation
    /// If platform_config is set:
    /// ```ignore
    /// fee = total_raised * platform_fee_bps / 10_000
    /// creator_payout = total_raised - fee
    /// ```
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        // ── Batch all instance reads up-front ─────────────────────────────────
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        let deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
        let goal: i128 = inst.get(&KEY_GOAL).unwrap();
        let total: i128 = inst.get(&KEY_TOTAL).unwrap();
        let token_address: Address = inst.get(&KEY_TOKEN).unwrap();
        let platform_config: Option<PlatformConfig> = inst.get(&KEY_PLATFORM);
        let vesting: Option<VestingSchedule> = inst.get(&KEY_VESTING);

        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        creator.require_auth();

        let now = env.ledger().timestamp();
        if now < deadline {
            return Err(ContractError::CampaignStillActive);
        }
        if total < goal {
            return Err(ContractError::GoalNotReached);
        }

        let token_client = token::Client::new(&env, &token_address);

        // ── Calculate fee and payout ──────────────────────────────────────────
        let fee = if let Some(ref config) = platform_config {
            let f = total * config.fee_bps as i128 / 10_000;
            token_client.transfer(&env.current_contract_address(), &config.address, &f);
            f
        } else {
            0
        };
        let mut payout = total - fee;

        // ── Apply vesting if configured ───────────────────────────────────────
        if let Some(ref v) = vesting {
            if now < v.cliff {
                return Err(ContractError::VestingNotComplete);
            }
            let vested = if now >= v.cliff + v.duration {
                payout
            } else {
                let elapsed = now - v.cliff;
                payout * elapsed as i128 / v.duration as i128
            };
            token_client.transfer(&env.current_contract_address(), &creator, &vested);
            payout = vested;
        } else {
            token_client.transfer(&env.current_contract_address(), &creator, &payout);
        }

        // ── Batch all instance writes ─────────────────────────────────────────
        inst.set(&KEY_TOTAL, &0i128);
        inst.set(&KEY_STATUS, &Status::Successful);
        inst.extend_ttl(17280, 518400);

        env.events().publish(
            ("campaign", "withdrawn"),
            EventWithdrawn {
                creator,
                total,
                fee,
                payout,
            },
        );
        Ok(())
    }

    /// Updates campaign metadata (title, description, social links).
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// Any field can be omitted (None) to leave it unchanged.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `title` - New campaign title (optional)
    /// * `description` - New campaign description (optional)
    /// * `social_links` - New social media links (optional)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Updates specified metadata fields in storage
    /// - Publishes "metadata_updated" event
    pub fn update_metadata(
        env: Env,
        title: Option<String>,
        description: Option<String>,
        social_links: Option<Vec<String>>,
    ) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let updated_title = title.is_some();
        let updated_description = description.is_some();
        let updated_social = social_links.is_some();

        if let Some(ref t) = title {
            validate_string_length(t, 64)?;
        }
        if let Some(ref d) = description {
            validate_string_length(d, 512)?;
        }
        if let Some(t) = title {
            inst.set(&KEY_TITLE, &t);
        }
        if let Some(d) = description {
            inst.set(&KEY_DESC, &d);
        }
        if let Some(l) = social_links {
            inst.set(&KEY_SOCIAL, &l);
        }

        env.events().publish(
            ("campaign", "metadata_updated"),
            EventMetadataUpdated {
                updated_title,
                updated_description,
                updated_social_links: updated_social,
            },
        );
        Ok(())
    }

    /// Extends the campaign deadline to a later time.
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// The new deadline must be strictly greater than the current deadline.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_deadline` - New Unix timestamp (seconds) for campaign end
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::InvalidDeadline)` if new_deadline <= current_deadline
    ///
    /// # Side Effects
    /// - Updates deadline in storage
    /// - Publishes "deadline_extended" event with new deadline
    pub fn extend_deadline(env: Env, new_deadline: u64) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let old_deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
        if new_deadline <= old_deadline {
            return Err(ContractError::InvalidDeadline);
        }
        inst.set(&KEY_DEADLINE, &new_deadline);
        env.events().publish(
            ("campaign", "deadline_extended"),
            EventDeadlineExtended {
                old_deadline,
                new_deadline,
            },
        );
        Ok(())
    }

    /// Cancels the campaign, allowing all contributors to claim refunds.
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// After cancellation, contributors can call `refund_single` to claim their refunds.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Sets campaign status to Cancelled
    /// - Publishes "cancelled" event
    pub fn cancel_campaign(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        inst.set(&KEY_STATUS, &Status::Cancelled);
        env.events().publish(("campaign", "cancelled"), ());
        env.events().publish(
            ("campaign", "status_changed"),
            EventStatusChanged {
                old_status: Status::Active,
                new_status: Status::Cancelled,
            },
        );
        Ok(())
    }

    /// Claims a refund for a single contributor (pull-based refund model).
    ///
    /// A contributor can claim their refund if:
    /// - The campaign was cancelled, OR
    /// - The deadline has passed AND the goal was not reached
    ///
    /// This implements a pull-based refund model where each contributor individually
    /// claims their refund, avoiding the gas cost and failure risk of a single
    /// transaction refunding all contributors.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address claiming the refund
    ///
    /// # Returns
    /// * `Ok(())` on success (even if contributor has no refund)
    /// * `Err(ContractError::CampaignStillActive)` if deadline not passed and not cancelled
    /// * `Err(ContractError::GoalReached)` if goal was reached and campaign not cancelled
    ///
    /// # Side Effects
    /// - Transfers refund amount to contributor (if > 0)
    /// - Sets contributor's contribution to 0
    /// - Publishes "refunded" event
    pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
        // ── Batch instance reads up-front ─────────────────────────────────────
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();

        if status != Status::Cancelled {
            let deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
            if env.ledger().timestamp() < deadline {
                return Err(ContractError::CampaignStillActive);
            }
            let goal: i128 = inst.get(&KEY_GOAL).unwrap();
            let total: i128 = inst.get(&KEY_TOTAL).unwrap();
            if total >= goal {
                return Err(ContractError::GoalReached);
            }
        }

        let key = DataKey::Contribution(contributor.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount > 0 {
            let token_address: Address = inst.get(&KEY_TOKEN).unwrap();
            token::Client::new(&env, &token_address).transfer(
                &env.current_contract_address(),
                &contributor,
                &amount,
            );
            env.storage().persistent().set(&key, &0i128);
            env.events().publish(
                ("campaign", "refunded"),
                EventRefunded {
                    contributor,
                    amount,
                },
            );
        }
        Ok(())
    }

    /// Refunds multiple contributors in a single transaction (batch refund).
    ///
    /// Processes refunds for a list of contributors. Stops early if the batch
    /// limit is reached to avoid exceeding resource limits.
    /// Each contributor is only refunded if eligible (same conditions as `refund_single`).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributors` - List of contributor addresses to refund
    ///
    /// # Returns
    /// * `Ok(u32)` - Number of contributors successfully refunded
    /// * `Err(ContractError::CampaignStillActive)` if deadline not passed and not cancelled
    /// * `Err(ContractError::GoalReached)` if goal was reached and campaign not cancelled
    pub fn refund_batch(env: Env, contributors: Vec<Address>) -> Result<u32, ContractError> {
        // ── Batch instance reads up-front ─────────────────────────────────────
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();

        if status != Status::Cancelled {
            let deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
            if env.ledger().timestamp() < deadline {
                return Err(ContractError::CampaignStillActive);
            }
            let goal: i128 = inst.get(&KEY_GOAL).unwrap();
            let total: i128 = inst.get(&KEY_TOTAL).unwrap();
            if total >= goal {
                return Err(ContractError::GoalReached);
            }
        }

        // Cache token address once for the whole batch
        let token_address: Address = inst.get(&KEY_TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // Cap batch size to avoid resource exhaustion
        const MAX_BATCH: u32 = 25;
        let limit = contributors.len().min(MAX_BATCH);
        let mut refunded: u32 = 0;

        for i in 0..limit {
            let contributor = contributors.get(i).unwrap();
            let key = DataKey::Contribution(contributor.clone());
            let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
            if amount > 0 {
                token_client.transfer(&env.current_contract_address(), &contributor, &amount);
                env.storage().persistent().set(&key, &0i128);
                env.events().publish(
                    ("campaign", "refunded"),
                    EventRefunded {
                        contributor,
                        amount,
                    },
                );
                refunded += 1;
            }
        }

        Ok(refunded)
    }

    /// Sets the per-address contribution rate limit (admin only).
    ///
    /// Configures the maximum amount a single address can contribute within a
    /// rolling window of `window_seconds`. Passing `max_amount = 0` clears the
    /// rate limit (and the window value is then ignored).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `max_amount` - Maximum contribution amount per address per window (0 = disabled)
    /// * `window_seconds` - Length of the per-address window in seconds (must be > 0 when enabling)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidRateLimit)` if `max_amount < 0`, or if enabling
    ///   the limit with `window_seconds == 0`
    ///
    /// # Side Effects
    /// - Updates or clears the stored rate limit configuration
    /// - Publishes a "rate_limit_updated" event
    pub fn set_rate_limit(
        env: Env,
        max_amount: i128,
        window_seconds: u64,
    ) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();

        if max_amount < 0 {
            return Err(ContractError::InvalidRateLimit);
        }
        if max_amount == 0 {
            inst.remove(&KEY_RATE_LIMIT);
            env.events().publish(
                ("campaign", "rate_limit_updated"),
                EventRateLimitUpdated {
                    max_amount: 0,
                    window_seconds: 0,
                },
            );
            return Ok(());
        }
        if window_seconds == 0 {
            return Err(ContractError::InvalidRateLimit);
        }
        inst.set(
            &KEY_RATE_LIMIT,
            &RateLimit {
                max_amount,
                window_seconds,
            },
        );
        env.events().publish(
            ("campaign", "rate_limit_updated"),
            EventRateLimitUpdated {
                max_amount,
                window_seconds,
            },
        );
        Ok(())
    }

    /// Returns the current per-address rate limit configuration, if any.
    pub fn get_rate_limit(env: Env) -> Option<RateLimit> {
        env.storage().instance().get(&KEY_RATE_LIMIT)
    }

    /// Initiates an emergency withdrawal (admin only).
    ///
    /// Starts a time-locked emergency withdrawal process. After the lock period expires,
    /// the admin can call `execute_emergency_withdrawal()` to recover funds.
    /// This requires admin authorization and can be cancelled before execution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `lock_period` - Time in seconds to lock the withdrawal (e.g., 604800 for 7 days)
    ///
    /// # Returns
    /// * `Ok(())` on success
    ///
    /// # Side Effects
    /// - Sets emergency lock time to current time + lock_period
    /// - Publishes "EmergencyWithdrawalInitiated" event
    pub fn initiate_emergency_withdrawal(env: Env, lock_period: u64) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        let lock_until = env.ledger().timestamp() + lock_period;
        inst.set(&DataKey::EmergencyLockTime, &lock_until);
        env.events().publish(
            ("campaign", "emergency_initiated"),
            EventEmergencyInitiated { lock_until },
        );
        Ok(())
    }

    /// Executes the emergency withdrawal (admin only).
    ///
    /// Transfers all funds to the admin after the lock period has expired.
    /// Can only be called after the time-lock delay has passed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::EmergencyLocked)` if lock period has not expired
    ///
    /// # Side Effects
    /// - Transfers all funds to admin
    /// - Clears emergency lock time
    /// - Publishes "EmergencyWithdrawalExecuted" event
    pub fn execute_emergency_withdrawal(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();

        let lock_time: u64 = inst.get(&DataKey::EmergencyLockTime).unwrap_or(0);
        if lock_time == 0 || env.ledger().timestamp() < lock_time {
            return Err(ContractError::EmergencyLocked);
        }

        let total: i128 = inst.get(&KEY_TOTAL).unwrap();
        if total > 0 {
            let token_address: Address = inst.get(&KEY_TOKEN).unwrap();
            token::Client::new(&env, &token_address).transfer(
                &env.current_contract_address(),
                &admin,
                &total,
            );
            inst.set(&KEY_TOTAL, &0i128);
        }

        inst.set(&DataKey::EmergencyLockTime, &0u64);
        env.events().publish(
            ("campaign", "emergency_executed"),
            EventEmergencyExecuted { amount: total },
        );
        Ok(())
    }

    /// Cancels a pending emergency withdrawal (admin only).
    ///
    /// Removes the emergency lock, preventing the withdrawal from being executed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    ///
    /// # Side Effects
    /// - Clears emergency lock time
    /// - Publishes "EmergencyWithdrawalCancelled" event
    pub fn cancel_emergency_withdrawal(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        inst.set(&DataKey::EmergencyLockTime, &0u64);
        env.events()
            .publish(("campaign", "emergency_cancelled"), ());
        Ok(())
    }

    /// Verify campaign (admin only).
    ///
    /// Can only be called while the campaign is in Active status.
    /// The admin (creator) must authorize this transaction.
    /// While paused, contributors cannot make new contributions.
    /// The campaign can be resumed with `unpause`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Sets campaign status to Paused
    /// - Publishes "paused" event
    pub fn pause(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        inst.set(&KEY_STATUS, &Status::Paused);
        env.events().publish(("campaign", "paused"), ());
        env.events().publish(
            ("campaign", "status_changed"),
            EventStatusChanged {
                old_status: Status::Active,
                new_status: Status::Paused,
            },
        );
        Ok(())
    }

    /// Resumes a paused campaign, allowing contributions again.
    ///
    /// Can only be called while the campaign is in Paused status.
    /// The admin (creator) must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Paused status
    ///
    /// # Side Effects
    /// - Sets campaign status to Active
    /// - Publishes "unpaused" event
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let status: Status = inst.get(&KEY_STATUS).unwrap();
        if status != Status::Paused {
            return Err(ContractError::NotActive);
        }
        let admin: Address = inst.get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        inst.set(&KEY_STATUS, &Status::Active);
        env.events().publish(("campaign", "unpaused"), ());
        env.events().publish(
            ("campaign", "status_changed"),
            EventStatusChanged {
                old_status: Status::Paused,
                new_status: Status::Active,
            },
        );
        Ok(())
    }

    /// Sets up a recurring contribution plan for a contributor.
    ///
    /// Allows a contributor to schedule automatic contributions at regular intervals.
    /// The contributor must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address (must authorize)
    /// * `amount` - Amount to contribute each interval in stroops
    /// * `interval` - Interval in seconds between contributions
    /// * `end_date` - Unix timestamp when recurring contributions should stop
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidRecurringPlan)` if parameters are invalid
    ///
    /// # Side Effects
    /// - Stores recurring plan in persistent storage
    /// - Publishes "recurring_setup" event
    pub fn setup_recurring(
        env: Env,
        contributor: Address,
        amount: i128,
        interval: u64,
        end_date: u64,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        validate_recurring_plan(amount, interval, end_date, env.ledger().timestamp())?;

        let plan = RecurringPlan {
            amount,
            interval,
            end_date,
            last_executed: env.ledger().timestamp(),
        };

        let key = DataKey::RecurringPlan(contributor.clone());
        env.storage().persistent().set(&key, &plan);
        env.storage().persistent().extend_ttl(&key, 100, 100);

        env.events().publish(
            ("campaign", "recurring_setup"),
            EventRecurringSetup {
                contributor,
                amount,
                interval,
                end_date,
            },
        );
        Ok(())
    }

    /// Executes pending recurring contributions for a contributor.
    ///
    /// Can be called by anyone to trigger scheduled contributions.
    /// Only executes if the interval has passed since last execution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidRecurringPlan)` if no plan exists or plan expired
    pub fn execute_recurring(env: Env, contributor: Address) -> Result<(), ContractError> {
        let key = DataKey::RecurringPlan(contributor.clone());
        let mut plan: RecurringPlan = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        let now = env.ledger().timestamp();
        if now > plan.end_date {
            return Err(ContractError::InvalidRecurringPlan);
        }
        if now < plan.last_executed + plan.interval {
            return Err(ContractError::InvalidRecurringPlan);
        }

        plan.last_executed = now;
        env.storage().persistent().set(&key, &plan);

        // Cache instance storage handle
        let inst = env.storage().instance();
        let token: Address = inst.get(&KEY_TOKEN).unwrap();
        token::Client::new(&env, &token).transfer(
            &contributor,
            &env.current_contract_address(),
            &plan.amount,
        );

        let contrib_key = DataKey::Contribution(contributor.clone());
        let prev: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        let new_amount = prev
            .checked_add(plan.amount)
            .ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&contrib_key, &new_amount);

        let total: i128 = inst.get(&KEY_TOTAL).unwrap();
        inst.set(
            &KEY_TOTAL,
            &total
                .checked_add(plan.amount)
                .ok_or(ContractError::Overflow)?,
        );

        env.events().publish(
            ("campaign", "recurring_executed"),
            EventRecurringExecuted {
                contributor,
                amount: plan.amount,
            },
        );
        Ok(())
    }

    /// Cancels a recurring contribution plan.
    ///
    /// The contributor must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn cancel_recurring(env: Env, contributor: Address) -> Result<(), ContractError> {
        contributor.require_auth();
        let key = DataKey::RecurringPlan(contributor.clone());
        env.storage().persistent().remove(&key);
        env.events().publish(
            ("campaign", "recurring_cancelled"),
            EventRecurringCancelled { contributor },
        );
        Ok(())
    }

    /// Proposes a deadline extension and initiates voting.
    ///
    /// Only the creator can propose extensions. Voting period is 7 days.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_deadline` - Proposed new deadline (Unix timestamp)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidDeadline)` if new_deadline <= current_deadline
    pub fn propose_extension(env: Env, new_deadline: u64) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let current_deadline: u64 = inst.get(&KEY_DEADLINE).unwrap();
        if new_deadline <= current_deadline {
            return Err(ContractError::InvalidDeadline);
        }

        let now = env.ledger().timestamp();
        let voting_ends_at = now + 604800; // 7 days
        let proposal = ExtensionProposal {
            new_deadline,
            votes_for: 0,
            votes_against: 0,
            created_at: now,
            voting_ends_at,
            executed: false,
        };

        inst.set(&DataKey::ExtensionProposal, &proposal);
        env.events().publish(
            ("campaign", "extension_proposed"),
            EventExtensionProposed {
                new_deadline,
                voting_ends_at,
            },
        );
        Ok(())
    }

    /// Votes on a pending deadline extension.
    ///
    /// Contributors vote with weight equal to their contribution amount.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    /// * `approve` - true to vote for, false to vote against
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::VotingEnded)` if voting period has ended
    pub fn vote_on_extension(
        env: Env,
        contributor: Address,
        approve: bool,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        let inst = env.storage().instance();
        let mut proposal: ExtensionProposal = inst
            .get(&DataKey::ExtensionProposal)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        if env.ledger().timestamp() > proposal.voting_ends_at {
            return Err(ContractError::VotingEnded);
        }

        let vote_weight: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contributor.clone()))
            .unwrap_or(0);

        if approve {
            proposal.votes_for = proposal
                .votes_for
                .checked_add(vote_weight)
                .ok_or(ContractError::Overflow)?;
        } else {
            proposal.votes_against = proposal
                .votes_against
                .checked_add(vote_weight)
                .ok_or(ContractError::Overflow)?;
        }

        inst.set(&DataKey::ExtensionProposal, &proposal);
        inst.set(&DataKey::ExtensionVote(contributor.clone()), &approve);

        env.events().publish(
            ("campaign", "extension_voted"),
            EventExtensionVoted {
                contributor,
                approve,
                vote_weight,
            },
        );
        Ok(())
    }

    /// Executes a deadline extension if voting threshold is met.
    ///
    /// Requires >50% of votes to be in favor. Can only be called after voting period ends.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn execute_extension(env: Env) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let mut proposal: ExtensionProposal = inst
            .get(&DataKey::ExtensionProposal)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        if env.ledger().timestamp() <= proposal.voting_ends_at {
            return Err(ContractError::VotingEnded);
        }

        if proposal.executed {
            return Ok(());
        }

        let total_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .ok_or(ContractError::Overflow)?;
        if total_votes > 0 && proposal.votes_for * 2 > total_votes {
            inst.set(&KEY_DEADLINE, &proposal.new_deadline);
            env.events().publish(
                ("campaign", "extension_executed"),
                EventExtensionExecuted {
                    new_deadline: proposal.new_deadline,
                    votes_for: proposal.votes_for,
                    votes_against: proposal.votes_against,
                },
            );
        }

        proposal.executed = true;
        inst.set(&DataKey::ExtensionProposal, &proposal);
        Ok(())
    }

    /// Allows a contributor to request a partial refund before campaign ends.
    ///
    /// Limited to 50% of original contribution. Contributor must authorize.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    /// * `amount` - Amount to refund in stroops
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::RefundLimitExceeded)` if amount > 50% of contribution
    pub fn refund_partial(
        env: Env,
        contributor: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        let contrib_key = DataKey::Contribution(contributor.clone());
        let total_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);

        validate_partial_refund(amount, total_contrib)?;

        let inst = env.storage().instance();
        let token: Address = inst.get(&KEY_TOKEN).unwrap();
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &contributor,
            &amount,
        );

        let remaining = total_contrib - amount;
        env.storage().persistent().set(&contrib_key, &remaining);

        let total: i128 = inst.get(&KEY_TOTAL).unwrap();
        inst.set(&KEY_TOTAL, &(total - amount));

        env.events().publish(
            ("campaign", "partial_refund"),
            EventPartialRefund {
                contributor,
                amount,
                remaining,
            },
        );
        Ok(())
    }

    // ── Whitelist/Blacklist Functions ─────────────────────────────────────────

    /// Adds an address to the whitelist (creator only).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to whitelist
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn add_to_whitelist(env: Env, address: Address) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Whitelist(address.clone()), &true);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Whitelist(address.clone()), 100, 100);
        env.events()
            .publish(("campaign", "whitelisted"), EventWhitelisted { address });
        Ok(())
    }

    /// Removes an address from the whitelist (creator only).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to remove from whitelist
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn remove_from_whitelist(env: Env, address: Address) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage()
            .persistent()
            .remove(&DataKey::Whitelist(address.clone()));
        env.events().publish(
            ("campaign", "whitelist_removed"),
            EventWhitelistRemoved { address },
        );
        Ok(())
    }

    /// Adds an address to the blacklist (creator only).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to blacklist
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn add_to_blacklist(env: Env, address: Address) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Blacklist(address.clone()), &true);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Blacklist(address.clone()), 100, 100);
        env.events()
            .publish(("campaign", "blacklisted"), EventBlacklisted { address });
        Ok(())
    }

    /// Removes an address from the blacklist (creator only).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to remove from blacklist
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn remove_from_blacklist(env: Env, address: Address) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage()
            .persistent()
            .remove(&DataKey::Blacklist(address.clone()));
        env.events().publish(
            ("campaign", "blacklist_removed"),
            EventBlacklistRemoved { address },
        );
        Ok(())
    }

    /// Enables whitelist-only mode (creator only).
    ///
    /// When enabled, only whitelisted addresses can contribute.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `enabled` - true to enable, false to disable
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn set_whitelist_only(env: Env, enabled: bool) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::WhitelistOnly, &enabled);
        env.events().publish(
            ("campaign", "whitelist_only_set"),
            EventWhitelistOnlySet { enabled },
        );
        Ok(())
    }

    /// Checks if an address is whitelisted.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to check
    ///
    /// # Returns
    /// true if whitelisted, false otherwise
    pub fn is_whitelisted(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&DataKey::Whitelist(address))
            .unwrap_or(false)
    }

    /// Checks if an address is blacklisted.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - Address to check
    ///
    /// # Returns
    /// true if blacklisted, false otherwise
    pub fn is_blacklisted(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&DataKey::Blacklist(address))
            .unwrap_or(false)
    }

    // ── Visibility Controls ───────────────────────────────────────────────────

    /// Sets the campaign visibility level (creator only).
    ///
    /// `Public` and `Unlisted` allow anyone to contribute; `Private` restricts
    /// contributions to whitelisted addresses (orthogonal to the legacy
    /// `whitelist_only` flag — either being on requires the contributor to be
    /// whitelisted). The `Unlisted` variant signals that the campaign should
    /// not appear in public discovery feeds but does not restrict contributions.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `visibility` - New visibility level
    pub fn set_visibility(env: Env, visibility: Visibility) -> Result<(), ContractError> {
        let inst = env.storage().instance();
        let creator: Address = inst.get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let old: Visibility = inst.get(&KEY_VISIBILITY).unwrap_or(Visibility::Public);
        inst.set(&KEY_VISIBILITY, &visibility);
        env.events().publish(
            ("campaign", "visibility_changed"),
            EventVisibilityChanged {
                old_visibility: old,
                new_visibility: visibility,
            },
        );
        Ok(())
    }

    /// Returns the campaign's current visibility level.
    pub fn get_visibility(env: Env) -> Visibility {
        env.storage()
            .instance()
            .get(&KEY_VISIBILITY)
            .unwrap_or(Visibility::Public)
    }

    // ── Delegation Functions ──────────────────────────────────────────────────

    /// Delegates contribution authority to another address (delegator must authorize).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `delegator` - The address delegating authority (must authorize)
    /// * `delegate` - The address receiving delegation authority
    /// * `amount` - Maximum amount the delegate can contribute on behalf of delegator
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn delegate_contribution(
        env: Env,
        delegator: Address,
        delegate: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        delegator.require_auth();

        validate_delegation(amount)?;

        let delegation = Delegation {
            amount,
            delegate: delegate.clone(),
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Delegation(delegator.clone()), &delegation);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Delegation(delegator.clone()), 100, 100);
        env.events().publish(
            ("campaign", "delegation_created"),
            EventDelegationCreated {
                delegator,
                delegate,
                amount,
            },
        );
        Ok(())
    }

    /// Contributes on behalf of a delegator (delegate must authorize).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `delegator` - The address on whose behalf the contribution is made
    /// * `delegate` - The delegate address (must authorize)
    /// * `amount` - Contribution amount in stroops
    /// * `token` - Token address
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn contribute_on_behalf(
        env: Env,
        delegator: Address,
        delegate: Address,
        amount: i128,
        token: Address,
    ) -> Result<(), ContractError> {
        delegate.require_auth();

        let delegation: Delegation = env
            .storage()
            .persistent()
            .get(&DataKey::Delegation(delegator.clone()))
            .ok_or(ContractError::DelegationNotFound)?;

        if !delegation.active || delegation.delegate != delegate {
            return Err(ContractError::InvalidDelegation);
        }

        let delegated_key = DataKey::DelegatedContribution(delegator.clone());
        let delegated_so_far: i128 = env.storage().persistent().get(&delegated_key).unwrap_or(0);
        if delegated_so_far + amount > delegation.amount {
            return Err(ContractError::ExceedsMaximum);
        }

        // Perform the contribution as if delegator is contributing
        let min: i128 = env.storage().instance().get(&KEY_MIN).unwrap();
        if amount < min {
            return Err(ContractError::BelowMinimum);
        }

        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }

        let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        if env.ledger().timestamp() >= deadline {
            return Err(ContractError::CampaignEnded);
        }

        // Check whitelist/blacklist
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Blacklist(delegator.clone()))
            .unwrap_or(false)
        {
            return Err(ContractError::Blacklisted);
        }

        let whitelist_only: bool = env
            .storage()
            .instance()
            .get(&DataKey::WhitelistOnly)
            .unwrap_or(false);
        let visibility: Visibility = env
            .storage()
            .instance()
            .get(&KEY_VISIBILITY)
            .unwrap_or(Visibility::Public);
        if (whitelist_only || visibility == Visibility::Private)
            && !env
                .storage()
                .persistent()
                .get::<_, bool>(&DataKey::Whitelist(delegator.clone()))
                .unwrap_or(false)
        {
            return Err(ContractError::NotWhitelisted);
        }

        token::Client::new(&env, &token).transfer(
            &delegate,
            &env.current_contract_address(),
            &amount,
        );

        let key = DataKey::Contribution(delegator.clone());
        let prev: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_amount = prev.checked_add(amount).ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&key, &new_amount);
        env.storage().persistent().extend_ttl(&key, 100, 100);

        env.storage()
            .persistent()
            .set(&delegated_key, &(delegated_so_far + amount));
        env.storage()
            .persistent()
            .extend_ttl(&delegated_key, 100, 100);

        let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
        let new_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;
        env.storage().instance().set(&KEY_TOTAL, &new_total);

        let presence_key = DataKey::ContributorPresence(delegator.clone());
        let is_present: bool = env
            .storage()
            .persistent()
            .get(&presence_key)
            .unwrap_or(false);
        if !is_present {
            env.storage().persistent().set(&presence_key, &true);
            env.storage()
                .persistent()
                .extend_ttl(&presence_key, 100, 100);
            let count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::ContributorCount)
                .unwrap();
            env.storage()
                .instance()
                .set(&DataKey::ContributorCount, &(count + 1));
        }

        env.events().publish(
            ("campaign", "delegated_contribution"),
            EventDelegatedContribution {
                delegator,
                delegate,
                amount,
            },
        );
        Ok(())
    }

    /// Revokes a delegation (delegator must authorize).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `delegator` - The delegator address (must authorize)
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn revoke_delegation(env: Env, delegator: Address) -> Result<(), ContractError> {
        delegator.require_auth();

        let mut delegation: Delegation = env
            .storage()
            .persistent()
            .get(&DataKey::Delegation(delegator.clone()))
            .ok_or(ContractError::DelegationNotFound)?;

        delegation.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Delegation(delegator.clone()), &delegation);
        env.events().publish(
            ("campaign", "delegation_revoked"),
            EventDelegationRevoked { delegator },
        );
        Ok(())
    }

    /// Gets delegation info for an address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `delegator` - The delegator address
    ///
    /// # Returns
    /// Optional Delegation info
    pub fn get_delegation(env: Env, delegator: Address) -> Option<Delegation> {
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(delegator))
    }

    // ── Template Functions ────────────────────────────────────────────────────

    /// Sets a campaign template (creator only).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `template_type` - The template type
    /// * `name` - Template name
    /// * `description` - Template description
    /// * `suggested_min` - Suggested minimum contribution
    /// * `goal_multiplier` - Goal multiplier in basis points
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn set_template(
        env: Env,
        template_type: TemplateType,
        name: String,
        description: String,
        suggested_min: i128,
        goal_multiplier: u32,
    ) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        validate_string_length(&name, 64)?;
        validate_string_length(&description, 512)?;

        let template = CampaignTemplate {
            template_type,
            name,
            description,
            suggested_min,
            goal_multiplier,
        };

        env.storage().instance().set(&DataKey::Template, &template);
        env.events().publish(("campaign", "template_set"), ());
        Ok(())
    }

    /// Gets the campaign template.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional CampaignTemplate
    pub fn get_template(env: Env) -> Option<CampaignTemplate> {
        env.storage().instance().get(&DataKey::Template)
    }

    // ── View functions ────────────────────────────────────────────────────────

    /// Returns the total amount raised so far in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Total raised amount (i128), or 0 if not yet initialized
    pub fn total_raised(env: Env) -> i128 {
        env.storage().instance().get(&KEY_TOTAL).unwrap_or(0)
    }

    /// Returns the campaign creator's Stellar address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Creator's address
    pub fn creator(env: Env) -> Address {
        env.storage().instance().get(&KEY_CREATOR).unwrap()
    }

    /// Returns the current campaign status.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Current Status (Active, Successful, Refunded, Cancelled, or Paused)
    pub fn status(env: Env) -> Status {
        env.storage().instance().get(&KEY_STATUS).unwrap()
    }

    /// Returns the campaign funding goal in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Goal amount (i128)
    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&KEY_GOAL).unwrap()
    }

    /// Returns the campaign deadline as a Unix timestamp (seconds).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Deadline timestamp (u64)
    pub fn deadline(env: Env) -> u64 {
        env.storage().instance().get(&KEY_DEADLINE).unwrap()
    }

    /// Returns the total contribution amount for a specific contributor in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address
    ///
    /// # Returns
    /// Total contribution amount (i128), or 0 if no contributions
    pub fn contribution(env: Env, contributor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0)
    }

    /// Checks if an address has made any contributions to the campaign.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if the address has contributed, false otherwise
    pub fn is_contributor(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::Contribution(address))
            .unwrap_or(0)
            > 0
    }

    /// Returns the minimum contribution amount in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Minimum contribution amount (i128)
    pub fn min_contribution(env: Env) -> i128 {
        env.storage().instance().get(&KEY_MIN).unwrap()
    }

    /// Returns the maximum contribution amount per contributor in stroops (0 = no limit).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Maximum contribution amount (i128), or 0 if no limit is set
    pub fn max_contribution(env: Env) -> i128 {
        env.storage().instance().get(&KEY_MAX).unwrap_or(0)
    }

    /// Returns the campaign title.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Campaign title string
    pub fn title(env: Env) -> String {
        env.storage()
            .instance()
            .get(&KEY_TITLE)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign description.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Campaign description string
    pub fn description(env: Env) -> String {
        env.storage()
            .instance()
            .get(&KEY_DESC)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign's social media links.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Vector of social media URLs
    pub fn social_links(env: Env) -> Vec<String> {
        env.storage()
            .instance()
            .get(&KEY_SOCIAL)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the list of accepted token addresses (whitelist).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Vector of accepted token addresses, or empty if no whitelist is set
    pub fn accepted_tokens(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::AcceptedTokens)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the platform fee configuration (if set).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional PlatformConfig containing address and fee_bps
    pub fn platform_config(env: Env) -> Option<PlatformConfig> {
        env.storage().instance().get(&KEY_PLATFORM)
    }

    /// Returns the contract version number.
    ///
    /// # Arguments
    /// * `_env` - The Soroban environment (unused)
    ///
    /// # Returns
    /// Contract version (u32)
    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    /// Returns comprehensive campaign statistics.
    ///
    /// Includes total raised, goal, progress percentage, contributor count,
    /// average contribution, and largest single contribution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// CampaignStats struct with all metrics
    ///
    /// # Progress Calculation
    /// progress_bps = (total_raised * 10_000) / goal, capped at 10_000 (100%)
    pub fn get_stats(env: Env) -> CampaignStats {
        let contributor_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ContributorCount)
            .unwrap_or(0);
        let largest_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LargestContribution)
            .unwrap_or(0);
        let total_raised: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();

        let progress_bps = if goal > 0 {
            let raw = (total_raised * 10_000) / goal;
            if raw > 10_000 {
                10_000
            } else {
                raw as u32
            }
        } else {
            0
        };

        let average_contribution = if contributor_count == 0 {
            0
        } else {
            total_raised / contributor_count as i128
        };

        CampaignStats {
            total_raised,
            goal,
            progress_bps,
            contributor_count,
            average_contribution,
            largest_contribution,
        }
    }

    /// Returns comprehensive campaign information.
    ///
    /// Includes creator, token, goal, deadline, minimum contribution, metadata,
    /// status, and platform fee configuration.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// CampaignInfo struct with all campaign details
    pub fn get_campaign_info(env: Env) -> CampaignInfo {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        let token: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();
        let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        let min_contribution: i128 = env.storage().instance().get(&KEY_MIN).unwrap();
        let max_contribution: i128 = env.storage().instance().get(&KEY_MAX).unwrap_or(0);
        let title: String = env
            .storage()
            .instance()
            .get(&KEY_TITLE)
            .unwrap_or_else(|| String::from_str(&env, ""));
        let description: String = env
            .storage()
            .instance()
            .get(&KEY_DESC)
            .unwrap_or_else(|| String::from_str(&env, ""));
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        let category: Category = env
            .storage()
            .instance()
            .get(&KEY_CATEGORY)
            .unwrap_or(Category::Other);

        let (has_platform_config, platform_fee_bps, platform_address) = if let Some(config) = env
            .storage()
            .instance()
            .get::<_, PlatformConfig>(&KEY_PLATFORM)
        {
            (true, config.fee_bps, config.address)
        } else {
            (false, 0, creator.clone())
        };

        CampaignInfo {
            creator,
            token,
            goal,
            deadline,
            min_contribution,
            max_contribution,
            title,
            description,
            status,
            has_platform_config,
            platform_fee_bps,
            platform_address,
            category,
        }
    }

    /// Returns the campaign category.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Campaign category
    pub fn get_category(env: Env) -> Category {
        env.storage()
            .instance()
            .get(&KEY_CATEGORY)
            .unwrap_or(Category::Other)
    }

    /// Returns the vesting schedule (if configured).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional VestingSchedule with cliff and duration
    pub fn get_vesting_info(env: Env) -> Option<VestingSchedule> {
        env.storage().instance().get(&KEY_VESTING)
    }

    /// Returns the goal adjustment history.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Vector of GoalAdjustment entries
    pub fn get_goal_history(env: Env) -> Vec<GoalAdjustment> {
        env.storage()
            .persistent()
            .get(&KEY_GOAL_HISTORY)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the penalty fee in basis points (if configured).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Penalty fee in basis points, or 0 if not configured
    pub fn get_penalty_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PenaltyBps)
            .unwrap_or(0)
    }

    /// Returns a paginated list of contributor addresses.
    ///
    /// Useful for iterating through all contributors without loading the entire list.
    /// The limit is capped at 50 to prevent excessive memory usage.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `offset` - Starting index in the contributor list (0-based)
    /// * `limit` - Maximum number of contributors to return (capped at 50)
    ///
    /// # Returns
    /// Vector of contributor addresses for the requested page
    ///
    /// # Example
    /// ```ignore
    /// // Get first 10 contributors
    /// let page1 = contributor_list(env, 0, 10);
    /// // Get next 10 contributors
    /// let page2 = contributor_list(env, 10, 10);
    /// ```
    pub fn contributor_list(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&KEY_CONTRIBS)
            .unwrap_or_else(|| Vec::new(&env));

        let total_count = contributors.len();
        if offset >= total_count {
            return Vec::new(&env);
        }

        let capped_limit = if limit > 50 { 50 } else { limit };
        let end = (offset + capped_limit).min(total_count);

        let mut result = Vec::new(&env);
        for i in offset..end {
            result.push_back(contributors.get(i).unwrap());
        }
        result
    }

    /// Returns the contribution message for a contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// Optional message string, or None if no message was provided
    pub fn get_contribution_message(env: Env, contributor: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::ContributionMessage(contributor))
    }

    /// Returns the recurring plan for a contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// Optional RecurringPlan, or None if no plan exists
    pub fn get_recurring_plan(env: Env, contributor: Address) -> Option<RecurringPlan> {
        env.storage()
            .persistent()
            .get(&DataKey::RecurringPlan(contributor))
    }

    /// Returns the current extension proposal.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional ExtensionProposal, or None if no proposal exists
    pub fn get_extension_proposal(env: Env) -> Option<ExtensionProposal> {
        env.storage().instance().get(&DataKey::ExtensionProposal)
    }

    /// Enables insurance for the campaign.
    ///
    /// Allows the creator to set up optional insurance protection for contributors.
    /// Insurance fees are collected during contributions and held in a pool.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `fee_bps` - Insurance fee in basis points (e.g., 100 = 1%)
    /// * `provider` - Address of the insurance provider
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::Unauthorized)` if caller is not the creator
    /// * `Err(ContractError::InvalidFee)` if fee_bps > 10,000
    pub fn enable_insurance(
        env: Env,
        fee_bps: u32,
        provider: Address,
    ) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        validate_fee_bps(fee_bps)?;
        validate_address_not_self(&creator, &provider)?;

        let config = InsuranceConfig {
            fee_bps,
            provider: provider.clone(),
            enabled: true,
        };

        let inst = env.storage().instance();
        inst.set(&KEY_INSURANCE, &config);
        inst.set(&KEY_INSURANCE_POOL, &0i128);
        env.events().publish(
            ("insurance", "enabled"),
            EventInsuranceEnabled { fee_bps, provider },
        );
        Ok(())
    }

    /// Returns the insurance configuration if enabled.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional InsuranceConfig, or None if insurance is not enabled
    pub fn get_insurance_config(env: Env) -> Option<InsuranceConfig> {
        env.storage().instance().get(&KEY_INSURANCE)
    }

    /// Returns the total insurance pool amount.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Total insurance fees collected in stroops
    pub fn get_insurance_pool(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&KEY_INSURANCE_POOL)
            .unwrap_or(0)
    }

    /// Returns the insurance fee paid by a contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// Insurance fee amount in stroops, or 0 if no insurance fee paid
    pub fn get_insurance_fee(env: Env, contributor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::InsuranceFee(contributor))
            .unwrap_or(0)
    }

    /// Pays out insurance to a contributor on campaign failure.
    ///
    /// Called internally when a campaign fails and insurance is enabled.
    /// Transfers insurance payout from the pool to the contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InsufficientFunds)` if pool is empty
    fn process_insurance_payout(env: Env, contributor: Address) -> Result<(), ContractError> {
        let insurance_fee: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::InsuranceFee(contributor.clone()))
            .unwrap_or(0);

        if insurance_fee == 0 {
            return Ok(());
        }

        let mut pool: i128 = env
            .storage()
            .instance()
            .get(&KEY_INSURANCE_POOL)
            .unwrap_or(0);
        if pool < insurance_fee {
            return Err(ContractError::InsufficientFunds);
        }

        pool = pool
            .checked_sub(insurance_fee)
            .ok_or(ContractError::Overflow)?;
        env.storage().instance().set(&KEY_INSURANCE_POOL, &pool);
        env.events().publish(
            ("insurance", "payout"),
            EventInsurancePayout {
                contributor,
                amount: insurance_fee,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod test;
