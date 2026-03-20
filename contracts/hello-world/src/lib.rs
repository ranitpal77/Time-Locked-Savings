#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposit {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
}

#[contracttype]
pub enum DataKey {
    UserDeposit(Address),
}

#[contract]
pub struct TimeLockedSavings;

#[contractimpl]
impl TimeLockedSavings {
    /// Deposit funds into the time-locked savings contract.
    /// `unlock_time` is the ledger timestamp (in seconds) after which the funds can be withdrawn.
    pub fn deposit(env: Env, user: Address, token: Address, amount: i128, unlock_time: u64) {
        user.require_auth();

        let key = DataKey::UserDeposit(user.clone());
        if env.storage().persistent().has(&key) {
            panic!("User already has an active deposit. Withdraw it first.");
        }

        if amount <= 0 {
            panic!("Deposit amount must be strictly positive.");
        }

        if unlock_time <= env.ledger().timestamp() {
            panic!("Unlock time must be in the future.");
        }

        // Transfer funds from the user to this contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        // Record the deposit
        let deposit = Deposit {
            token,
            amount,
            unlock_time,
        };
        env.storage().persistent().set(&key, &deposit);
    }

    /// Withdraw funds from the time-locked savings.
    /// Fails if the current ledger timestamp is less than the `unlock_time`.
    pub fn withdraw(env: Env, user: Address) {
        user.require_auth();

        let key = DataKey::UserDeposit(user.clone());
        let deposit: Deposit = env.storage().persistent().get(&key).expect("No deposit found for user.");

        if env.ledger().timestamp() < deposit.unlock_time {
            panic!("Funds are still locked.");
        }

        // Transfer funds back to the user
        let token_client = token::Client::new(&env, &deposit.token);
        token_client.transfer(&env.current_contract_address(), &user, &deposit.amount);

        // Remove the deposit record
        env.storage().persistent().remove(&key);
    }

    /// Read the current deposit for a user, if any.
    pub fn get_deposit(env: Env, user: Address) -> Option<Deposit> {
        let key = DataKey::UserDeposit(user);
        env.storage().persistent().get(&key)
    }
}

// mod test;
