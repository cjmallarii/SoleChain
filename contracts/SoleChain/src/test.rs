#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, SoleChainContractClient<'static>, Address, Address, String) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SoleChainContract);
        let client = SoleChainContractClient::new(&env, &contract_id);
        let manufacturer = Address::generate(&env);
        let buyer = Address::generate(&env);
        let shoe_id = String::from_str(&env, "NK-AJ1-2024-00841");
        client.initialize(&manufacturer);
        (env, client, manufacturer, buyer, shoe_id)
    }

    // Test 1 — Happy path: mint then transfer succeeds end-to-end
    #[test]
    fn test_mint_and_transfer_happy_path() {
        let (env, client, manufacturer, buyer, shoe_id) = setup();
        let reseller = Address::generate(&env);

        client.mint(&shoe_id, &buyer);
        assert_eq!(client.get_owner(&shoe_id), buyer);

        client.transfer_ownership(&shoe_id, &reseller);
        assert_eq!(client.get_owner(&shoe_id), reseller);
        assert_eq!(client.get_transfer_count(&shoe_id), 1);
    }

    // Test 2 — Edge case: non-owner cannot transfer ownership
    #[test]
    #[should_panic(expected = "unauthorized")]
    fn test_non_owner_cannot_transfer() {
        let (env, client, _manufacturer, buyer, shoe_id) = setup();
        let impersonator = Address::generate(&env);

        client.mint(&shoe_id, &buyer);
        // impersonator attempts transfer — should panic since auth mocking
        // will fail for an address that doesn't own the shoe
        env.mock_auths(&[]); // clear auth mocks to force real auth check
        client.transfer_ownership(&shoe_id, &impersonator);
    }

    // Test 3 — State: storage reflects correct owner after transfer
    #[test]
    fn test_storage_state_after_transfer() {
        let (env, client, _manufacturer, buyer, shoe_id) = setup();
        let reseller = Address::generate(&env);

        client.mint(&shoe_id, &buyer);
        client.transfer_ownership(&shoe_id, &reseller);

        // Storage must hold reseller, not buyer
        let stored_owner = client.get_owner(&shoe_id);
        assert_eq!(stored_owner, reseller);
        assert_ne!(stored_owner, buyer);
    }

    // Test 4 — Edge case: recalled shoe cannot be transferred
    #[test]
    #[should_panic(expected = "shoe has been recalled")]
    fn test_recalled_shoe_cannot_transfer() {
        let (_env, client, _manufacturer, buyer, shoe_id) = setup();
        let reseller = Address::generate(&_env);

        client.mint(&shoe_id, &buyer);
        client.clawback(&shoe_id);
        // Should panic — shoe is recalled
        client.transfer_ownership(&shoe_id, &reseller);
    }

    // Test 5 — Edge case: duplicate mint for same shoe ID is rejected
    #[test]
    #[should_panic(expected = "shoe already registered")]
    fn test_duplicate_mint_rejected() {
        let (_env, client, _manufacturer, buyer, shoe_id) = setup();
        let buyer2 = Address::generate(&_env);

        client.mint(&shoe_id, &buyer);
        // Second mint of same shoe_id should panic
        client.mint(&shoe_id, &buyer2);
    }
}