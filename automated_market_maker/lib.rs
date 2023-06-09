#![cfg_attr(not(feature = "std"), no_std)]

const PRECISION: u128 = 1_000;

#[ink::contract]
pub mod automated_market_maker {
    use ink_prelude::collections::BTreeMap;

    /// Storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct AutomatedMarketMaker {
        trading_fee: Balance, // Percent of trading fees charged on every trade
        token1_balance: BTreeMap<AccountId, Balance>, // Amount of token1 balance of each user
        token2_balance: BTreeMap<AccountId, Balance>, // Amount of token2 balance of each user
        pool_total_token1: Balance, // The amount of token1 locked in the pool
        pool_total_token2: Balance, // The amount of token2 locked in the pool
        total_shares: Balance, // Stores the total amount of share issued for the pool
        shares: BTreeMap<AccountId, Balance>, // Stores the share holding of each user
    }

    #[ink(impl)]
    impl AutomatedMarketMaker {
        /// Instantiating AMM instance
        /// @param _fees: valid interval -> [0,1000)
        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            Self {
                trading_fee: if _fees >= 1000 { 0 } else { _fees },
                ..Default::default()
            }
        }

        /// Providing new liquidity to the pool
        /// Returns the amount of shares issues for locking assets
        #[ink(message)]
        pub fn provide_liquidity(
            &mut self,
            _amount_token1: Balance,
            _amount_token2: Balance,
        ) -> Result<Balance, Error> {
            self.check_valid_amount(&self.token1_balance, _amount_token1)?;

            let _caller = self.env().caller();
            let issued_shares: u128;

            if self.total_shares == 0 {
                issued_shares = 1000 * super::PRECISION;
            } else {
                let share_1 = self.total_shares * _amount_token1 / self.pool_total_token1;
                let share_2 = self.total_shares * _amount_token2 / self.pool_total_token1;

                if share_1 != share_2 {
                    return Err(Error::NonEquivalentValueErr(
                        "No equivalent of value of tokens".to_string(),
                    ));
                }
                issued_shares = share_1;
            };

            let _token_1 = self.token1_balance.get(&_caller).unwrap();
            let _token_2 = self.token1_balance.get(&_caller).unwrap();

            self.token1_balance
                .clone()
                .insert(_caller, _token_1 - _amount_token1);
            self.token2_balance.insert(_caller, _token_2 - _amount_token2);

            self.pool_total_token1 += _amount_token1;
            self.pool_total_token2 += _amount_token2;
            self.total_shares += issued_shares;

            self.shares
                .entry(_caller)
                .and_modify(|value| *value += issued_shares)
                .or_insert(issued_shares);

            Ok(issued_shares)
        }

        /// Returns amount of token1 required when providing liquidity with _amount_token2 quantity of token2
        #[ink(message)]
        pub fn get_equivalent_token1_estimate(&self, _amount_token2: Balance) -> Result<Balance, Error> {
            self.restrict_liquidity_in_pool()?;
            Ok(self.pool_total_token1 * _amount_token2 / self.pool_total_token2)
        }

        /// Returns amount of token2 required when providing liquidity with _amount_token1 quantity of token1
        #[ink(message)]
        pub fn get_equivalent_token2_estimate(&self, _amount_token1: Balance) -> Result<Balance, Error> {
            self.restrict_liquidity_in_pool()?;
            Ok(self.pool_total_token2 * _amount_token1 / self.pool_total_token1)
        }

        #[ink(message)]
        pub fn estimate_swap_token1_for_given_token1(
            &self,
            _amount_token_1: Balance,
        ) -> Result<Balance, Error> {
            self.restrict_liquidity_in_pool()?;
            let _amount_token1 = (1000 - self.trading_fee) * _amount_token_1 / 1000;

            let token_1_after_trade = self.pool_total_token1 + _amount_token1;
            let token_2_after_trade = self.get_k() / token_1_after_trade;

            let amount_token_2 = self.pool_total_token2 - token_2_after_trade;

            if amount_token_2 == self.pool_total_token2 {
                return Err(Error::PoolDepleted("Pool depleted".to_string()));
            }

            Ok(amount_token_2)
        }

        /// Returns the amount of token2 that the user will get swapping a given amount of token1 for token2
        #[ink(message)]
        pub fn estimate_swap_token1_for_given_token2(
            &self,
            _amount_token1: Balance,
        ) -> Result<Balance, Error> {
            self.restrict_liquidity_in_pool()?;
            let _amount_token1 = (1000 - self.trading_fee) * _amount_token1 / 1000;

            let token1_after = self.pool_total_token1 + _amount_token1;
            let token2_after = self.get_k() / token1_after;
            let amount_token1 = self.pool_total_token2 - token2_after;

            Ok(amount_token1)
        }

        /// Returns the amount of token1 that the user should swap to get _amount_token2 in return
        #[ink(message)]
        pub fn swap_token1_for_given_token2(&self, _amount_token2: Balance) -> Result<Balance, Error> {
            self.restrict_liquidity_in_pool()?;

            if _amount_token2 >= self.pool_total_token2 {
                return Err(Error::InsufficientLiquidityErr(
                    "No sufficient pool balance".to_string(),
                ));
            }

            let token2_after = self.pool_total_token2 - _amount_token2;
            let token1_after = self.get_k() / token2_after;
            let amount_token1 = (token1_after - self.pool_total_token1) * 1000 / (1000 - self.trading_fee);

            Ok(amount_token1)
        }

        /// Returns estimation of token1 and token2 that will be released on burning given _share
        #[ink(message)]
        pub fn get_withdraw_estimation(&self, _share: Balance) -> Result<(Balance, Balance), Error> {
            self.restrict_liquidity_in_pool()?;

            if _share > self.total_shares {
                return Err(Error::InvalidShareErr(
                    "Amount of shares should be greater than total shares".to_string(),
                ));
            }

            let amount_token1 = _share * self.pool_total_token1 / self.total_shares;
            let amount_token2 = _share * self.pool_total_token2 / self.total_shares;

            Ok((amount_token1, amount_token2))
        }

        /// Removes liquidity from the pool and releases corresponding token_1 and token_2 to the withdrawer
        #[ink(message)]
        pub fn withdraw(&mut self, _share: Balance) -> Result<(Balance, Balance), Error> {
            let _caller = self.env().caller();
            self.check_valid_amount(&self.shares, _share)?;

            let (amount_token1, amount_token2) = self.get_withdraw_estimation(_share)?;
            self.shares.entry(_caller).and_modify(|value| *value -= _share);
            self.total_shares -= _share;

            self.pool_total_token1 -= amount_token1;
            self.pool_total_token2 -= amount_token2;

            self.token1_balance
                .entry(_caller)
                .and_modify(|value| *value += amount_token1);
            self.token2_balance
                .entry(_caller)
                .and_modify(|value| *value += amount_token2);

            Ok((amount_token1, amount_token2))
        }

        /// Ensure that quantity is non-zero and user has enough balance
        fn check_valid_amount(
            &self,
            _balance: &BTreeMap<AccountId, Balance>,
            _quantity: Balance,
        ) -> Result<(), Error> {
            let _caller = self.env().caller();
            let balance_user = _balance.get(&_caller).unwrap_or(&0);

            match _quantity {
                0 => Err(Error::ZeroAmountErr("Value cannot be zero!".to_string())),
                _ if (_quantity > *balance_user) => Err(Error::InsufficientAmountErr(
                    "You have no sufficient amount of value".to_string(),
                )),
                _ => Ok(()),
            }
        }

        /// Sends free token(s) to the invoker
        #[ink(message)]
        pub fn faucet_brrr(&mut self, _amount_token1: Balance, _amount_token2: Balance) {
            let _caller = self.env().caller();
            let token1 = self.token1_balance.get(&_caller).unwrap_or(&0);
            let token2 = self.token2_balance.get(&_caller).unwrap_or(&0);

            self.token1_balance.insert(_caller, token1 + _amount_token1);
            self.token2_balance.insert(_caller, token2 + _amount_token2);
        }

        /// Returns the liquidity constant K value of a pool
        fn get_k(&self) -> Balance {
            self.pool_total_token1 * self.pool_total_token2
        }

        /// Restriction of withdrawing and swapping till liquidity is added to a pool
        fn restrict_liquidity_in_pool(&self) -> Result<(), Error> {
            match self.get_k() {
                0 => Err(Error::ZeroLiquidityErr(
                    "You have no liquidity and there is no way to make brrr".to_string(),
                )),
                _ => Ok(()),
            }
        }

        /// Returns the balance of a user
        pub fn get_information_portfolio(&self) -> (Balance, Balance, Balance) {
            let _caller = self.env().caller();
            let token_1 = self.token1_balance.get(&_caller).unwrap_or(&0);
            let token_2 = self.token2_balance.get(&_caller).unwrap_or(&0);
            let user_shares = self.shares.get(&_caller).unwrap_or(&0);

            (*token_1, *token_2, *user_shares)
        }

        /// Returns the amount of tokens locked in the pool, total shares issued and trading fee parameter
        pub fn get_pool_details(&self) -> (Balance, Balance, Balance, Balance) {
            (
                self.pool_total_token1,
                self.pool_total_token2,
                self.total_shares,
                self.trading_fee,
            )
        }
    } //---LAST LINE OF IMPLEMENTATION OF THE INK! SMART CONTRACT---//

    /// Errors definitions
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        PoolDepleted(String),
        ZeroAmountErr(String),
        InvalidShareErr(String),
        ZeroLiquidityErr(String),
        SlippageExceededErr(String),
        InsufficientAmountErr(String),
        NonEquivalentValueErr(String),
        ThresholdNotReachedErr(String),
        InsufficientLiquidityErr(String),
    }
}
