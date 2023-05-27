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
    pub struct Listing {
        vendor: AccountId,
    }

    #[derive(Debug, Default)]
    #[ink::storage_item]
    pub struct Listings {
        values: Mapping<u32, Listing>,
        length: u32,
    }
    impl Listings {
        pub fn index(&self, page: u32, size: u8) -> Vec<Listing> {
            let mut listings: Vec<Listing> = vec![];
            // When there's no listings
            if self.length == 0 {
                return listings;
            }

            let listings_to_skip: Option<u32> = page.checked_mul(size.into());
            let starting_index: u32;
            let ending_index: u32;
            // When the listings to skip is greater than max possible
            if listings_to_skip.is_none() {
                return listings;
            } else {
                let listings_to_skip_unwrapped: u32 = listings_to_skip.unwrap();
                let ending_index_wrapped: Option<u32> =
                    self.length.checked_sub(listings_to_skip_unwrapped);
                // When listings to skip is greater than total number of listings
                if ending_index_wrapped.is_none() {
                    return listings;
                }
                ending_index = ending_index_wrapped.unwrap();
                starting_index = ending_index.checked_sub(size.into()).unwrap_or(0);
            }
            for i in (starting_index..=ending_index).rev() {
                listings.push(self.values.get(i).unwrap())
            }
            listings
        }

        pub fn set(&mut self, value: &Listing) {
            if self.values.insert(self.length, value).is_none() {
                self.length += 1
            }
        }
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug, Clone)]
    pub struct Vendor {}

    // === CONTRACT ===
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Escrow {
        #[storage_field]
        ownable: ownable::Data,
        listings: Listings,
        vendors: Mapping<AccountId, Vendor>,
    }
    impl Escrow {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance._init_with_owner(Self::env().caller());
            instance.listings = Listings {
                values: Mapping::default(),
                length: 0,
            };
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
        pub fn create_listing(&mut self) -> Result<(), EscrowError> {

            // let caller: AccountId = Self::env().caller();
            // if self.vendors.get(&caller).is_some() {
            //     return Err(EscrowError::VendorAlreadyExists);
            // }

            // // Create vendor for caller
            // let vendor: Vendor = Vendor {};
            // self.vendors.insert(caller, &vendor);

            // // Emit event
            // self.env().emit_event(CreateVendor { caller });

            // Ok(())
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

    // === TESTS ===
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
            // * it sets listings
            // assert_eq!(escrow.listings.values, Mapping::default());
            assert_eq!(escrow.listings.length, 0);
            // * it sets vendors
            // assert_eq!(escrow.vendors, Mapping::default());
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
