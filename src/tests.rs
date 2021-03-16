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
			let expand_by = 110_000;
			let price = 89;
			assert_ok!(SerpMarket::expand_supply(Origin::root(), STP258_TOKEN_ID, expand_by, price)); 
			assert_eq!(
				Stp258Tokens::total_issuance(STP258_TOKEN_ID), 
				prev_supply.saturating_add(expand_by),
			"supply should be increased by expand_by"
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
			let contract_by = 100_000;
			let price = 20;
			assert_ok!(SerpMarket::contract_supply(Origin::root(), STP258_TOKEN_ID, contract_by, price)); 
			assert_eq!(
				Stp258Tokens::total_issuance(STP258_TOKEN_ID), 
				prev_supply.saturating_sub(contract_by),
			"supply should be decreased by contract_by"
			);
		});
}
