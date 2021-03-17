//! Unit tests for the SerpMarket module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;


#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			let prev_supply = Stp258Tokens::total_issuance(STP258_TOKEN_ID);
			let prev_reserved_balance = Stp258Tokens::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC);
			let prev_free_balance = Stp258Tokens::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC);
			let expand_by = 100_000;
			let quote_price = 90;
			let supply = Stp258Tokens::total_issuance(STP258_TOKEN_ID);
			// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
			// ↑ verify ↑
			let serper = &SERPER_ACC; 
			let new_supply = supply + expand_by; 
			let base_price = new_supply / supply;
			let _base_unit = 1_000;
			let serp_quote_multiple = 2;
			let fraction = base_price - 1;
			let quotation = fraction * serp_quote_multiple;
			let serp_quoted_price =  base_price - quotation;
			let price = quote_price / serp_quoted_price;
			let pay_by_quoted = expand_by / price;
			assert_ok!(SerpMarket::expand_supply(Origin::root(), STP258_TOKEN_ID, expand_by, quote_price)); 
			assert_eq!(
				Stp258Tokens::total_issuance(STP258_TOKEN_ID), 
				prev_supply + expand_by,
			"supply should be increased by expand_by"
			);
			assert_eq!(
				Stp258Tokens::free_balance(STP258_TOKEN_ID, serper),
				prev_free_balance + expand_by,
				"reserved balance should be decreased by contract_by"
			);
			assert_eq!(
				Stp258Tokens::reserved_balance(STP258_NATIVE_ID, serper),
				prev_reserved_balance - pay_by_quoted,
				"reserved balance should be decreased by contract_by"
			);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			let prev_supply = Stp258Tokens::total_issuance(STP258_TOKEN_ID);
			let prev_reserved_balance = Stp258Tokens::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC);
			let contract_by = 100_000;
			let quote_price = 20;
			let base_price = 900;
			// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
			// ↑ verify ↑
			let serper = &SERPER_ACC; 
			let _base_unit = 1_000;
			let serp_quote_multiple = 2;
			let fraction = base_price - 1;
			let quotation = fraction * serp_quote_multiple;
			let serp_quoted_price =  base_price + quotation;
			let price = serp_quoted_price / quote_price;
			let pay_by_quoted = price * contract_by;
			assert_ok!(SerpMarket::contract_supply(Origin::root(), STP258_TOKEN_ID, contract_by, quote_price, base_price)); 
			assert_eq!(
				Stp258Tokens::total_issuance(STP258_TOKEN_ID), 
				prev_supply.checked_sub(contract_by),
			"supply should be decreased by contract_by"
			);
			assert_eq!(
				Stp258Tokens::reserved_balance(STP258_TOKEN_ID, serper),
				prev_reserved_balance - contract_by,
				"reserved balance should be decreased by contract_by"
			)
		});
}
