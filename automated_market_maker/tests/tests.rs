#[cfg(test)]
mod tests {
    use automated_market_maker::automated_market_maker::AutomatedMarketMaker;
    use ink_env::{test::{set_callee, set_caller}, DefaultEnvironment};
    use ink_primitives::AccountId;

    #[test]
    fn create_new_contract_test() {
        let mut _amm_contract = AutomatedMarketMaker::new(0);
        let _accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
        let contract_addr: AccountId = AccountId::from([0xFF as u8; 32]);
        set_callee::<DefaultEnvironment>(contract_addr);
        ink_env::test::set_caller::<ink_env::DefaultEnvironment>(_accounts.alice);

        // // Set the contract as callee and Bob as caller.
        // let _contract = ink_env::account_id::<ink_env::DefaultEnvironment>();
        // ink_env::test::set_callee::<ink_env::DefaultEnvironment>(_contract);
        // ink_env::test::set_caller::<ink_env::DefaultEnvironment>(_accounts.alice);

        // let mut data =
        // ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
        
        // ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
        //     _accounts.bob,
        //     callee,
        //     1000000,
        //     1000000,
        //     data,
        // );





        _amm_contract.faucet(10, 20);
        assert_eq!(_amm_contract.get_information_portfolio(), (10, 20, 0));
    }
}
