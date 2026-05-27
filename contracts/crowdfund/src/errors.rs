/// Error types for the crowdfund contract.
///
/// This module defines all possible error conditions that can occur during contract execution.
use soroban_sdk::contracterror;

/// Contract error types.
///
/// Represents all possible error conditions that can occur during contract execution.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Campaign has already been initialized
    AlreadyInitialized = 1,
    /// Campaign deadline has passed
    CampaignEnded = 2,
    /// Campaign deadline has not yet passed
    CampaignStillActive = 3,
    /// Campaign goal was not reached
    GoalNotReached = 4,
    /// Campaign goal was already reached
    GoalReached = 5,
    /// Arithmetic overflow occurred
    Overflow = 6,
    /// Campaign is not in Active status
    NotActive = 7,
    /// Platform fee is invalid (> 10,000 bps)
    InvalidFee = 8,
    /// Amount is below minimum contribution
    BelowMinimum = 9,
    /// Deadline is invalid
    InvalidDeadline = 10,
    /// Campaign is paused
    CampaignPaused = 11,
    /// Campaign goal is invalid (<= 0)
    InvalidGoal = 12,
    /// Token is not accepted by this campaign
    TokenNotAccepted = 13,
    /// Contribution would exceed the per-contributor maximum
    ExceedsMaximum = 14,
    /// Address is not whitelisted
    NotWhitelisted = 15,
    /// Address is blacklisted
    Blacklisted = 16,
    /// Invalid delegation
    InvalidDelegation = 17,
    /// Delegation not found
    DelegationNotFound = 18,
    /// Invalid template
    InvalidTemplate = 19,
    /// Voting period has ended
    VotingEnded = 20,
    /// Invalid recurring plan
    InvalidRecurringPlan = 21,
    /// Refund limit exceeded
    RefundLimitExceeded = 22,
    /// Vesting not complete
    VestingNotComplete = 23,
    /// Emergency withdrawal locked
    EmergencyLocked = 24,
    /// Rate limit exceeded
    RateLimitExceeded = 25,
    /// Message too long
    MessageTooLong = 26,
    /// String field (title/description) is empty
    StringEmpty = 27,
    /// String field exceeds maximum allowed length
    StringTooLong = 28,
    /// Numeric amount must be positive (> 0)
    AmountNotPositive = 29,
    /// Platform fee address must differ from creator
    SelfFeeAddress = 30,
    /// Goal would overflow i128 when combined with existing totals
    GoalOverflow = 31,
    /// Insufficient funds in pool or balance
    InsufficientFunds = 32,
    /// Caller is not authorized for this operation
    Unauthorized = 33,
    /// Rate limit configuration is invalid (negative amount or zero window)
    InvalidRateLimit = 34,
}
