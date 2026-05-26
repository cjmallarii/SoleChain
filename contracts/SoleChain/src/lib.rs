#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String, Symbol,
};

#[contracttype]
pub enum DataKey {
    Owner(String),
    History(String),
    Manufacturer,
    Clawback(String),
}

#[contract]
pub struct SoleChainContract;

#[contractimpl]
impl SoleChainContract {

    pub fn initialize(env: Env, manufacturer: Address) {
        if env.storage().instance().has(&DataKey::Manufacturer) {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&DataKey::Manufacturer, &manufacturer);
    }

    pub fn mint(env: Env, shoe_id: String, initial_owner: Address) {
        let manufacturer: Address = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturer)
            .expect("not initialized");

        manufacturer.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::Owner(shoe_id.clone()))
        {
            panic!("shoe already registered");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Owner(shoe_id.clone()), &initial_owner);

        env.storage()
            .persistent()
            .set(&DataKey::History(shoe_id.clone()), &0u32);

        env.events().publish(
            (Symbol::new(&env, "mint"), shoe_id),
            initial_owner,
        );
    }

    pub fn transfer_ownership(env: Env, shoe_id: String, new_owner: Address) {
        let recalled: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Clawback(shoe_id.clone()))
            .unwrap_or(false);

        if recalled {
            panic!("shoe has been recalled");
        }

        let current_owner: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Owner(shoe_id.clone()))
            .expect("shoe not found");

        current_owner.require_auth();

        if current_owner == new_owner {
            panic!("cannot transfer to current owner");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Owner(shoe_id.clone()), &new_owner);

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::History(shoe_id.clone()))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::History(shoe_id.clone()), &(count + 1));

        env.events().publish(
            (Symbol::new(&env, "transfer"), shoe_id),
            (current_owner, new_owner),
        );
    }

    pub fn clawback(env: Env, shoe_id: String) {
        let manufacturer: Address = env
            .storage()
            .instance()
            .get(&DataKey::Manufacturer)
            .expect("not initialized");

        manufacturer.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Clawback(shoe_id.clone()), &true);

        env.events().publish(
            (Symbol::new(&env, "clawback"), shoe_id),
            true,
        );
    }

    pub fn get_owner(env: Env, shoe_id: String) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Owner(shoe_id))
            .expect("shoe not found")
    }

    pub fn get_transfer_count(env: Env, shoe_id: String) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::History(shoe_id))
            .unwrap_or(0)
    }
}