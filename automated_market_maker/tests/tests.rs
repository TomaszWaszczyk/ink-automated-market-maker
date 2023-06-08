#[cfg(test)]
mod tests {
    use automated_market_maker::automated_market_maker::AutomatedMarketMaker;

    #[test]
    fn create_new_contract_test() {
        let mut _amm_contract = AutomatedMarketMaker::new(0);
        let _accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
        // amm_contract.faucet(10, 20);
        // assert_eq!(amm_contract.get_information_portfolio(), (10, 20, 0));
    }
}
