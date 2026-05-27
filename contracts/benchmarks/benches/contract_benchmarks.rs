use criterion::{black_box, criterion_group, criterion_main, Criterion};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};
use crowdfund::{Category, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

fn create_test_campaign(env: &Env, goal: i128, deadline: u64, min: i128, max: i128) -> (CrowdfundContractClient<'_>, Address, Address) {
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
        &min,
        &max,
        &String::from_str(env, "Benchmark Campaign"),
        &String::from_str(env, "Performance testing"),
        &None::<soroban_sdk::Vec<String>>,
        &None::<PlatformConfig>,
        &None,
        &Category::Other,
        &None,
        &None,
    );
    (client, token_id, token_admin_client)
}

fn benchmark_contribute(c: &mut Criterion) {
    c.bench_function("contribute_single", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, token_id, token_admin_client) = create_test_campaign(&env, 100_000, 10_000, 100, 0);

            let contributor = Address::generate(&env);
            token_admin_client.mint(&contributor, &1_000);

            black_box(client.contribute(&contributor, &1_000, &token_id, &None));
        })
    });

    c.bench_function("contribute_multiple_10", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, token_id, token_admin_client) = create_test_campaign(&env, 100_000, 10_000, 100, 0);

            for i in 0..10 {
                let contributor = Address::generate(&env);
                token_admin_client.mint(&contributor, &1_000);
                client.contribute(&contributor, &1_000, &token_id, &None);
                black_box(i);
            }
        })
    });

    c.bench_function("contribute_with_message", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, token_id, token_admin_client) = create_test_campaign(&env, 100_000, 10_000, 0, 0);

            let contributor = Address::generate(&env);
            token_admin_client.mint(&contributor, &1_000);

            client.contribute(&contributor, &1_000, &token_id, &Some(String::from_str(&env, "Test message")));
        })
    });
}

fn benchmark_refund(c: &mut Criterion) {
    c.bench_function("refund_single", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, token_id, token_admin_client) = create_test_campaign(&env, 100_000, 10_000, 100, 0);

            let contributor = Address::generate(&env);
            token_admin_client.mint(&contributor, &1_000);
            client.contribute(&contributor, &1_000, &token_id, &None);

            env.ledger().set_timestamp(11_000);
            client.cancel_campaign();

            black_box(client.refund_single(&contributor));
        })
    });

    c.bench_function("refund_batch_25", |b| {
        b.iter(|| {
            let env = Env::default();
            let (client, token_id, token_admin_client) = create_test_campaign(&env, 100_000, 10_000, 100, 0);

            let mut contributors: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
            for _ in 0..25 {
                let contributor = Address::generate(&env);
                token_admin_client.mint(&contributor, &1_000);
                client.contribute(&contributor, &1_000, &token_id, &None);
                contributors.push_back(contributor);
            }

            env.ledger().set_timestamp(11_000);
            client.cancel_campaign();

            black_box(client.refund_batch(&contributors));
        })
    });
}

fn benchmark_withdraw(c: &mut Criterion) {
    c.bench_function("withdraw_successful", |b| {
        b.iter(|| {
            let env = Env::default();
            env.mock_all_auths();
            let creator = Address::generate(&env);
            let token_admin = Address::generate(&env);
            let token_id = env.register_stellar_asset_contract(token_admin);
            let contract_id = env.register_contract(None, CrowdfundContract);
            let client = CrowdfundContractClient::new(&env, &contract_id);
            let token_admin_client = token::StellarAssetClient::new(&env, &token_id);

            env.ledger().set_timestamp(100);
            client.initialize(
                &creator,
                &token_id,
                &1_000,
                &10_000,
                &100,
                &0i128,
                &String::from_str(&env, "Test"),
                &String::from_str(&env, "Test"),
                &None,
                &Some(PlatformConfig {
                    address: Address::generate(&env),
                    fee_bps: 250,
                }),
                &None,
                &Category::Other,
                &None,
                &None,
            );

            let contributor = Address::generate(&env);
            token_admin_client.mint(&contributor, &1_000);
            client.contribute(&contributor, &1_000, &token_id, &None);

            env.ledger().set_timestamp(11_000);
            black_box(client.withdraw());
        })
    });

    c.bench_function("withdraw_with_vesting", |b| {
        b.iter(|| {
            let env = Env::default();
            env.mock_all_auths();
            let creator = Address::generate(&env);
            let token_admin = Address::generate(&env);
            let token_id = env.register_stellar_asset_contract(token_admin);
            let contract_id = env.register_contract(None, CrowdfundContract);
            let client = CrowdfundContractClient::new(&env, &contract_id);
            let token_admin_client = token::StellarAssetClient::new(&env, &token_id);

            env.ledger().set_timestamp(100);
            client.initialize(
                &creator,
                &token_id,
                &1_000,
                &10_000,
                &100,
                &0i128,
                &String::from_str(&env, "Test"),
                &String::from_str(&env, "Test"),
                &None,
                &None,
                &None,
                &Category::Other,
                &Some(crowdfund::VestingSchedule { cliff: 2_000, duration: 5_000 }),
                &None,
            );

            let contributor = Address::generate(&env);
            token_admin_client.mint(&contributor, &1_000);
            client.contribute(&contributor, &1_000, &token_id, &None);

            env.ledger().set_timestamp(3_000);
            black_box(client.withdraw());
        })
    });
}

criterion_group!(benches, benchmark_contribute, benchmark_refund, benchmark_withdraw);
criterion_main!(benches);
