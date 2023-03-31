#![cfg_attr(not(feature = "std"), no_std)]

const PRECISION: u128 = 1_000_000;


#[ink::contract]
mod automated_market_maker {
    use ink::storage::Mapping;


    #[ink(impl)]
    impl AutomatedMarketMaker {
        /// Constructs a new AMM instance
        /// @param _fees: valid interval -> [0,1000)
        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            Self {
                fees: if _fees >= 1000 { 0 } else { _fees },
                ..Default::default()
            }
        }

        /// Providing new liquidity to the pool
        /// Returns the amount of shares issues for locking assets
        #[ink(message)]
        pub fn provide_liquidity(&mut self, _amount_token1: Balance, _amount_token2: Balance) -> Result<Balance, Error> {
            // self.valid_amount_check(&self.token1_balance, _amount_token1);
            todo!()
        }

        fn valid_amount_check(&self, _balance: Mapping<AccountId, Balance>, _qty: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            let my_balance = _balance.get(&caller).unwrap_or(0);

            match _qty {
                0 => Err(Error::ZeroAmount),
                _ if _qty > my_balance => Err(Error::InsufficientAmount),
                _ => Ok(()),
            }
        }

        /// Sends free token(s) to the invoker
        #[ink(message)]
        pub fn faucet(&mut self, _amount_token1: Balance, _amount_token2: Balance) {
            let caller = self.env().caller();
            let token1 = self.token1_balance.get(&caller).unwrap_or(0);
            let token2 = self.token2_balance.get(&caller).unwrap_or(0);

            self.token1_balance.insert(caller, &(token1 + _amount_token1));
            self.token2_balance.insert(caller, &(token2 + _amount_token2));
        }

        /// Returns the liquidity constant of the pool
        fn get_k(&self) -> Balance {
            self.total_token1 * self.total_token2
        }

        /// Restriction of withdrawing and swapping feature till liquidity is added to the pool
        fn active_pool(&self) -> Result<(), Error> {
            match self.get_k() {
                0 => Err(Error::ZeroLiquidity),
                _ => Ok(()),
            }
        }

        /// Returns the balance of the user
        pub fn get_info_about_holdings(&self) -> (Balance, Balance, Balance) {
            let caller = self.env().caller();
            let token_1 = self.token1_balance.get(&caller).unwrap_or(0);
            let token_2 = self.token2_balance.get(&caller).unwrap_or(0);
            let my_shares = self.shares.get(&caller).unwrap_or(0);

            (token_1, token_2, my_shares)
        }
        
        /// Returns the amount of tokens locked in the pool, total shares issued and trading fee parameter
        pub fn get_pool_details(&self) -> (Balance, Balance, Balance, Balance) {
            (
                self.total_token1,
                self.total_token2,
                self.total_shares,
                self.fees,
            )
        }
    }

    /// Storage struct
    #[derive(Default)]
    #[ink(storage)]
    pub struct AutomatedMarketMaker {
        total_shares: Balance,                       // Stores the total amount of share issued for the pool
        total_token1: Balance,                       // Stores the amount of Token1 locked in the pool
        total_token2: Balance,                       // Stores the amount of Token2 locked in the pool
        shares: Mapping<AccountId, Balance>,         // Stores the share holding of each provider
        token1_balance: Mapping<AccountId, Balance>, // Stores the token1 balance of each user
        token2_balance: Mapping<AccountId, Balance>, // Stores the token2 balance of each user
        fees: Balance,                               // Percent of trading fees charged on trade
    }

    /// Errors definitions
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
                .instantiate(
                    "automated_market_maker",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
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
                .instantiate(
                    "automated_market_maker",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
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
