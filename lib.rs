#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod escrow {
    use ink::storage::Mapping;
    use openbrush::{contracts::ownable::*, traits::Storage};

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EscrowError {
        VendorAlreadyExists,
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct CreateVendor {
        #[ink(topic)]
        caller: AccountId,
    }

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        admin: AccountId,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug, Clone)]
    pub struct Vendor {}

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Escrow {
        #[storage_field]
        ownable: ownable::Data,
        vendors: Mapping<AccountId, Vendor>,
    }
    impl Escrow {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance._init_with_owner(Self::env().caller());
            instance.vendors = Mapping::default();
            instance
        }

        #[ink(message)]
        pub fn config(&self) -> Config {
            Config {
                admin: self.ownable.owner(),
            }
        }

        #[ink(message)]
        pub fn create_vendor(&mut self) -> Result<(), EscrowError> {
            let caller: AccountId = Self::env().caller();
            if self.vendors.get(&caller).is_some() {
                return Err(EscrowError::VendorAlreadyExists);
            }

            // Create vendor for caller
            let vendor: Vendor = Vendor {};
            self.vendors.insert(caller, &vendor);

            // Emit event
            self.env().emit_event(CreateVendor { caller });

            Ok(())
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

        #[ink::test]
        fn test_create_vendor() {
            let accounts = test_utils::accounts();
            test_utils::change_caller(accounts.alice);
            let mut escrow = Escrow::new();
            // when account is not a vendor
            // * it creates a vendor profile for account
            // * it emits a CreateVendor event (TO DO AFTER HACKATHON)
            let mut result = escrow.create_vendor();
            assert!(result.is_ok());
            assert!(escrow.vendors.get(&accounts.alice).is_some());

            // when account is already a vendor
            // * it raises an error
            result = escrow.create_vendor();
            assert_eq!(result, Err(EscrowError::VendorAlreadyExists));
        }
    }
}
