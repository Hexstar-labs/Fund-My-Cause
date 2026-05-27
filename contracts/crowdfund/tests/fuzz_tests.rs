#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};

use crowdfund::{Category, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

struct Campaign<'a> {
    client: CrowdfundContractClient<'a>,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
    token_id: Address,
    contract_id: Address,
    creator: Address,
}

fn setup_campaign(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    max_contribution: i128,
) -> Campaign {
    env.mock_all_auths();

    let creator = Address::generate(env);
    let token_admin_addr = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(token_admin_addr);
    let contract_id = env.register_contract(None, CrowdfundContract);

    let client = CrowdfundContractClient::new(env, &contract_id);
    let token = token::Client::new(env, &token_id);
    let token_admin = token::StellarAssetClient::new(env, &token_id);

    env.ledger().set_timestamp(100);

    client.initialize(
        &creator,
        &token_id,
        &goal,
        &deadline,
        &min_contribution,
        &max_contribution,
        &String::from_str(env, "Fuzz Test Campaign"),
        &String::from_str(env, "Fuzz testing"),
        &None,
        &None,
        &None,
        &Category::Other,
        &None,
        &None,
    );

    Campaign {
        client,
        token,
        token_admin,
        token_id,
        contract_id,
        creator,
    }
}

prop_compose! {
    fn valid_amount()(amount in 1i128..1_000_000i128) -> i128 { amount }
}

prop_compose! {
    fn valid_timestamp()(ts in 100_000u64..1_000_000u64) -> u64 { ts }
}

prop_compose! {
    fn valid_deadline()(deadline in 100_000u64..1_000_000u64) -> u64 { deadline }
}

// ── Fuzz tests for contribute ────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn fuzz_contribute_valid_amounts_succeeds(
        amount in valid_amount(),
        _deadline_offset in 10_000u64..100_000u64,
    ) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &amount);

        c.client.contribute(&contributor, &amount, &c.token_id, &None);

        assert_eq!(c.client.contribution(&contributor), amount);
        assert_eq!(c.client.total_raised(), amount);
    }

    #[test]
    fn fuzz_contribute_multiple_contributors_succeeds(
        (amount1, amount2) in (valid_amount(), valid_amount()),
    ) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let c1 = Address::generate(&env);
        let c2 = Address::generate(&env);

        c.token_admin.mint(&c1, &amount1);
        c.token_admin.mint(&c2, &amount2);

        c.client.contribute(&c1, &amount1, &c.token_id, &None);
        c.client.contribute(&c2, &amount2, &c.token_id, &None);

        assert_eq!(c.client.total_raised(), amount1 + amount2);
    }

    #[test]
    fn fuzz_contribute_below_min_is_rejected(amount in 0i128..100i128) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 100, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &1_000);

        let result = c.client.try_contribute(&contributor, &amount, &c.token_id, &None);
        assert!(result.is_err());
    }

    #[test]
    fn fuzz_contribute_capped_at_max_succeeds(
        total_contribution in 1i128..400i128,
        additional_amount in 1i128..200i128,
    ) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 100, 500);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &(total_contribution + additional_amount));

        // First contribution
        c.client.contribute(&contributor, &total_contribution, &c.token_id, &None);

        // Second contribution - should succeed if total <= 500, fail otherwise
        let result = c.client.try_contribute(&contributor, &additional_amount, &c.token_id, &None);

        // Only succeeds if total contribution would be <= 500
        if total_contribution + additional_amount <= 500 {
            assert!(result.is_ok());
        } else {
            // Should fail with ExceedsMaximum (error #14) if over limit
            assert!(result.is_err());
        }
    }
}

// ── Fuzz tests for refund_single ─────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn fuzz_refund_single_after_cancel_succeeds(amount in valid_amount()) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &amount);

        c.client.contribute(&contributor, &amount, &c.token_id, &None);
        c.client.cancel_campaign();

        env.ledger().set_timestamp(1_000_001);
        c.client.refund_single(&contributor);

        assert_eq!(c.client.contribution(&contributor), 0);
        assert_eq!(c.token.balance(&contributor), amount);
    }

    #[test]
    fn fuzz_refund_single_after_failed_campaign_succeeds(
        amount in valid_amount(),
        goal in 1i128..100_000i128,
    ) {
        let env = Env::default();
        if goal <= amount {
            return Ok(());
        }
        let c = setup_campaign(&env, goal, 1_000, 1, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &amount);

        c.client.contribute(&contributor, &amount, &c.token_id, &None);

        env.ledger().set_timestamp(2_000);
        let result = c.client.try_refund_single(&contributor);

        if amount >= goal {
            assert!(result.is_err());
        }
    }

    #[test]
    fn fuzz_refund_single_double_claim_succeeds(amount in valid_amount()) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &amount);

        c.client.contribute(&contributor, &amount, &c.token_id, &None);
        c.client.cancel_campaign();

        env.ledger().set_timestamp(1_000_001);
        c.client.refund_single(&contributor);

        let balance_after_first = c.token.balance(&contributor);
        c.client.refund_single(&contributor);
        assert_eq!(c.token.balance(&contributor), balance_after_first);
    }
}

// ── Fuzz tests for withdraw ──────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn fuzz_withdraw_successful_campaign_succeeds(
        goal in 10_000i128..100_000i128,
        total in 10_000i128..100_000i128,
    ) {
        let env = Env::default();
        if total < goal {
            return Ok(());
        }
        let c = setup_campaign(&env, goal, 1_000_000, 1, 0);

        let contributor = Address::generate(&env);
        c.token_admin.mint(&contributor, &total);

        c.client.contribute(&contributor, &total, &c.token_id, &None);

        env.ledger().set_timestamp(1_000_001);
        c.client.withdraw();

        assert_eq!(c.client.total_raised(), 0);
    }

    #[test]
    fn fuzz_withdraw_with_platform_fee_succeeds(
        goal in 10_000i128..100_000i128,
        fee_bps in 0u32..10_000u32,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let platform = Address::generate(&env);
        let token_admin_addr = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin_addr);
        let contract_id = env.register_contract(None, CrowdfundContract);

        let client = CrowdfundContractClient::new(&env, &contract_id);
        let token_admin = token::StellarAssetClient::new(&env, &token_id);

        env.ledger().set_timestamp(100);

        client.initialize(
            &creator,
            &token_id,
            &goal,
            &1_000_000,
            &1,
            &0i128,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
            &None,
            &Some(PlatformConfig { address: platform, fee_bps }),
            &None,
            &Category::Other,
            &None,
            &None,
        );

        let contributor = Address::generate(&env);
        token_admin.mint(&contributor, &goal);
        client.contribute(&contributor, &goal, &token_id, &None);

        env.ledger().set_timestamp(1_000_001);
        client.withdraw();

        assert_eq!(client.total_raised(), 0);
    }
}

// ── Fuzz tests for edge cases ──────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn fuzz_large_number_of_contributors_succeeds(count in 1usize..50usize) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let mut total = 0i128;
        for _ in 0..count {
            let contributor = Address::generate(&env);
            c.token_admin.mint(&contributor, &1_000);
            c.client.contribute(&contributor, &1_000, &c.token_id, &None);
            total += 1_000;
        }

        assert_eq!(c.client.total_raised(), total);
    }

    #[test]
    fn fuzz_concurrent_contributions_succeed(
        (a1, a2, a3) in (valid_amount(), valid_amount(), valid_amount()),
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let mut contributors = Vec::new(&env);
        contributors.push_back(Address::generate(&env));
        contributors.push_back(Address::generate(&env));
        contributors.push_back(Address::generate(&env));

        c.token_admin.mint(&contributors.get(0).unwrap(), &a1);
        c.token_admin.mint(&contributors.get(1).unwrap(), &a2);
        c.token_admin.mint(&contributors.get(2).unwrap(), &a3);

        c.client.contribute(&contributors.get(0).unwrap(), &a1, &c.token_id, &None);
        c.client.contribute(&contributors.get(1).unwrap(), &a2, &c.token_id, &None);
        c.client.contribute(&contributors.get(2).unwrap(), &a3, &c.token_id, &None);

        assert_eq!(c.client.total_raised(), a1.saturating_add(a2).saturating_add(a3));
    }
}

// ── Fuzz tests for overflow protection ──────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn fuzz_negative_amounts_rejected(amount in i128::MIN..0i128) {
        let env = Env::default();
        let c = setup_campaign(&env, 1_000_000, 1_000_000, 1, 0);

        let contributor = Address::generate(&env);

        let result = c.client.try_contribute(&contributor, &amount, &c.token_id, &None);
        assert!(result.is_err());
    }
}