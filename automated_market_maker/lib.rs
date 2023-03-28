#![cfg_attr(not(feature = "std"), no_std)]

const PRECISION: u128 = 1_000_000;


#[ink::contract]
mod automated_market_maker {
    use ink::storage::{
        Mapping,
    };

#[derive(Default)]
    #[ink(storage)]
    pub struct AutomatedMarketMaker {
        totalShares: Balance, // Stores the total amount of share issued for the pool
        totalToken1: Balance, // Stores the amount of Token1 locked in the pool
        totalToken2: Balance, // Stores the amount of Token2 locked in the pool
        shares: Mapping<AccountId, Balance>, // Stores the share holding of each provider
        token1Balance: Mapping<AccountId, Balance>, // Stores the token1 balance of each user
        token2Balance: Mapping<AccountId, Balance>, // Stores the token2 balance of each user
        fees: Balance,        // Percent of trading fees charged on trade
    }

    impl AutomatedMarketMaker {

        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            Self {
                fees: if _fees >= 1000 { 0 } else { _fees },
                ..Default::default()
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            todo!();
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            todo!();
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let automated_market_maker = AutomatedMarketMaker::default();
            assert_eq!(automated_market_maker.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut automated_market_maker = AutomatedMarketMaker::new(false);
            assert_eq!(automated_market_maker.get(), false);
            automated_market_maker.flip();
            assert_eq!(automated_market_maker.get(), true);
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ZeroAmount,
        ZeroLiquidity,
        InsufficientAmount,
        NonEquivalentValue,
        ThresholdNotReached,
        InvalidShare,
        InsufficientLiquidity,
        SlippageExceeded,
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = AutomatedMarketMakerRef::default();

            // When
            let contract_account_id = client
                .instantiate("automated_market_maker", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<AutomatedMarketMakerRef>(contract_account_id.clone())
                .call(|automated_market_maker| automated_market_maker.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = AutomatedMarketMakerRef::new(false);
            let contract_account_id = client
                .instantiate("automated_market_maker", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<AutomatedMarketMakerRef>(contract_account_id.clone())
                .call(|automated_market_maker| automated_market_maker.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<AutomatedMarketMakerRef>(contract_account_id.clone())
                .call(|automated_market_maker| automated_market_maker.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<AutomatedMarketMakerRef>(contract_account_id.clone())
                .call(|automated_market_maker| automated_market_maker.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
