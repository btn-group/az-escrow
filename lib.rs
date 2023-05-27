#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod escrow {
    use ink::storage::Mapping;
    use openbrush::{contracts::ownable::*, traits::Storage};

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EscrowError {
        ListingCanOnlyBeCreatedByAVendor,
        ListingLimitReached,
        ListingNotFound,
        VendorAlreadyExists,
        Unauthorized,
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct CreateListing {
        id: u32,
        #[ink(topic)]
        vendor: AccountId,
    }

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
        id: u32,
        vendor: AccountId,
        available_amount: Balance,
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
            if let Some(listings_to_skip_unwrapped) = listings_to_skip {
                let ending_index_wrapped: Option<u32> =
                    self.length.checked_sub(listings_to_skip_unwrapped);
                // When listings to skip is greater than total number of listings
                if ending_index_wrapped.is_none() {
                    return listings;
                }
                ending_index = ending_index_wrapped.unwrap();
                starting_index = ending_index.saturating_sub(size.into());
            } else {
                return listings;
            }
            for i in (starting_index..=ending_index).rev() {
                listings.push(self.values.get(i).unwrap())
            }
            listings
        }

        pub fn create(&mut self, value: &Listing) {
            if self.values.insert(self.length, value).is_none() {
                self.length += 1
            }
        }

        pub fn update(&mut self, value: &Listing) {
            self.values.insert(value.id, value);
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
            if self.listings.length == u32::MAX {
                return Err(EscrowError::ListingLimitReached);
            }
            let caller: AccountId = Self::env().caller();
            if self.vendors.get(caller).is_none() {
                return Err(EscrowError::ListingCanOnlyBeCreatedByAVendor);
            }

            let listing: Listing = Listing {
                id: self.listings.length,
                vendor: caller,
                available_amount: 0,
            };
            self.listings.create(&listing);

            // Emit event
            self.env().emit_event(CreateListing {
                id: listing.id,
                vendor: listing.vendor,
            });

            Ok(())
        }

        #[ink(message, payable)]
        pub fn deposit_into_listing(&mut self, id: u32) -> Result<(), EscrowError> {
            let listing_wrapped: Option<Listing> = self.listings.values.get(id);
            if let Some(mut listing) = listing_wrapped {
                if listing.vendor != Self::env().caller() {
                    return Err(EscrowError::Unauthorized);
                }

                listing.available_amount += self.env().transferred_value();
                self.listings.update(&listing);
            } else {
                return Err(EscrowError::ListingNotFound);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn create_vendor(&mut self) -> Result<(), EscrowError> {
            let caller: AccountId = Self::env().caller();
            if self.vendors.get(caller).is_some() {
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
        use ink::env::{test::DefaultAccounts, DefaultEnvironment};
        use openbrush::test_utils;

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, Escrow) {
            let accounts = test_utils::accounts();
            test_utils::change_caller(accounts.bob);
            let escrow = Escrow::new();
            (accounts, escrow)
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        // === TESTS ===
        #[ink::test]
        fn test_new() {
            let (accounts, escrow) = init();
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
            let (accounts, escrow) = init();
            let config = escrow.config();
            // * it returns the config
            assert_eq!(config.admin, accounts.bob);
        }

        #[ink::test]
        fn test_create_listing() {
            let (accounts, mut escrow) = init();
            // when the maximum number of listings has been reached
            escrow.listings.length = u32::MAX;
            // * it raises an error
            let mut result = escrow.create_listing();
            assert_eq!(result, Err(EscrowError::ListingLimitReached));
            // when the maximum number of listings hasn't been reached
            escrow.listings.length = u32::MAX - 1;
            // = when caller isn't a vendor
            // = * it raises an error
            result = escrow.create_listing();
            assert_eq!(result, Err(EscrowError::ListingCanOnlyBeCreatedByAVendor));
            // = when caller is a vendor
            escrow.vendors.insert(accounts.bob, &Vendor {});
            // = * it creates a listing at the listings length index
            result = escrow.create_listing();
            assert!(result.is_ok());
            assert_eq!(
                escrow.listings.values.get(u32::MAX - 1).unwrap().vendor,
                accounts.bob
            );
            // = * it increases the listings length by one
            assert_eq!(escrow.listings.length, u32::MAX);
        }

        #[ink::test]
        fn test_create_vendor() {
            let (accounts, mut escrow) = init();
            // when account is not a vendor
            // * it creates a vendor profile for account
            // * it emits a CreateVendor event (TO DO AFTER HACKATHON)
            let mut result = escrow.create_vendor();
            assert!(result.is_ok());
            assert!(escrow.vendors.get(&accounts.bob).is_some());

            // when account is already a vendor
            // * it raises an error
            result = escrow.create_vendor();
            assert_eq!(result, Err(EscrowError::VendorAlreadyExists));
        }

        #[ink::test]
        fn test_deposit_into_listing() {
            let (accounts, mut escrow) = init();

            // when listing does not exist
            // * it raises an error
            let mut result = escrow.deposit_into_listing(0);
            assert_eq!(result, Err(EscrowError::ListingNotFound));

            // when listing exists
            let _ = escrow.create_vendor();
            let _ = escrow.create_listing();
            // = when listing does not belong to caller
            test_utils::change_caller(accounts.alice);
            // = * it raises an error
            result = escrow.deposit_into_listing(0);
            assert_eq!(result, Err(EscrowError::Unauthorized));
            // = when listing belongs to caller
            test_utils::change_caller(accounts.bob);
            set_balance(accounts.bob, 10);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1);
            // = * it increases the listing available_amount
            result = escrow.deposit_into_listing(0);
            assert!(result.is_ok());
            assert_eq!(escrow.listings.values.get(0).unwrap().available_amount, 1);
        }
    }
}
