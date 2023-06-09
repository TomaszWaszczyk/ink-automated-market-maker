#[cfg(test)]
mod tests {
    use automated_market_maker::automated_market_maker::AutomatedMarketMaker;
    use ink_env::{
        test::{set_callee, set_caller},
        DefaultEnvironment,
    };
    use ink_primitives::AccountId;

    #[test]
    fn create_new_contract_test() {
        let mut _amm_contract = AutomatedMarketMaker::new(0);
        let _accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
        let _contract_addr: AccountId = AccountId::from([0xFF as u8; 32]);

        set_callee::<DefaultEnvironment>(_contract_addr);
        set_caller::<ink_env::DefaultEnvironment>(_accounts.alice);
    }

    #[test]
    fn should_activate_brrr_test() {
        let mut _amm_contract = AutomatedMarketMaker::new(0);
        let _accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
        let _contract_addr: AccountId = AccountId::from([0xFF as u8; 32]);
        set_callee::<DefaultEnvironment>(_contract_addr);
        set_caller::<ink_env::DefaultEnvironment>(_accounts.alice);

        _amm_contract.faucet_brrr(10, 20);
        
        assert_eq!(_amm_contract.get_information_portfolio(), (10, 20, 0));
    }

    #[test]
    fn should_provide_liquidity_test() {
        let mut _amm_contract = AutomatedMarketMaker::new(0);
        let _accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
        let _contract_addr: AccountId = AccountId::from([0xFF as u8; 32]);
        set_callee::<DefaultEnvironment>(_contract_addr);
        set_caller::<ink_env::DefaultEnvironment>(_accounts.alice);

        _amm_contract.faucet_brrr(10, 20);

        let _provided_liquidity = _amm_contract.provide_liquidity(1, 2).unwrap();

        assert_eq!(_provided_liquidity, 1_000_000);
    }
}
