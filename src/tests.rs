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
			assert_ok!(SerpMarket::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(SerpMarket::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::expand_supply(Origin::root(), STP258_TOKEN_ID, 110_000 * 1000, 89 * 1000)); 
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 460_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 148_900);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 998_900);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_ok!(SerpMarket::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(Stp258Tokens::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::contract_supply(Origin::root(), STP258_TOKEN_ID, 100_000 * 1000, 20 * 1000)); 
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 355_500);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 50_000 * 1_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_005_500);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 900_000 * 1_000);
		});
}
