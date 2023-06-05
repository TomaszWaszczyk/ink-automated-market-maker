#[cfg(test)]
mod tests {
    use automated_market_maker::automated_market_maker::AutomatedMarketMaker;

    #[test]
    fn create_new_contract_test() {
        let amm_contract = AutomatedMarketMaker::new(0);
    }
}
