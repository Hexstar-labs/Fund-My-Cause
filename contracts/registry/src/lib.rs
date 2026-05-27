//! # Fund-My-Cause Registry Contract
//!
//! A lightweight Soroban contract that maintains a deduplicated, paginated list
//! of all deployed [`CrowdfundContract`] campaign addresses on the Stellar network.
//!
//! ## Overview
//!
//! The registry acts as an on-chain directory. When a new campaign contract is
//! deployed, its address is registered here so that frontends and indexers can
//! discover all campaigns without relying on off-chain databases.
//!
//! ## Usage
//!
//! ```ignore
//! // Register a newly deployed campaign
//! registry_client.register(&campaign_contract_address);
//!
//! // List the first 20 campaigns
//! let page = registry_client.list(&0, &20);
//!
//! // List the next 20
//! let page2 = registry_client.list(&20, &20);
//! ```
//!
//! ## Storage
//!
//! All campaign addresses are stored in a single instance-storage entry under
//! the `CMPLIST` key as a `Vec<Address>`. Deduplication is enforced on write.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Instance storage key for the list of registered campaign contract addresses.
const KEY_CAMPAIGNS: Symbol = symbol_short!("CMPLIST");

/// Storage key variants for category-indexed campaign lists.
///
/// Each `CategoryList(id)` entry stores a `Vec<Address>` of all campaigns
/// registered under that category identifier.  The `id` follows the same
/// numeric discriminant as the crowdfund contract's `Category` enum:
///
/// | id | Category   |
/// |----|------------|
/// |  0 | Charity    |
/// |  1 | Technology |
/// |  2 | Creative   |
/// |  3 | Event      |
/// |  4 | Personal   |
/// |  5 | Other      |
#[contracttype]
enum RegDataKey {
    /// Paginated list of campaign addresses for a given numeric category id.
    CategoryList(u32),
}

/// The Fund-My-Cause registry contract.
///
/// Maintains a deduplicated, append-only list of all deployed campaign contract
/// addresses. Provides paginated read access for frontends and indexers.
#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Registers a campaign contract address in the registry.
    ///
    /// If the address is already registered, this is a no-op — no duplicate
    /// entries are created and no event is emitted.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `campaign_id` - The contract address of the deployed campaign to register.
    ///
    /// # Side Effects
    ///
    /// - Appends `campaign_id` to the stored campaign list (if not already present).
    /// - Publishes a `("registry", "registered")` event with `campaign_id` as the
    ///   data payload.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Called immediately after deploying a new CrowdfundContract
    /// registry_client.register(&new_campaign_address);
    /// ```
    pub fn register(env: Env, campaign_id: Address) {
        let mut campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&KEY_CAMPAIGNS)
            .unwrap_or_else(|| Vec::new(&env));

        if !campaigns.contains(&campaign_id) {
            campaigns.push_back(campaign_id.clone());
            env.storage().instance().set(&KEY_CAMPAIGNS, &campaigns);
            env.events()
                .publish(("registry", "registered"), campaign_id);
        }
    }

    /// Registers a campaign contract address together with its numeric category id.
    ///
    /// Performs all the same deduplication and bookkeeping as [`register`], and
    /// additionally maintains a per-category index so that callers can later
    /// retrieve campaigns filtered by category via [`get_campaigns_by_category`].
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `campaign_id` - The contract address of the deployed campaign.
    /// * `category_id` - Numeric category identifier (see [`RegDataKey`]).
    ///
    /// # Side Effects
    ///
    /// - Appends `campaign_id` to the global campaign list (if not already present).
    /// - Appends `campaign_id` to the category-specific list (if not already present).
    /// - Publishes `("registry", "registered")` when a globally new campaign is added.
    pub fn register_with_category(env: Env, campaign_id: Address, category_id: u32) {
        // ── Global list (same logic as register()) ────────────────────────────
        let mut campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&KEY_CAMPAIGNS)
            .unwrap_or_else(|| Vec::new(&env));

        if !campaigns.contains(&campaign_id) {
            campaigns.push_back(campaign_id.clone());
            env.storage().instance().set(&KEY_CAMPAIGNS, &campaigns);
            env.events()
                .publish(("registry", "registered"), campaign_id.clone());
        }

        // ── Category-specific list ────────────────────────────────────────────
        let cat_key = RegDataKey::CategoryList(category_id);
        let mut cat_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&cat_key)
            .unwrap_or_else(|| Vec::new(&env));

        if !cat_list.contains(&campaign_id) {
            cat_list.push_back(campaign_id);
            env.storage().instance().set(&cat_key, &cat_list);
        }
    }

    /// Returns a paginated slice of campaign addresses filtered by category.
    ///
    /// Only campaigns registered via [`register_with_category`] appear in category
    /// results.  Pagination is zero-indexed, identical to [`list`].
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `category_id` - Numeric category to filter by (see [`RegDataKey`]).
    /// * `offset` - Zero-based index of the first result to return.
    /// * `limit` - Maximum number of results to return.  Passing `0` returns empty.
    ///
    /// # Returns
    ///
    /// A `Vec<Address>` of up to `limit` campaign addresses in the given category,
    /// or an empty `Vec` if no campaigns match.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Fetch first 10 Technology campaigns (category_id = 1)
    /// let tech = registry_client.get_campaigns_by_category(&1, &0, &10);
    /// ```
    pub fn get_campaigns_by_category(
        env: Env,
        category_id: u32,
        offset: u32,
        limit: u32,
    ) -> Vec<Address> {
        if limit == 0 {
            return Vec::new(&env);
        }

        let campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&RegDataKey::CategoryList(category_id))
            .unwrap_or_else(|| Vec::new(&env));

        let total = campaigns.len();
        if offset >= total {
            return Vec::new(&env);
        }

        let end = offset.saturating_add(limit).min(total);
        let mut out = Vec::new(&env);

        let mut i = offset;
        while i < end {
            if let Some(addr) = campaigns.get(i) {
                out.push_back(addr);
            }
            i += 1;
        }

        out
    }

    /// Returns a paginated slice of registered campaign contract addresses.
    ///
    /// Pagination is zero-indexed: pass `offset = 0, limit = 20` for the first
    /// page, `offset = 20, limit = 20` for the second, and so on.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment.
    /// * `offset` - Zero-based index of the first item to return.
    /// * `limit` - Maximum number of items to return. Passing `0` returns an
    ///   empty list immediately.
    ///
    /// # Returns
    ///
    /// A `Vec<Address>` containing up to `limit` campaign addresses starting at
    /// `offset`. Returns an empty `Vec` if `limit` is `0`, `offset` is beyond
    /// the end of the list, or no campaigns have been registered yet.
    pub fn list(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        if limit == 0 {
            return Vec::new(&env);
        }

        let campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&KEY_CAMPAIGNS)
            .unwrap_or_else(|| Vec::new(&env));

        let total = campaigns.len();
        if offset >= total {
            return Vec::new(&env);
        }

        let end = offset.saturating_add(limit).min(total);
        let mut out = Vec::new(&env);

        let mut i = offset;
        while i < end {
            if let Some(addr) = campaigns.get(i) {
                out.push_back(addr);
            }
            i += 1;
        }

        out
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address};

    #[test]
    fn test_register_with_category_and_filter() {
        let env = Env::default();

        let registry_id = env.register_contract(None, RegistryContract);
        let client = RegistryContractClient::new(&env, &registry_id);

        let charity1 = Address::generate(&env);
        let charity2 = Address::generate(&env);
        let tech1 = Address::generate(&env);

        // category_id 0 = Charity, 1 = Technology
        client.register_with_category(&charity1, &0);
        client.register_with_category(&charity2, &0);
        client.register_with_category(&tech1, &1);

        // All campaigns appear in global list
        let all = client.list(&0, &10);
        assert_eq!(all.len(), 3);

        // Charity filter returns only charity campaigns
        let charities = client.get_campaigns_by_category(&0, &0, &10);
        assert_eq!(charities.len(), 2);

        // Tech filter returns only tech campaign
        let techs = client.get_campaigns_by_category(&1, &0, &10);
        assert_eq!(techs.len(), 1);
        assert_eq!(techs.get(0).unwrap(), tech1);

        // Unknown category returns empty
        let empty = client.get_campaigns_by_category(&99, &0, &10);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_register_with_category_deduplicates() {
        let env = Env::default();

        let registry_id = env.register_contract(None, RegistryContract);
        let client = RegistryContractClient::new(&env, &registry_id);

        let campaign = Address::generate(&env);

        client.register_with_category(&campaign, &0);
        client.register_with_category(&campaign, &0); // duplicate — ignored

        let charities = client.get_campaigns_by_category(&0, &0, &10);
        assert_eq!(charities.len(), 1);

        let all = client.list(&0, &10);
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_get_campaigns_by_category_pagination() {
        let env = Env::default();

        let registry_id = env.register_contract(None, RegistryContract);
        let client = RegistryContractClient::new(&env, &registry_id);

        let a = Address::generate(&env);
        let b = Address::generate(&env);
        let c = Address::generate(&env);

        // Register 3 creative campaigns (category_id = 2)
        client.register_with_category(&a, &2);
        client.register_with_category(&b, &2);
        client.register_with_category(&c, &2);

        let page1 = client.get_campaigns_by_category(&2, &0, &2);
        assert_eq!(page1.len(), 2);

        let page2 = client.get_campaigns_by_category(&2, &2, &2);
        assert_eq!(page2.len(), 1);

        // limit = 0 returns empty
        let empty = client.get_campaigns_by_category(&2, &0, &0);
        assert_eq!(empty.len(), 0);

        // offset beyond end returns empty
        let beyond = client.get_campaigns_by_category(&2, &10, &5);
        assert_eq!(beyond.len(), 0);
    }

    #[test]
    fn register_deduplicates_and_lists_with_pagination() {
        let env = Env::default();

        let registry_id = env.register_contract(None, RegistryContract);
        let client = RegistryContractClient::new(&env, &registry_id);

        let a = Address::generate(&env);
        let b = Address::generate(&env);
        let c = Address::generate(&env);

        client.register(&a);
        client.register(&b);
        client.register(&a); // duplicate — should be ignored
        client.register(&c);

        // Total unique registrations: 3
        let page1 = client.list(&0, &2);
        assert_eq!(page1.len(), 2);

        let page2 = client.list(&2, &2);
        assert_eq!(page2.len(), 1);

        // Empty page beyond end
        let page3 = client.list(&10, &5);
        assert_eq!(page3.len(), 0);

        // limit = 0 returns empty
        let empty = client.list(&0, &0);
        assert_eq!(empty.len(), 0);
    }
}
