#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod escrow {
    use ink::storage::Mapping;
    use openbrush::{contracts::ownable::*, traits::Storage};

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EscrowError {
        AmountUnavailable,
        InsufficientFunds,
        ListingCanOnlyBeCreatedByAVendor,
        ListingLimitReached,
        ListingNotFound,
        OrderCancelled,
        OrderFinalised,
        OrderNotFound,
        VendorAlreadyExists,
        Unauthorised,
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct CreateListing {
        #[ink(topic)]
        id: u32,
        vendor: AccountId,
    }

    #[ink(event)]
    pub struct CreateOrder {
        #[ink(topic)]
        id: u64,
        buyer: AccountId,
        vendor: AccountId,
    }

    #[ink(event)]
    pub struct CreateVendor {
        #[ink(topic)]
        caller: AccountId,
    }

    #[ink(event)]
    pub struct UpdateOrder {
        #[ink(topic)]
        id: u64,
        status: u8,
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

    // Order statuses
    // 0 => Open
    // 1 => PendingVerification
    // 2 => Finalised
    // 3 => Cancelled
    // 4 => Disputed
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug, Clone)]
    pub struct Order {
        id: u64,
        buyer: AccountId,
        vendor: AccountId,
        amount: Balance,
        payment_verification: Option<String>,
        status: u8,
    }

    #[derive(Debug, Default)]
    #[ink::storage_item]
    pub struct Orders {
        values: Mapping<u64, Order>,
        length: u64,
    }
    impl Orders {
        pub fn index(&self, page: u64, size: u8) -> Vec<Order> {
            let mut orders: Vec<Order> = vec![];
            // When there's no orders
            if self.length == 0 {
                return orders;
            }

            let orders_to_skip: Option<u64> = page.checked_mul(size.into());
            let starting_index: u64;
            let ending_index: u64;
            // When the orders to skip is greater than max possible
            if let Some(orders_to_skip_unwrapped) = orders_to_skip {
                let ending_index_wrapped: Option<u64> =
                    self.length.checked_sub(orders_to_skip_unwrapped);
                // When orders to skip is greater than total number of orders
                if ending_index_wrapped.is_none() {
                    return orders;
                }
                ending_index = ending_index_wrapped.unwrap();
                starting_index = ending_index.saturating_sub(size.into());
            } else {
                return orders;
            }
            for i in (starting_index..=ending_index).rev() {
                orders.push(self.values.get(i).unwrap())
            }
            orders
        }

        pub fn create(&mut self, value: &Order) {
            if self.values.insert(self.length, value).is_none() {
                self.length += 1
            }
        }

        pub fn update(&mut self, value: &Order) {
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
        orders: Orders,
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
            instance.orders = Orders {
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

        #[ink(message)]
        pub fn create_order(
            &mut self,
            listing_id: u32,
            amount: Balance,
        ) -> Result<(), EscrowError> {
            let listing_wrapped: Option<Listing> = self.listings.values.get(listing_id);
            if let Some(mut listing) = listing_wrapped {
                let caller: AccountId = Self::env().caller();
                if listing.vendor == caller {
                    return Err(EscrowError::Unauthorised);
                }
                if amount > listing.available_amount {
                    return Err(EscrowError::AmountUnavailable);
                }

                listing.available_amount -= amount;
                self.listings.update(&listing);

                let order: Order = Order {
                    id: self.orders.length,
                    buyer: caller,
                    vendor: listing.vendor,
                    amount,
                    payment_verification: None,
                    status: 0,
                };
                self.orders.create(&order);

                // Emit event
                self.env().emit_event(CreateOrder {
                    id: order.id,
                    buyer: order.buyer,
                    vendor: order.vendor,
                });
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

        #[ink(message, payable)]
        pub fn deposit_into_listing(&mut self, id: u32) -> Result<(), EscrowError> {
            let listing_wrapped: Option<Listing> = self.listings.values.get(id);
            if let Some(mut listing) = listing_wrapped {
                if listing.vendor != Self::env().caller() {
                    return Err(EscrowError::Unauthorised);
                }

                listing.available_amount += self.env().transferred_value();
                self.listings.update(&listing);
            } else {
                return Err(EscrowError::ListingNotFound);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn update_order_payment_verification(
            &mut self,
            order_id: u64,
            payment_verification: String,
        ) -> Result<(), EscrowError> {
            let order_wrapped: Option<Order> = self.orders.values.get(order_id);
            if let Some(mut order) = order_wrapped {
                let caller: AccountId = Self::env().caller();
                if order.buyer != caller {
                    return Err(EscrowError::Unauthorised);
                } else if order.status == 2 {
                    return Err(EscrowError::OrderFinalised);
                } else if order.status == 3 {
                    return Err(EscrowError::OrderCancelled);
                }
                order.payment_verification = Some(payment_verification);
                order.status = 1;
                self.orders.update(&order);

                // Emit event
                self.env().emit_event(UpdateOrder {
                    id: order.id,
                    status: order.status,
                });
            } else {
                return Err(EscrowError::OrderNotFound);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn withdraw_from_listing(
            &mut self,
            id: u32,
            amount: Balance,
        ) -> Result<(), EscrowError> {
            let listing_wrapped: Option<Listing> = self.listings.values.get(id);
            if let Some(mut listing) = listing_wrapped {
                if listing.vendor != Self::env().caller() {
                    return Err(EscrowError::Unauthorised);
                }
                if amount > listing.available_amount {
                    return Err(EscrowError::InsufficientFunds);
                };

                listing.available_amount -= amount;
                self.listings.update(&listing);
                if self.env().transfer(listing.vendor, amount).is_err() {
                    panic!(
                        "requested transfer failed. this can be the case if the contract does not\
                         have sufficient free funds or if the transfer would have brought the\
                         contract's balance below minimum balance."
                    )
                }
            } else {
                return Err(EscrowError::ListingNotFound);
            }

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

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
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
        fn test_create_order() {
            let (accounts, mut escrow) = init();
            let _ = escrow.create_vendor();
            let _ = escrow.create_listing();

            // when listing does not exist
            // * it raises an error
            let mut result = escrow.create_order(1, 5);
            assert_eq!(result, Err(EscrowError::ListingNotFound));
            // when listing exists
            // = when caller is vendor
            // = * it raises an error
            result = escrow.create_order(0, 5);
            assert_eq!(result, Err(EscrowError::Unauthorised));
            // = when caller is not vendor
            test_utils::change_caller(accounts.alice);
            // == when amount to purchase is not available
            // == * it raises an error
            result = escrow.create_order(0, 5);
            assert_eq!(result, Err(EscrowError::AmountUnavailable));
            // == when amount to purchase is available
            test_utils::change_caller(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(5);
            let _ = escrow.deposit_into_listing(0);
            test_utils::change_caller(accounts.alice);
            result = escrow.create_order(0, 5);
            assert!(result.is_ok());
            // == * it reduces the amount_availabe by the amount
            assert_eq!(escrow.listings.values.get(0).unwrap().available_amount, 0);
            // == * it create an order
            let order: Order = escrow.orders.values.get(0).unwrap();
            assert_eq!(order.amount, 5);
            assert_eq!(order.buyer, accounts.alice);
            assert_eq!(order.vendor, accounts.bob);
            assert_eq!(order.id, 0);
            assert_eq!(escrow.orders.length, 1);
            assert_eq!(order.status, 0);
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
            assert_eq!(result, Err(EscrowError::Unauthorised));
            // = when listing belongs to caller
            test_utils::change_caller(accounts.bob);
            set_balance(accounts.bob, 10);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1);
            // = * it increases the listing available_amount
            result = escrow.deposit_into_listing(0);
            assert!(result.is_ok());
            assert_eq!(escrow.listings.values.get(0).unwrap().available_amount, 1);
        }

        #[ink::test]
        fn test_update_order_payment_verification() {
            let (accounts, mut escrow) = init();
            let payment_verification: String = "tx-hash-that-proves-payment".to_string();

            // when order does not exist
            // * it raises an error
            let mut result =
                escrow.update_order_payment_verification(0, payment_verification.clone());
            assert_eq!(result, Err(EscrowError::OrderNotFound));
            // when order exists
            let _ = escrow.create_vendor();
            let _ = escrow.create_listing();
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(10);
            let _ = escrow.deposit_into_listing(0);
            test_utils::change_caller(accounts.alice);
            let _ = escrow.create_order(0, 5);
            // = when called by non-buyer
            // = * it raises an error
            test_utils::change_caller(accounts.bob);
            result = escrow.update_order_payment_verification(0, payment_verification.clone());
            assert_eq!(result, Err(EscrowError::Unauthorised));
            // = when called by buyer
            test_utils::change_caller(accounts.alice);
            let mut order: Order = escrow.orders.values.get(0).unwrap();
            // == when order has status finalised
            order.status = 2;
            escrow.orders.update(&order);
            // == * it raises an error
            result = escrow.update_order_payment_verification(0, payment_verification.clone());
            assert_eq!(result, Err(EscrowError::OrderFinalised));
            // == when order has status cancelled
            order.status = 3;
            escrow.orders.update(&order);
            // == * it raises an error
            result = escrow.update_order_payment_verification(0, payment_verification.clone());
            assert_eq!(result, Err(EscrowError::OrderCancelled));
            // == when order has status open
            order.status = 0;
            escrow.orders.update(&order);
            let _ = escrow.update_order_payment_verification(0, payment_verification.clone());
            order = escrow.orders.values.get(0).unwrap();
            // == * it updates the order's tx hash
            assert_eq!(
                order.payment_verification,
                Some(payment_verification.clone())
            );
            // == * it updates the status to PendingVerification
            assert_eq!(order.status, 1);
            // == when order has status PendingVerification
            // == * it updates the order's tx hash
            let payment_verification_two: String = "Hey Joni".to_string();
            let _ = escrow.update_order_payment_verification(0, payment_verification_two.clone());
            order = escrow.orders.values.get(0).unwrap();
            // == * it updates the order's tx hash
            assert_eq!(order.payment_verification, Some(payment_verification_two));
            assert_eq!(order.status, 1);
            // == when order has status Disputed
            order.status = 4;
            escrow.orders.update(&order);
            let _ = escrow.update_order_payment_verification(0, payment_verification.clone());
            order = escrow.orders.values.get(0).unwrap();
            // == * it updates the order's tx hash
            assert_eq!(order.payment_verification, Some(payment_verification));
            // == * it updates the order's tx hash
            assert_eq!(order.status, 1);
        }

        #[ink::test]
        fn test_withdraw_from_listing() {
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
            assert_eq!(result, Err(EscrowError::Unauthorised));
            // = when listing belongs to caller
            test_utils::change_caller(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(5);
            set_balance(accounts.bob, 10);
            let _ = escrow.deposit_into_listing(0);
            // == when amount is less than or equal to the the available_amount
            // == * it sends the amount to the vendor
            result = escrow.withdraw_from_listing(0, 1);
            assert!(result.is_ok());
            assert_eq!(get_balance(accounts.bob), 11);
            // == * it reduces the available amount
            assert_eq!(escrow.listings.values.get(0).unwrap().available_amount, 4);
            // == when amount is greater than the available_amount
            // == * it raises an error
            result = escrow.withdraw_from_listing(0, 5);
            assert_eq!(result, Err(EscrowError::InsufficientFunds));
        }
    }
}
