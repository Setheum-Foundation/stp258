//! Unit tests for the SerpMarket module.

#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn get_stable_price_should_work() {
	assert_ok!(SerpMarket::get_stable_price(STP258_TOKEN_ID, 1.1));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 1.1 * 1_100);
	assert_ok!(SerpMarket::get_stable_price(STP258_JCHF_ID, 1));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 1 * 1_000);
	assert_ok!(SerpMarket::get_stable_price(STP258_JUSD_ID, 0.9));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 0.9 * 1_000);
}

#[test]
fn get_relative_price_should_work() {
	assert_ok!(SerpMarket::get_relative_price(STP258_TOKEN_ID, 1.3, STP258_NATIVE_ID, 0.065 * 1_000));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 20);
	assert_ok!(SerpMarket::get_relative_price(STP258_NATIVE_ID, 2_000, STP258_JCHF_ID, 1));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 1);
	assert_ok!(SerpMarket::get_relative_price(STP258_JUSD_ID, 1.1599, STP258_JCHF_ID, 0.92));
	assert_eq!(SerpMarket::price(STP258_TOKEN_ID), 1.260);
}

#[test]
fn quoting_serp_price_should_work() {
	let price = 1.3 * 1_000;
	let serp_quote_multiple = SerpMarket::GetSerpQuoteMultiple;
	let serp_quoted_price = SerpMarket::quote_serp_price(price);
	assert_eq!(serp_quoted_price, * 1_000);
}

#[test]
fn calculate_supply_change_should_work() {
let price = 1_000 + 100;
	let supply = u64::max_value();
	let contract_by = SerpMarket::calculate_supply_change(price);
	// the error should be low enough
	assert_eq!(contract_by, u64::max_value() / 10 - 1);
	assert_eq!(contract_by, u64::max_value() / 10 + 1);
}

#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_ok!(PalletBalances::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(SerpMarket::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::expand_supply(STP258_TOKEN_ID, 100_000 * 1000)); 
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 575_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 169_230.769);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 175_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(Stp258Tokens::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::contract_supply(STP258_TOKEN_ID, 10_000 * 1000)); 
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Native::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 170_600);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 140_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);
		});
}

#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(SerpMarket::get_stable_price(STP258_TOKEN_ID, 1.1) 1.1 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.1 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		
			assert_ok!(Stp258Native::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_ok!(Stp258Tokens::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_ok!(SerpMarket::expand_supply(STP258_TOKEN_ID, 100_000 * 1000)); 
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 575_000 * 1_000);
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 169_230.769);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 175_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::SerpedUpSupply(STP258_TOKEN_ID, 100_000 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
			
			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.3 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));


			assert_ok!(Stp258Native::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_ok!(SerpMarket::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_ok!(SerpMarket::contract_supply(STP258_TOKEN_ID, 10_000 * 1000)); 
			assert_eq!(Stp258Native::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 170_600);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 140_000 * 1_000);
			assert_eq!(Stp258Native::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::SerpedDownSupply(STP258_TOKEN_ID, 10_000 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
			
			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.03 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
