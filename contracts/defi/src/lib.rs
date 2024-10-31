#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::{call::{call, Call}, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Defi {
        mapping(address => uint256) balance;
    }
}

sol! {
    error InsufficientBalance(uint balance, uint amount);
    error WithdrawFailed(uint balance, uint withdrawAmount);
}

#[derive(SolidityError)]
pub enum DefiError {
    InsufficientBalance(InsufficientBalance),
    WithdrawFailed(WithdrawFailed)
}

#[public]
impl Defi {
    #[payable]
    pub fn deposit(&mut self) -> U256 {
        self.balance.setter(msg::sender()).set(msg::value());
        return self.balance.get(msg::sender());
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<U256, DefiError> {
        if amount > self.balance.get(msg::sender()) {
            return Err(DefiError::InsufficientBalance(InsufficientBalance {
                balance: self.balance.get(msg::sender()),
                amount: amount
            }));
        }

        // computing the difference
        let mut setter = self.balance.setter(msg::sender());
        match setter.checked_sub(amount) {
            Some(new_balance) => setter.set(new_balance),
            None => panic!("overflow encountered")
        }

        // withdrawing
        match call(Call::new_in(self).value(amount), msg::sender(), &[]) {
            Ok(_) => Ok(self.balance.get(msg::sender())),
            Err(_) => Err(DefiError::WithdrawFailed(WithdrawFailed {
                balance: self.balance.get(msg::sender()),
                withdrawAmount: amount
            }))
        }
    }

    pub fn balance(&self) -> U256 {
        self.balance.get(msg::sender())
    }

    pub fn transfer(&mut self, receiver: Address, amount: U256) -> Result<U256, DefiError> {
        if amount > self.balance.get(msg::sender()) {
            return Err(DefiError::InsufficientBalance(InsufficientBalance {
                balance: self.balance.get(msg::sender()),
                amount: amount
            }));
        }

        // updates
        let mut sender_setter = self.balance.setter(msg::sender());
        match sender_setter.checked_sub(amount) {
            Some(new_balance) => sender_setter.set(new_balance),
            None => panic!("overflow")
        }

        let mut receiver_setter = self.balance.setter(receiver);
        match receiver_setter.checked_add(amount) {
            Some(new_balance) => receiver_setter.set(new_balance),
            None => panic!("overflow")
        }

        Ok(self.balance.get(msg::sender()))
    }
}
