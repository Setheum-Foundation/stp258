
#[test]
fn calculate_supply_change_should_work() {
let price = 1_000 + 100;
	let supply = u64::max_value();
	let contract_by = SerpMarket::calculate_supply_change(price);
	// the error should be low enough
	assert_eq!(contract_by, u64::max_value() / 10 - 1);
	assert_eq!(contract_by, u64::max_value() / 10 + 1);
}
