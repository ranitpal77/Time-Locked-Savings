#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Env, Address, Symbol,
};

use soroban_sdk::token::{self, Client as TokenClient};

#[contracttype]
#[derive(Clone)]
pub struct Deposit {
    pub amount: i128,
    pub unlock_time: u64,
    pub token: Address,
}

#[contract]
pub struct TimeLockedSavings;

#[contractimpl]
impl TimeLockedSavings {

    // 🔒 Deposit tokens with unlock time
    pub fn deposit(
        env: Env,
        user: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
    ) {
        user.require_auth();

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            panic!("Unlock time must be in future");
        }

        let key = (Symbol::short("dep"), user.clone());

        if env.storage().instance().has(&key) {
            panic!("Already deposited");
        }

        let token_client = TokenClient::new(&env, &token);

        // transfer tokens from user → contract
        token_client.transfer(
            &user,
            &env.current_contract_address(),
            &amount,
        );

        let deposit = Deposit {
            amount,
            unlock_time,
            token,
        };

        env.storage().instance().set(&key, &deposit);
    }

    // 💰 Withdraw after unlock
    pub fn withdraw(env: Env, user: Address) {
        user.require_auth();

        let key = (Symbol::short("dep"), user.clone());

        let deposit: Deposit = env
            .storage()
            .instance()
            .get(&key)
            .expect("No deposit");

        let now = env.ledger().timestamp();

        if now < deposit.unlock_time {
            panic!("Still locked");
        }

        let token_client = TokenClient::new(&env, &deposit.token);

        // transfer tokens back to user
        token_client.transfer(
            &env.current_contract_address(),
            &user,
            &deposit.amount,
        );

        env.storage().instance().remove(&key);
    }

    // 🔍 View deposit
    pub fn get_deposit(env: Env, user: Address) -> Option<Deposit> {
        let key = (Symbol::short("dep"), user);
        env.storage().instance().get(&key)
    }
}