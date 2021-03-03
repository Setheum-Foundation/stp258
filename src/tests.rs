//! Unit tests for the Stp258 module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
		let prev_supply = Stp258::settcurrency_supply(X_TOKEN_ID);
		let amount = 13 * BaseUnit::get();
		assert_ok!(Stp258::expand_supply(prev_supply, amount));
		assert_eq!(Stp258::total_issuance(X_TOKEN_ID), settcurrency_supply + amount);
		assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 100);
		assert_eq!(Stp258::free_balance(X_TOKEN_ID, &BOB), 100);
		assert_eq!(
			Stp258::settcurrency_supply(X_TOKEN_ID),
			prev_supply + amount,
			"supply should be increased by amount"
		);
	});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
		let prev_supply = SettCurrency::settcurrency_supply(X_TOKEN_ID);
		let amount = 2 * BaseUnit::get();
		assert_ok!(SettCurrency::contract_supply(prev_supply, amount));
		assert_eq!(
			SettCurrency::settcurrency_supply(X_TOKEN_ID),
			prev_supply - amount,
			"supply should be decreased by amount"
		);
	})
}

#[test]
fn currency_adapter_burn_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let init_total_issuance = AdaptedBasicCurrency::total_issuance();
			let imbalance = AdaptedBasicCurrency::burn(10);
			assert_eq!(AdaptedBasicCurrency::total_issuance(), init_total_issuance - 10);
			drop(imbalance);
			assert_eq!(AdaptedBasicCurrency::total_issuance(), init_total_issuance);
		});
}

#[test]
fn currency_adapter_issue_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let init_total_issuance = AdaptedBasicCurrency::total_issuance();
			let imbalance = AdaptedBasicCurrency::issue(22);
			assert_eq!(AdaptedBasicCurrency::total_issuance(), init_total_issuance + 22);
			drop(imbalance);
			assert_eq!(AdaptedBasicCurrency::total_issuance(), init_total_issuance);
		});
}

#[test]
fn sett_currency_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::set_lock(ID_1, X_TOKEN_ID, &ALICE, 50));
			assert_eq!(Tokens::locks(&ALICE, X_TOKEN_ID).len(), 1);
			assert_ok!(Stp258::set_lock(ID_1, NATIVE_CURRENCY_ID, &ALICE, 50));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
		});
}

#[test]
fn sett_currency_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Stp258::total_issuance(NATIVE_CURRENCY_ID), 200);
			assert_eq!(Stp258::total_issuance(X_TOKEN_ID), 200);
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 100);
			assert_eq!(NativeCurrency::free_balance(&ALICE), 100);

			assert_ok!(Stp258::reserve(X_TOKEN_ID, &ALICE, 30));
			assert_ok!(Stp258::reserve(NATIVE_CURRENCY_ID, &ALICE, 40));
			assert_eq!(Stp258::reserved_balance(X_TOKEN_ID, &ALICE), 30);
			assert_eq!(Stp258::reserved_balance(NATIVE_CURRENCY_ID, &ALICE), 40);
		});
}

#[test]
fn native_currency_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(NativeCurrency::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn native_currency_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::reserve(&ALICE, 50));
			assert_eq!(NativeCurrency::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_lockable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(AdaptedBasicCurrency::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_reservable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::reserve(&ALICE, 50));
			assert_eq!(AdaptedBasicCurrency::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn sett_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::transfer(Some(ALICE).into(), BOB, X_TOKEN_ID, 50));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 50);
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &BOB), 150);
		});
}

#[test]
fn sett_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(<Stp258 as SettCurrencyExtended<AccountId>>::update_balance(
				X_TOKEN_ID, &ALICE, 50
			));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 150);
		});
}

#[test]
fn native_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 50);
			assert_eq!(NativeCurrency::free_balance(&BOB), 150);

			assert_ok!(NativeCurrency::transfer(&ALICE, &BOB, 10));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 40);
			assert_eq!(NativeCurrency::free_balance(&BOB), 160);

			assert_eq!(Stp258::slash(NATIVE_CURRENCY_ID, &ALICE, 10), 0);
			assert_eq!(NativeCurrency::free_balance(&ALICE), 30);
			assert_eq!(NativeCurrency::total_issuance(), 190);
		});
}

#[test]
fn native_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::update_balance(&ALICE, 10));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 110);

			assert_ok!(<Stp258 as SettCurrencyExtended<AccountId>>::update_balance(
				NATIVE_CURRENCY_ID,
				&ALICE,
				10
			));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 120);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_transfer() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::transfer(&ALICE, &BOB, 50));
			assert_eq!(PalletBalances::total_balance(&ALICE), 50);
			assert_eq!(PalletBalances::total_balance(&BOB), 150);

			// creation fee
			assert_ok!(AdaptedBasicCurrency::transfer(&ALICE, &EVA, 10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 40);
			assert_eq!(PalletBalances::total_balance(&EVA), 10);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_deposit() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::deposit(&EVA, 50));
			assert_eq!(PalletBalances::total_balance(&EVA), 50);
			assert_eq!(PalletBalances::total_issuance(), 250);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_withdraw() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::withdraw(&ALICE, 100));
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_slash() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(AdaptedBasicCurrency::slash(&ALICE, 101), 1);
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_update_balance() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::update_balance(&ALICE, -10));
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
			assert_ok!(Stp258::update_balance(
				Origin::root(),
				ALICE,
				NATIVE_CURRENCY_ID,
				-10
			));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 90);
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 100);
			assert_ok!(Stp258::update_balance(Origin::root(), ALICE, X_TOKEN_ID, 10));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 110);
		});
}

#[test]
fn update_balance_call_fails_if_not_root_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stp258::update_balance(Some(ALICE).into(), ALICE, X_TOKEN_ID, 100),
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

			assert_ok!(Stp258::transfer(Some(ALICE).into(), BOB, X_TOKEN_ID, 50));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 50);
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &BOB), 150);

			let transferred_event = Event::stp258(crate::Event::Transferred(X_TOKEN_ID, ALICE, BOB, 50));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::transfer(
				X_TOKEN_ID, &ALICE, &BOB, 10
			));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 40);
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &BOB), 160);

			let transferred_event = Event::stp258(crate::Event::Transferred(X_TOKEN_ID, ALICE, BOB, 10));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::deposit(
				X_TOKEN_ID, &ALICE, 100
			));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 140);

			let transferred_event = Event::stp258(crate::Event::Deposited(X_TOKEN_ID, ALICE, 100));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::withdraw(
				X_TOKEN_ID, &ALICE, 20
			));
			assert_eq!(Stp258::free_balance(X_TOKEN_ID, &ALICE), 120);

			let transferred_event = Event::stp258(crate::Event::Withdrawn(X_TOKEN_ID, ALICE, 20));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
