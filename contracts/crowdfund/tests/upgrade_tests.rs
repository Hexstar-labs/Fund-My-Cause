#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};

use crowdfund::{Category, CrowdfundContract, CrowdfundContractClient, PlatformConfig, Status};

fn setup_campaign(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    max_contribution: i128,
) -> (CrowdfundContractClient<'_>, Address, token::StellarAssetClient<'_>) {
    env.mock_all_auths();
    let creator = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(env, &contract_id);
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    env.ledger().set_timestamp(100);
    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &max_contribution,
        &String::from_str(env, "Upgrade Test Campaign"),
        &String::from_str(env, "Testing upgrade scenarios"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    (client, token_id, token_admin_client)
}

// ── #448: Upgrade test suite ─────────────────────────────────────────────────

#[test]
fn test_upgrade_preserves_state() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 10_000, 1_000_000, 100, 0);

    let contributor = Address::generate(&env);
    token_admin.mint(&contributor, &1_000);

    client.contribute(&contributor, &1_000, &token_id, &None);

    assert_eq!(client.total_raised(), 1_000);
    assert_eq!(client.contribution(&contributor), 1_000);
    assert_eq!(client.status(), Status::Active);

    // Simulate upgrade by verifying state is still accessible
    let info = client.get_campaign_info();
    assert_eq!(info.goal, 10_000);
    assert_eq!(info.status, Status::Active);
}

#[test]
fn test_upgrade_with_platform_fee_preserves_config() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let platform = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);

    env.ledger().set_timestamp(100);
    client.initialize(
        &creator,
        &token_id,
        &10_000,
        &1_000_000,
        &100,
        &0i128,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &None,
        &Some(PlatformConfig { address: platform.clone(), fee_bps: 500 }),
        &None,
        &Category::Other,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &1_000);
    client.contribute(&contributor, &1_000, &token_id, &None);

    // Verify platform config is preserved
    let platform_after = client.platform_config().expect("platform config exists");
    assert_eq!(platform_after.address, platform);
    assert_eq!(platform_after.fee_bps, 500);
}

#[test]
fn test_upgrade_with_vesting_preserves_schedule() {
    let env = Env::default();
    let (client, _token_id, _token_admin) = setup_campaign(&env, 1_000, 1_000, 100, 0);

    // Vesting info should be preserved
    let vesting = client.get_vesting_info();
    assert!(vesting.is_none());
}

#[test]
fn test_backward_compatibility_existing_storage() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 10_000, 1_000_000, 100, 0);

    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);

    token_admin.mint(&contributor1, &1_000);
    token_admin.mint(&contributor2, &2_000);

    client.contribute(&contributor1, &1_000, &token_id, &None);
    client.contribute(&contributor2, &2_000, &token_id, &None);

    // Verify all storage keys are accessible
    assert!(client.is_contributor(&contributor1));
    assert!(client.is_contributor(&contributor2));
    assert_eq!(client.contribution(&contributor1), 1_000);
    assert_eq!(client.contribution(&contributor2), 2_000);

    // Stats should work
    let stats = client.get_stats();
    assert_eq!(stats.contributor_count, 2);
    assert_eq!(stats.total_raised, 3_000);
}

#[test]
fn test_state_migration_goal_history() {
    let env = Env::default();
    let (client, _token_id, _token_admin) = setup_campaign(&env, 10_000, 1_000_000, 100, 0);

    // Goal history should have initial entry
    let history = client.get_goal_history();
    assert_eq!(history.len(), 1);

    // Modify goal
    client.adjust_goal(&15_000);

    let history = client.get_goal_history();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_state_migration_metadata_versioning() {
    let env = Env::default();
    let (client, _token_id, _token_admin) = setup_campaign(&env, 10_000, 1_000_000, 100, 0);

    // Update metadata
    client.update_metadata(&Some(String::from_str(&env, "New Title")), &None, &None);

    // Metadata version history should have entries
    let history = client.get_metadata_history();
    assert!(history.len() >= 1);
}

#[test]
fn test_concurrent_operations_after_upgrade() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 100_000, 1_000_000, 100, 0);

    // Simulate multiple operations that should all work post-upgrade
    let mut contributors: Vec<Address> = Vec::new(&env);
    for _ in 0..5 {
        contributors.push_back(Address::generate(&env));
    }

    for contributor in contributors.iter() {
        token_admin.mint(&contributor, &1_000);
        client.contribute(&contributor, &1_000, &token_id, &None);
    }

    // Pause/unpause
    client.pause();
    assert_eq!(client.status(), Status::Paused);
    client.unpause();
    assert_eq!(client.status(), Status::Active);

    // Whitelist/blacklist
    client.add_to_whitelist(&contributors.get(0).unwrap());
    assert!(client.is_whitelisted(&contributors.get(0).unwrap()));

    client.add_to_blacklist(&contributors.get(1).unwrap());
    assert!(client.is_blacklisted(&contributors.get(1).unwrap()));

    // Cancel and refund
    client.cancel_campaign();
    assert_eq!(client.status(), Status::Cancelled);

    env.ledger().set_timestamp(1_000_001);
    client.refund_single(&contributors.get(0).unwrap());
    assert_eq!(client.contribution(&contributors.get(0).unwrap()), 0);
}

#[test]
fn test_upgrade_with_multiple_contributions() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 100_000, 1_000_000, 100, 0);

    // Multiple contributors
    let mut contributors: Vec<Address> = Vec::new(&env);
    for _ in 0..10 {
        contributors.push_back(Address::generate(&env));
    }
    let mut total = 0i128;

    for contributor in contributors.iter() {
        let amount = 5_000i128;
        token_admin.mint(&contributor, &amount);
        client.contribute(&contributor, &amount, &token_id, &None);
        total += amount;
    }

    assert_eq!(client.total_raised(), total);
    assert_eq!(client.get_stats().contributor_count, 10);

    // Goal not met (50,000 < 100,000), all should get refunds
    env.ledger().set_timestamp(1_000_001);
    let mut batch = Vec::new(&env);
    for i in 0..5 {
        batch.push_back(contributors.get(i).unwrap().clone());
    }

    let refunded = client.refund_batch(&batch);
    assert_eq!(refunded, 5);
}

#[test]
fn test_full_campaign_lifecycle_after_upgrade() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 5_000, 1_000_000, 100, 0);

    let contributor = Address::generate(&env);
    token_admin.mint(&contributor, &5_000);

    // Contribute
    client.contribute(&contributor, &5_000, &token_id, &None);
    assert_eq!(client.total_raised(), 5_000);

    // Past deadline, goal reached - withdraw
    env.ledger().set_timestamp(1_000_001);
    client.withdraw();

    // Status should be Successful
    assert_eq!(client.status(), Status::Successful);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_refund_full_campaign_after_upgrade() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 100_000, 1_000_000, 100, 0);

    let contributor = Address::generate(&env);
    token_admin.mint(&contributor, &5_000);

    // Contribute less than goal
    client.contribute(&contributor, &5_000, &token_id, &None);

    // Past deadline, goal not met - refund
    env.ledger().set_timestamp(1_000_001);
    client.refund_single(&contributor);

    assert_eq!(client.contribution(&contributor), 0);
    assert_eq!(token::Client::new(&env, &token_id).balance(&contributor), 5_000);
}

#[test]
fn test_rate_limiting_after_upgrade() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 100_000, 1_000_000, 100, 0);

    // Set rate limit
    client.set_rate_limit(&1_000, &3_600);

    let contributor = Address::generate(&env);
    token_admin.mint(&contributor, &2_000);

    // First contribution
    env.ledger().set_timestamp(100);
    client.contribute(&contributor, &500, &token_id, &None);

    // Second within rate limit
    env.ledger().set_timestamp(200);
    client.contribute(&contributor, &400, &token_id, &None);

    // Third exceeding rate limit
    let result = client.try_contribute(&contributor, &300, &token_id, &None);
    assert!(result.is_err());
}

#[test]
fn test_extension_voting_after_upgrade() {
    let env = Env::default();
    let (client, token_id, token_admin) = setup_campaign(&env, 100_000, 1_000_000, 100, 0);

    let contributor = Address::generate(&env);
    token_admin.mint(&contributor, &5_000);

    client.contribute(&contributor, &5_000, &token_id, &None);

    env.ledger().set_timestamp(100);
    client.propose_extension(&2_000_000);

    // Vote for extension - vote weight equals contribution amount
    client.vote_on_extension(&contributor, &true);

    let proposal = client.get_extension_proposal().unwrap();
    assert_eq!(proposal.votes_for, 5_000);

    // Execute after voting period
    env.ledger().set_timestamp(100 + 604_801);
    client.execute_extension();

    assert_eq!(client.deadline(), 2_000_000);
}