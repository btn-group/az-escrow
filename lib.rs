#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod escrow {
    use openbrush::{contracts::ownable::*, traits::Storage};

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        admin: AccountId,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Escrow {
        #[storage_field]
        ownable: ownable::Data,
    }

    impl Escrow {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance._init_with_owner(Self::env().caller());
            instance
        }

        #[ink(message)]
        pub fn config(&self) -> Config {
            Config {
                admin: self.ownable.owner(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use openbrush::test_utils;

        // === TESTS ===
        #[ink::test]
        fn test_new() {
            let accounts = test_utils::accounts();
            test_utils::change_caller(accounts.bob);
            let escrow = Escrow::new();
            // * it sets owner as caller
            assert_eq!(escrow.ownable.owner(), accounts.bob);
        }

        #[ink::test]
        fn test_config() {
            let accounts = test_utils::accounts();
            test_utils::change_caller(accounts.alice);
            let escrow = Escrow::new();
            let config = escrow.config();
            // * it returns the config
            assert_eq!(config.admin, accounts.alice);
        }
    }
}
