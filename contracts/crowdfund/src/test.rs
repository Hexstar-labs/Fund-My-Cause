#![cfg(test)]
#![allow(deprecated)]

use super::*;
use crate::types::Category;
use crate::{CrowdfundContract, CrowdfundContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup_contract(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &0i128,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn initialize_and_contribute_updates_state() {
    let env = Env::default();
    let deadline = 1_000u64;
    let goal = 10_000i128;
    let min_contribution = 100i128;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, goal, min_contribution);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);

    assert_eq!(client.total_raised(), 500);
    assert_eq!(client.contribution(&contributor), 500);
    assert!(client.is_contributor(&contributor));

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 500);
    assert_eq!(stats.goal, goal);
    assert_eq!(stats.contributor_count, 1);
    assert_eq!(stats.average_contribution, 500);
    assert_eq!(stats.largest_contribution, 500);
}

#[test]
fn cancel_allows_refund_before_deadline() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 10_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    env.ledger().set_timestamp(deadline - 10);
    client.refund_single(&contributor);

    assert_eq!(client.contribution(&contributor), 0);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn invalid_platform_fee_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &Some(PlatformConfig {
            address: Address::generate(&env),
            fee_bps: 10_001,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );

    assert_eq!(result.err(), Some(Ok(ContractError::InvalidFee)));
}

// ── Boundary tests (#107) ─────────────────────────────────────────────────────

#[test]
fn accepted_token_whitelist_is_enforced() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let allowed_token = env.register_stellar_asset_contract(token_admin.clone());
    let other_token = env.register_stellar_asset_contract(token_admin);
    let allowed_token_admin = token::StellarAssetClient::new(&env, &allowed_token);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let mut accepted_tokens = Vec::new(&env);
    accepted_tokens.push_back(allowed_token.clone());

    client.initialize(
        &creator,
        &allowed_token,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &None,
        &None,
        &Some(accepted_tokens),
        &Category::Other,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    allowed_token_admin.mint(&contributor, &100);

    let result = client.try_contribute(&contributor, &100, &other_token, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::TokenNotAccepted)));
}

// ── refund_batch tests (#278) ─────────────────────────────────────────────────

#[test]
fn refund_batch_refunds_multiple_contributors() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let token_client = token::Client::new(&env, &token_id);

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    let c3 = Address::generate(&env);

    token_admin_client.mint(&c1, &500);
    token_admin_client.mint(&c2, &300);
    token_admin_client.mint(&c3, &200);

    client.contribute(&c1, &500, &token_id, &None);
    client.contribute(&c2, &300, &token_id, &None);
    client.contribute(&c3, &200, &token_id, &None);

    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    batch.push_back(c2.clone());
    batch.push_back(c3.clone());

    let refunded = client.refund_batch(&batch);
    assert_eq!(refunded, 3);

    assert_eq!(token_client.balance(&c1), 500);
    assert_eq!(token_client.balance(&c2), 300);
    assert_eq!(token_client.balance(&c3), 200);
    assert_eq!(client.contribution(&c1), 0);
    assert_eq!(client.contribution(&c2), 0);
    assert_eq!(client.contribution(&c3), 0);
}

#[test]
fn refund_batch_skips_already_refunded() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);
    client.cancel_campaign();

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());
    let r1 = client.refund_batch(&batch);
    assert_eq!(r1, 1);

    let r2 = client.refund_batch(&batch);
    assert_eq!(r2, 0);
}

#[test]
fn refund_batch_fails_when_campaign_still_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let c1 = Address::generate(&env);
    token_admin_client.mint(&c1, &500);
    client.contribute(&c1, &500, &token_id, &None);

    let mut batch = Vec::new(&env);
    batch.push_back(c1.clone());

    let result = client.try_refund_batch(&batch);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignStillActive)));
}

// ── pause/unpause tests (#279) ────────────────────────────────────────────────

#[test]
fn pause_blocks_contributions_and_unpause_resumes() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    client.pause();
    assert_eq!(client.status(), Status::Paused);

    let result = client.try_contribute(&contributor, &500, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::CampaignPaused)));

    client.unpause();
    assert_eq!(client.status(), Status::Active);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.total_raised(), 500);
}

#[test]
fn pause_allows_refunds_when_cancelled() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 100_000, 100);

    let contributor = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    client.cancel_campaign();

    client.refund_single(&contributor);
    assert_eq!(token_client.balance(&contributor), 500);
}

#[test]
fn unpause_fails_when_not_paused() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    let result = client.try_unpause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

#[test]
fn pause_fails_when_not_active() {
    let env = Env::default();
    let deadline = 1_000u64;

    let (_creator, _token_id, client, _) = setup_contract(&env, deadline, 100_000, 100);

    client.cancel_campaign();

    let result = client.try_pause();
    assert_eq!(result.err(), Some(Ok(ContractError::NotActive)));
}

// ── max_contribution tests ────────────────────────────────────────────────────

fn setup_contract_with_max(
    env: &Env,
    deadline: u64,
    goal: i128,
    min_contribution: i128,
    max_contribution: i128,
) -> (
    Address,
    Address,
    CrowdfundContractClient<'_>,
    token::StellarAssetClient<'_>,
) {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &max_contribution,
        &String::from_str(env, "My Title"),
        &String::from_str(env, "My Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (creator, token_id, client, token_admin_client)
}

#[test]
fn contribute_within_max_succeeds() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 500);
}

#[test]
fn contribute_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    let result = client.try_contribute(&contributor, &600, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn cumulative_contribution_exceeding_max_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract_with_max(&env, 1_000, 10_000, 100, 500);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &600);

    client.contribute(&contributor, &300, &token_id, &None);
    let result = client.try_contribute(&contributor, &300, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn no_max_limit_allows_large_contribution() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) = setup_contract(&env, 1_000, 10_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &9_000);

    client.contribute(&contributor, &9_000, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 9_000);
}

#[test]
fn max_contribution_view_returns_stored_value() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract_with_max(&env, 1_000, 10_000, 100, 750);

    assert_eq!(client.max_contribution(), 750);
}

#[test]
fn initialize_with_max_below_min_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &10_000,
        &1_000,
        &200,
        &100, // max < min — invalid
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Desc"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

// ── Input validation tests ────────────────────────────────────────────────────

#[test]
fn initialize_with_empty_title_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, ""), // empty title
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn initialize_with_title_too_long_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    // 65-character title (max is 64)
    let long_title = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &long_title,
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::StringTooLong)));
}

#[test]
fn initialize_with_self_fee_address_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let result = client.try_initialize(
        &creator,
        &token_id,
        &1_000,
        &1_000,
        &10,
        &0i128,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &Some(PlatformConfig {
            address: creator.clone(), // same as creator — invalid
            fee_bps: 100,
        }),
        &None,
        &Category::Other,
        &None,
        &None,
    );
    assert_eq!(result.err(), Some(Ok(ContractError::SelfFeeAddress)));
}

#[test]
fn contribute_with_zero_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &0, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn contribute_with_negative_amount_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let contributor = Address::generate(&env);
    let result = client.try_contribute(&contributor, &-1, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::AmountNotPositive)));
}

#[test]
fn update_metadata_with_empty_title_is_rejected() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    let result = client.try_update_metadata(&Some(String::from_str(&env, "")), &None, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::StringEmpty)));
}

#[test]
fn update_metadata_with_valid_title_succeeds() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.update_metadata(&Some(String::from_str(&env, "New Title")), &None, &None);
    assert_eq!(client.title(), String::from_str(&env, "New Title"));
}

// ── Rate limit tests (#428) ───────────────────────────────────────────────────

#[test]
fn set_rate_limit_stores_struct_and_get_returns_it() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    client.set_rate_limit(&500, &3_600);
    let rl = client.get_rate_limit().expect("rate limit set");
    assert_eq!(rl.max_amount, 500);
    assert_eq!(rl.window_seconds, 3_600);
}

#[test]
fn set_rate_limit_zero_clears_existing_limit() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    client.set_rate_limit(&500, &3_600);
    assert!(client.get_rate_limit().is_some());

    client.set_rate_limit(&0, &0);
    assert!(client.get_rate_limit().is_none());
}

#[test]
fn set_rate_limit_rejects_negative_amount() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let result = client.try_set_rate_limit(&-1, &3_600);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidRateLimit)));
}

#[test]
fn set_rate_limit_rejects_zero_window_when_enabling() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 0);

    let result = client.try_set_rate_limit(&500, &0);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidRateLimit)));
}

#[test]
fn contribute_exceeding_rate_limit_in_window_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    client.set_rate_limit(&500, &3_600);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    env.ledger().set_timestamp(100);
    client.contribute(&contributor, &400, &token_id, &None);

    // Second contribution in same window would exceed 500
    let result = client.try_contribute(&contributor, &200, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::RateLimitExceeded)));
    // First contribution still counted; second rolled back
    assert_eq!(client.contribution(&contributor), 400);
}

#[test]
fn contribute_within_rate_limit_succeeds() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    client.set_rate_limit(&500, &3_600);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    env.ledger().set_timestamp(100);
    client.contribute(&contributor, &300, &token_id, &None);
    client.contribute(&contributor, &200, &token_id, &None);

    assert_eq!(client.contribution(&contributor), 500);
}

#[test]
fn rate_limit_resets_after_window_elapses() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 100_000, 1_000_000, 100);

    client.set_rate_limit(&500, &3_600);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &2_000);

    env.ledger().set_timestamp(1_000);
    client.contribute(&contributor, &500, &token_id, &None);

    // Advance past the window — the per-address counter should reset
    env.ledger().set_timestamp(1_000 + 3_601);
    client.contribute(&contributor, &500, &token_id, &None);

    assert_eq!(client.contribution(&contributor), 1_000);
}

#[test]
fn disabled_rate_limit_allows_large_contributions() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 100_000, 1_000_000, 100);

    // No rate limit configured by default
    assert!(client.get_rate_limit().is_none());

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &50_000);
    client.contribute(&contributor, &50_000, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 50_000);
}

// ── Visibility control tests (#429) ───────────────────────────────────────────

#[test]
fn default_visibility_is_public() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);
    assert_eq!(client.get_visibility(), Visibility::Public);
}

#[test]
fn set_visibility_updates_storage() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000, 10_000, 100);

    client.set_visibility(&Visibility::Unlisted);
    assert_eq!(client.get_visibility(), Visibility::Unlisted);

    client.set_visibility(&Visibility::Private);
    assert_eq!(client.get_visibility(), Visibility::Private);
}

#[test]
fn private_visibility_blocks_non_whitelisted_contributors() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    client.set_visibility(&Visibility::Private);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);

    let result = client.try_contribute(&contributor, &500, &token_id, &None);
    assert_eq!(result.err(), Some(Ok(ContractError::NotWhitelisted)));
}

#[test]
fn private_visibility_allows_whitelisted_contributors() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    let contributor = Address::generate(&env);
    client.set_visibility(&Visibility::Private);
    client.add_to_whitelist(&contributor);

    token_admin_client.mint(&contributor, &500);
    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 500);
}

#[test]
fn unlisted_visibility_allows_any_contributor() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    client.set_visibility(&Visibility::Unlisted);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500);
    client.contribute(&contributor, &500, &token_id, &None);
    assert_eq!(client.contribution(&contributor), 500);
}

// ── Delegation tests (#430) ───────────────────────────────────────────────────

#[test]
fn delegate_contribution_then_contribute_on_behalf() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    token_admin_client.mint(&delegate, &500);

    client.delegate_contribution(&delegator, &delegate, &500);

    let info = client.get_delegation(&delegator).expect("delegation stored");
    assert_eq!(info.amount, 500);
    assert_eq!(info.delegate, delegate);
    assert!(info.active);

    client.contribute_on_behalf(&delegator, &delegate, &300, &token_id);

    // Contribution is credited to the delegator, not the delegate
    assert_eq!(client.contribution(&delegator), 300);
    assert_eq!(client.contribution(&delegate), 0);
    assert_eq!(client.total_raised(), 300);
}

#[test]
fn contribute_on_behalf_rejects_exceeding_delegated_amount() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    token_admin_client.mint(&delegate, &1_000);

    client.delegate_contribution(&delegator, &delegate, &500);
    client.contribute_on_behalf(&delegator, &delegate, &400, &token_id);

    let result = client.try_contribute_on_behalf(&delegator, &delegate, &200, &token_id);
    assert_eq!(result.err(), Some(Ok(ContractError::ExceedsMaximum)));
}

#[test]
fn revoke_delegation_blocks_further_contributions() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    token_admin_client.mint(&delegate, &500);

    client.delegate_contribution(&delegator, &delegate, &500);
    client.revoke_delegation(&delegator);

    let info = client.get_delegation(&delegator).expect("still stored");
    assert!(!info.active);

    let result = client.try_contribute_on_behalf(&delegator, &delegate, &200, &token_id);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidDelegation)));
}

#[test]
fn contribute_on_behalf_without_delegation_is_rejected() {
    let env = Env::default();
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, 10_000, 100_000, 100);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);
    token_admin_client.mint(&delegate, &500);

    let result = client.try_contribute_on_behalf(&delegator, &delegate, &200, &token_id);
    assert_eq!(result.err(), Some(Ok(ContractError::DelegationNotFound)));
}

#[test]
fn delegate_contribution_rejects_non_positive_amount() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 10_000, 100_000, 100);

    let delegator = Address::generate(&env);
    let delegate = Address::generate(&env);

    let result = client.try_delegate_contribution(&delegator, &delegate, &0);
    assert_eq!(result.err(), Some(Ok(ContractError::InvalidDelegation)));
}

// ── Recurring contribution tests (#431) ───────────────────────────────────────

#[test]
fn setup_and_execute_recurring_contribution() {
    let env = Env::default();
    let deadline = 1_000_000u64;
    let (_creator, token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 1_000_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &10_000);

    env.ledger().set_timestamp(1_000);
    let interval = 3_600u64;
    let end_date = 100_000u64;
    client.setup_recurring(&contributor, &500, &interval, &end_date);

    let plan = client.get_recurring_plan(&contributor).expect("plan stored");
    assert_eq!(plan.amount, 500);
    assert_eq!(plan.interval, interval);
    assert_eq!(plan.end_date, end_date);

    // Not enough time has passed yet
    let too_early = client.try_execute_recurring(&contributor);
    assert_eq!(too_early.err(), Some(Ok(ContractError::InvalidRecurringPlan)));

    env.ledger().set_timestamp(1_000 + interval + 1);
    client.execute_recurring(&contributor);
    assert_eq!(client.contribution(&contributor), 500);

    // Cannot execute again immediately
    let again = client.try_execute_recurring(&contributor);
    assert_eq!(again.err(), Some(Ok(ContractError::InvalidRecurringPlan)));

    // After another interval passes, a second execution succeeds
    env.ledger().set_timestamp(1_000 + 2 * interval + 2);
    client.execute_recurring(&contributor);
    assert_eq!(client.contribution(&contributor), 1_000);
}

#[test]
fn setup_recurring_rejects_invalid_parameters() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000_000, 1_000_000, 100);

    let contributor = Address::generate(&env);
    env.ledger().set_timestamp(1_000);

    // amount must be > 0
    let r = client.try_setup_recurring(&contributor, &0, &3_600, &100_000);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidRecurringPlan)));

    // interval must be > 0
    let r = client.try_setup_recurring(&contributor, &500, &0, &100_000);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidRecurringPlan)));

    // end_date must be in the future
    let r = client.try_setup_recurring(&contributor, &500, &3_600, &500);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidRecurringPlan)));
}

#[test]
fn execute_recurring_after_end_date_is_rejected() {
    let env = Env::default();
    let deadline = 1_000_000u64;
    let (_creator, _token_id, client, token_admin_client) =
        setup_contract(&env, deadline, 1_000_000, 100);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);

    env.ledger().set_timestamp(1_000);
    client.setup_recurring(&contributor, &500, &3_600, &10_000);

    env.ledger().set_timestamp(20_000);
    let r = client.try_execute_recurring(&contributor);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidRecurringPlan)));
}

#[test]
fn cancel_recurring_removes_plan() {
    let env = Env::default();
    let (_creator, _token_id, client, _) = setup_contract(&env, 1_000_000, 1_000_000, 100);

    let contributor = Address::generate(&env);
    env.ledger().set_timestamp(1_000);
    client.setup_recurring(&contributor, &500, &3_600, &100_000);
    assert!(client.get_recurring_plan(&contributor).is_some());

    client.cancel_recurring(&contributor);
    assert!(client.get_recurring_plan(&contributor).is_none());

    let r = client.try_execute_recurring(&contributor);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidRecurringPlan)));
}
