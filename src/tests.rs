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
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 148_900);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 250_000 * 1_000);
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
			assert_eq!(SerpMarket::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 155_500);
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 50_000 * 1_000);
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 1_005_500);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 900_000 * 1_000);
		});
}

#[test]
fn stp258_currency_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SerpMarket::set_lock(ID_1, STP258_TOKEN_ID, &ALICE, 50 * 1000));
			assert_eq!(Stp258Tokens::locks(&ALICE, STP258_TOKEN_ID).len(), 1);
			assert_ok!(SerpMarket::set_lock(ID_1, STP258_NATIVE_ID, &ALICE, 50  * 1000));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
		});
}

#[test]
fn stp258_currency_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(SerpMarket::total_issuance(STP258_NATIVE_ID), 200);
			assert_eq!(SerpMarket::total_issuance(STP258_TOKEN_ID), 200 * 1000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 100 * 1000);
			assert_eq!(Stp258Native::free_balance(&ALICE), 100);

			assert_ok!(SerpMarket::reserve(STP258_TOKEN_ID, &ALICE, 30 * 1000));
			assert_ok!(SerpMarket::reserve(STP258_NATIVE_ID, &ALICE, 40));
			assert_eq!(SerpMarket::reserved_balance(STP258_TOKEN_ID, &ALICE), 30 * 1000);
			assert_eq!(SerpMarket::reserved_balance(STP258_NATIVE_ID, &ALICE), 40);
		});
}

#[test]
fn stp258_native_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(Stp258Native::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn stp258_native_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::reserve(&ALICE, 50));
			assert_eq!(Stp258Native::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_lockable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(AdaptedStp258Asset::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_reservable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::reserve(&ALICE, 50));
			assert_eq!(AdaptedStp258Asset::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn stp258_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SerpMarket::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
		});
}

#[test]
fn stp258_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(<SerpMarket as Stp258CurrencyExtended<AccountId>>::update_balance(
				STP258_TOKEN_ID, &ALICE, 50 * 1000
			));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 150 * 1000);
		});
}

#[test]
fn stp258_native_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SerpMarket::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(Stp258Native::free_balance(&ALICE), 50);
			assert_eq!(Stp258Native::free_balance(&BOB), 150);

			assert_ok!(Stp258Native::transfer(&ALICE, &BOB, 10));
			assert_eq!(Stp258Native::free_balance(&ALICE), 40);
			assert_eq!(Stp258Native::free_balance(&BOB), 160);

			assert_eq!(SerpMarket::slash(STP258_NATIVE_ID, &ALICE, 10), 0);
			assert_eq!(Stp258Native::free_balance(&ALICE), 30);
			assert_eq!(Stp258Native::total_issuance(), 190);
		});
}

#[test]
fn stp258_native_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::update_balance(&ALICE, 10));
			assert_eq!(Stp258Native::free_balance(&ALICE), 110);

			assert_ok!(<SerpMarket as Stp258CurrencyExtended<AccountId>>::update_balance(
				STP258_NATIVE_ID,
				&ALICE,
				10
			));
			assert_eq!(Stp258Native::free_balance(&ALICE), 120);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_transfer() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::transfer(&ALICE, &BOB, 50));
			assert_eq!(PalletBalances::total_balance(&ALICE), 50);
			assert_eq!(PalletBalances::total_balance(&BOB), 150);

			// creation fee
			assert_ok!(AdaptedStp258Asset::transfer(&ALICE, &EVA, 10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 40);
			assert_eq!(PalletBalances::total_balance(&EVA), 10);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_deposit() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::deposit(&EVA, 50));
			assert_eq!(PalletBalances::total_balance(&EVA), 50);
			assert_eq!(PalletBalances::total_issuance(), 250);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_withdraw() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::withdraw(&ALICE, 100));
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_slash() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(AdaptedStp258Asset::slash(&ALICE, 101), 1);
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_update_balance() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::update_balance(&ALICE, -10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 90);
			assert_eq!(PalletBalances::total_issuance(), 190);
		});
}

#[test]
fn update_balance_call_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SerpMarket::update_balance(
				Origin::root(),
				ALICE,
				STP258_NATIVE_ID,
				-10
			));
			assert_eq!(Stp258Native::free_balance(&ALICE), 90);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 100 * 1000);
			assert_ok!(SerpMarket::update_balance(Origin::root(), ALICE, STP258_TOKEN_ID, 10 * 1000));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 110 * 1000);
		});
}

#[test]
fn update_balance_call_fails_if_not_root_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SerpMarket::update_balance(Some(ALICE).into(), ALICE, STP258_TOKEN_ID, 100 * 1000),
			BadOrigin
		);
	});
}

#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(SerpMarket::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);

			let transferred_event = Event::serp_market(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 50 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<SerpMarket as Stp258Currency<AccountId>>::transfer(
				STP258_TOKEN_ID, &ALICE, &BOB, 10 * 1000
			));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 40 * 1000);
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &BOB), 160 * 1000);

			let transferred_event = Event::serp_market(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 10 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<SerpMarket as Stp258Currency<AccountId>>::deposit(
				STP258_TOKEN_ID, &ALICE, 100 * 1000
			));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 140 * 1000);

			let transferred_event = Event::serp_market(crate::Event::Deposited(STP258_TOKEN_ID, ALICE, 100 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<SerpMarket as Stp258Currency<AccountId>>::withdraw(
				STP258_TOKEN_ID, &ALICE, 20 * 1000
			));
			assert_eq!(SerpMarket::free_balance(STP258_TOKEN_ID, &ALICE), 120 * 1000);

			let transferred_event = Event::serp_market(crate::Event::Withdrawn(STP258_TOKEN_ID, ALICE, 20 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
