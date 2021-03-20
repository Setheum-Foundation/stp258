#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Codec;
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency as SetheumCurrency, ExistenceRequirement, Get, 
		LockableCurrency as SetheumLockableCurrency,
		ReservableCurrency as SetheumReservableCurrency, WithdrawReasons,
	},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use stp258_traits::{
	account::MergeAccount,
	arithmetic::{Signed, SimpleArithmetic},
	BalanceStatus, SerpMarket, Stp258Asset, Stp258AssetExtended, Stp258AssetLockable, Stp258AssetReservable,
	LockIdentifier, Stp258Currency, Stp258CurrencyExtended, Stp258CurrencyReservable, Stp258CurrencyLockable,
};
use orml_utilities::with_transaction_result;
use sp_runtime::{
	traits::{CheckedSub,  MaybeSerializeDeserialize, StaticLookup, Zero},
	DispatchError, DispatchResult,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker, result,
};

mod default_weight;
mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub trait WeightInfo {
		fn transfer_non_native_currency() -> Weight;
		fn transfer_native_currency() -> Weight;
		fn update_balance_non_native_currency() -> Weight;
		fn update_balance_native_currency_creating() -> Weight;
		fn update_balance_native_currency_killing() -> Weight;
	}

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Stp258Currency as Stp258Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::Stp258Currency as Stp258Currency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type AmountOf<T> =
		<<T as Config>::Stp258Currency as Stp258CurrencyExtended<<T as frame_system::Config>::AccountId>>::Amount;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Stp258Currency: MergeAccount<Self::AccountId>
			+ Stp258CurrencyExtended<Self::AccountId>
			+ Stp258CurrencyLockable<Self::AccountId>
			+ Stp258CurrencyReservable<Self::AccountId>;

		type Stp258Native: Stp258AssetExtended<Self::AccountId, Balance = BalanceOf<Self>, Amount = AmountOf<Self>>
			+ Stp258AssetLockable<Self::AccountId, Balance = BalanceOf<Self>>
			+ Stp258AssetReservable<Self::AccountId, Balance = BalanceOf<Self>>;

		
		#[pallet::constant]
		type GetStp258NativeId: Get<CurrencyIdOf<Self>>;


		/// The balance of an account.
		#[pallet::constant]
		type GetBaseUnit: Get<BalanceOf<Self>>;

		/// The single unit of base_units.
		#[pallet::constant]
		type GetSingleUnit: Get<BalanceOf<Self>>;

		/// The Serper ratio type getter
		#[pallet::constant]
		type GetSerperRatio: Get<BalanceOf<Self>>;

		/// The SettPay ratio type getter
		#[pallet::constant]
		type GetSettPayRatio: Get<BalanceOf<Self>>;	

		/// The SettPay Account type
		#[pallet::constant]
		type GetSettPayAcc: Get<Self::AccountId>;

		/// The Serpers Account type
		#[pallet::constant]
		type GetSerperAcc: Get<Self::AccountId>;

		/// The Serp quote multiple type for qUOTE, quoting 
		/// `(mintrate * SERP_QUOTE_MULTIPLE) = SerpQuotedPrice`.
		#[pallet::constant]
		type GetSerpQuoteMultiple: Get<BalanceOf<Self>>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to convert the Amount type into Balance.
		AmountIntoBalanceFailed,
		/// Balance is too low.
		BalanceTooLow,
		// Cannott expand or contract Native Asset, only SettCurrency	Serping.
		CannotSerpNativeAssetOnlySerpSettCurrency,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency transfer success. [currency_id, from, to, amount]
		Transferred(CurrencyIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Update balance success. [currency_id, who, amount]
		BalanceUpdated(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Deposit success. [currency_id, who, amount]
		Deposited(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Withdraw success. [currency_id, who, amount]
		Withdrawn(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Serp Expand Supply successful. [currency_id, who, amount]
		SerpedUpSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// Serp Contract Supply successful. [currency_id, who, amount]
		SerpedDownSupply(CurrencyIdOf<T>, BalanceOf<T>),
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		/// Transfer some balance to another account under `currency_id`.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_non_native_currency())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as Stp258Currency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}

		/// Transfer some native currency to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_native_currency())]
		pub fn transfer_native_currency(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			T::Stp258Native::transfer(&from, &to, amount)?;

			Self::deposit_event(Event::Transferred(T::GetStp258NativeId::get(), from, to, amount));
			Ok(().into())
		}

		/// update amount of account `who` under `currency_id`.
		///
		/// The dispatch origin of this call must be _Root_.
		#[pallet::weight(T::WeightInfo::update_balance_non_native_currency())]
		pub fn update_balance(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			amount: AmountOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let dest = T::Lookup::lookup(who)?;
			<Self as Stp258CurrencyExtended<T::AccountId>>::update_balance(currency_id, &dest, amount)?;
			Ok(().into())
		}

		/// Called when `expand_supply` is received from the SERP.
		/// Implementation should `deposit` the `amount` to `serpup_to`, 
		/// then `amount` will be slashed from `serper` and update
		/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
		/// the `native_currency` used to expand settcurrency supply.
		#[pallet::weight(0)]
		pub  fn expand_supply(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>, 
			expand_by: BalanceOf<T>,
			quote_price: BalanceOf<T>, // the price of Dinar, so as to expand settcurrency supply.
		) -> DispatchResultWithPostInfo {
			// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
			// ↑ verify ↑
			let serper = &T::GetSerperAcc::get();
			let pay_by_quoted = Self::pay_serpup_by_quoted(currency_id, expand_by, quote_price);
			<Self as Stp258Currency<T::AccountId>>::deposit(currency_id, serper, expand_by);                                                                                                                                                                                                                                                                                                                                                                                                                                                                          
			<Self as Stp258AssetReservable<T::AccountId>>::slash_reserved(serper, pay_by_quoted);
			// both slash and deposit take care of total issuance, therefore nothing more to do.
			Self::deposit_event(Event::SerpedUpSupply(currency_id, expand_by));
			Ok(().into())
		}

		/// Called when `contract_supply` is received from the SERP.
		/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
		/// of `amount` to `serper`, then `amount` will be slashed from `serper` 
		/// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
		/// the `native_currency` used to contract settcurrency supply.
		#[pallet::weight(0)]
		pub fn contract_supply(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			contract_by: BalanceOf<T>,
			quote_price: BalanceOf<T>, // the price of Dinar, so as to contract settcurrency supply.
		) -> DispatchResultWithPostInfo {
			// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
			// ↑ verify ↑
			let serper = &T::GetSerperAcc::get();
			let pay_by_quoted = Self::pay_serpdown_by_quoted(currency_id, contract_by, quote_price);
			<Self as Stp258CurrencyReservable<T::AccountId>>::slash_reserved(currency_id, serper, contract_by);
			T::Stp258Native::deposit(serper, pay_by_quoted);
			// both slash and deposit take care of total issuance, therefore nothing more to do.
			Self::deposit_event(Event::SerpedDownSupply(currency_id, contract_by));
			Ok(().into())
		}

		/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpup, 
		/// the Serp Quote is `new_base_price - quotation` as the amount of native_currency to slash/buy-and-burn from serpers, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
		/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
		/// the fraction for `serp_quoted_price` is same as `(market_price - (mint_rate * 2))` - where `market-price = new_base_price / quote_price`, 
		/// `(mint_rate * 2) = serp_quote_multiple` as in price balance, `mint_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
		/// Therefore buying the native currency for more than market price.
		///
		/// The quoted amount to pay serpers for serping up supply.
		#[pallet::weight(0)]
		pub fn pay_serpup_by_quoted(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>, 
			expand_by: BalanceOf<T>, 
			quote_price: BalanceOf<T>, 
		) -> DispatchResultWithPostInfo {
			let supply = T::Stp258Currency::total_issuance(currency_id);
			let new_supply = supply.saturating_add(expand_by);
			let base_unit = T::GetBaseUnit;
			let serp_quote_multiple = T::GetSerpQuoteMultiple::get();
			let defloated = new_supply.saturating_mul_int(base_unit);
			let new_base_price = defloated.saturating_div_int(supply);
			let fractioned = new_base_price.saturating_sub(base_unit);
			let quotation = fractioned.saturating_mul_int(serp_quote_multiple);
			let serp_quoted_price = new_base_price.saturating_sub(quotation);;
			let relative_price = quote_price.saturating_div_int(serp_quoted_price as Self::Balance).to_num::<Self::Balance>();
			let pay_by_quoted = expand_by.saturating_div_int(relative_price);
			Ok(().into())
		}
	}
}

impl<T: Config> SerpMarket<T::AccountId> for Pallet<T>{
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;
	
	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpup, 
	/// the Serp Quote is `new_base_price - quotation` as the amount of native_currency to slash/buy-and-burn from serpers, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price - (mint_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(mint_rate * 2) = serp_quote_multiple` as in price balance, `mint_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the native currency for more than market price.
	///
	/// The quoted amount to pay serpers for serping up supply.
	fn pay_serpup_by_quoted(
		currency_id: CurrencyIdOf<T>, 
		expand_by: Self::Balance, 
		quote_price: Self::Balance, 
	) ->  DispatchResult {
        let supply = T::Stp258Currency::total_issuance(currency_id);
		let new_supply = supply.saturating_add(expand_by);
		let base_unit = T::GetBaseUnit;
        let serp_quote_multiple = T::GetSerpQuoteMultiple::get();
		let defloated = new_supply.saturating_mul_int(base_unit);
		let new_base_price = defloated.saturating_div_int(supply);
		let fractioned = new_base_price.saturating_sub(base_unit);
		let quotation = fractioned.saturating_mul_int(serp_quote_multiple);
		let serp_quoted_price = new_base_price.saturating_sub(quotation);;
		let relative_price = quote_price.saturating_div_int(serp_quoted_price as Self::Balance).to_num::<Self::Balance>();
		let pay_by_quoted = expand_by.saturating_div_int(relative_price);
	}

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpdown, 
	/// the Serp Quote is `quotation + new_base_price`, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price + (burn_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(burn_rate * 2) = serp_quote_multiple` as in price balance, `burn_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the stable currency for more than market price.
	///
	/// The quoted amount to pay serpers for serping down supply.
	fn pay_serpdown_by_quoted(
		currency_id: CurrencyIdOf<T>, 
		contract_by: Self::Balance, 
		quote_price: Self::Balance, 
	) ->  DispatchResult {
        let supply = T::Stp258Currency::total_issuance(currency_id);
		let new_supply = supply.saturating_sub(contract_by);
		let base_unit = T::GetBaseUnit;
        let serp_quote_multiple = T::GetSerpQuoteMultiple::get();
		let defloated = new_supply.saturating_mul_int(base_unit);
		let new_base_price = defloated.saturating_div_int(supply);
		let fractioned = base_unit.saturating_sub(new_base_price);
		let quotation = fractioned.saturating_mul_int(serp_quote_multiple);
		let serp_quoted_price = quotation.saturating_add(new_base_price);
		let relative_price = serp_quoted_price.saturating_div_int(quote_price as Self::Balance).to_num::<Self::Balance>();
		let defloated_by_quoted = relative_price.saturating_mul_int(contract_by);
		let pay_by_quoted = defloated_by_quoted.saturating_div_int(base_unit);
	}
}

impl<T: Config> Stp258Currency<T::AccountId> for Pallet<T> {
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::minimum_balance()
		} else {
			T::Stp258Currency::minimum_balance(currency_id)
		}
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::total_issuance()
		} else {
			T::Stp258Currency::total_issuance(currency_id)
		}
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::total_balance(who)
		} else {
			T::Stp258Currency::total_balance(currency_id, who)
		}
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::free_balance(who)
		} else {
			T::Stp258Currency::free_balance(currency_id, who)
		}
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::ensure_can_withdraw(who, amount)
		} else {
			T::Stp258Currency::ensure_can_withdraw(currency_id, who, amount)
		}
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::transfer(from, to, amount)?;
		} else {
			T::Stp258Currency::transfer(currency_id, from, to, amount)?;
		}
		Self::deposit_event(Event::Transferred(currency_id, from.clone(), to.clone(), amount));
		Ok(())
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::deposit(who, amount)?;
		} else {
			T::Stp258Currency::deposit(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
		Ok(())
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::withdraw(who, amount)?;
		} else {
			T::Stp258Currency::withdraw(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Withdrawn(currency_id, who.clone(), amount));
		Ok(())
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> bool {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::can_slash(who, amount)
		} else {
			T::Stp258Currency::can_slash(currency_id, who, amount)
		}
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::slash(who, amount)
		} else {
			T::Stp258Currency::slash(currency_id, who, amount)
		}
	}
}

impl<T: Config> Stp258CurrencyExtended<T::AccountId> for Pallet<T> {
	type Amount = AmountOf<T>;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::update_balance(who, by_amount)?;
		} else {
			T::Stp258Currency::update_balance(currency_id, who, by_amount)?;
		}
		Self::deposit_event(Event::BalanceUpdated(currency_id, who.clone(), by_amount));
		Ok(())
	}
}

impl<T: Config> Stp258CurrencyLockable<T::AccountId> for Pallet<T> {
	type Moment = T::BlockNumber;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::set_lock(lock_id, who, amount)
		} else {
			T::Stp258Currency::set_lock(lock_id, currency_id, who, amount)
		}
	}

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::extend_lock(lock_id, who, amount)
		} else {
			T::Stp258Currency::extend_lock(lock_id, currency_id, who, amount)
		}
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::remove_lock(lock_id, who)
		} else {
			T::Stp258Currency::remove_lock(lock_id, currency_id, who)
		}
	}
}

impl<T: Config> Stp258CurrencyReservable<T::AccountId> for Pallet<T> {
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::can_reserve(who, value)
		} else {
			T::Stp258Currency::can_reserve(currency_id, who, value)
		}
	}

	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::slash_reserved(who, value)
		} else {
			T::Stp258Currency::slash_reserved(currency_id, who, value)
		}
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::reserved_balance(who)
		} else {
			T::Stp258Currency::reserved_balance(currency_id, who)
		}
	}

	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::reserve(who, value)
		} else {
			T::Stp258Currency::reserve(currency_id, who, value)
		}
	}

	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::unreserve(who, value)
		} else {
			T::Stp258Currency::unreserve(currency_id, who, value)
		}
	}

	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::repatriate_reserved(slashed, beneficiary, value, status)
		} else {
			T::Stp258Currency::repatriate_reserved(currency_id, slashed, beneficiary, value, status)
		}
	}
}

pub struct Currency<T, GetCurrencyId>(marker::PhantomData<T>, marker::PhantomData<GetCurrencyId>);

impl<T, GetCurrencyId> Stp258Asset<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Balance = BalanceOf<T>;

	fn minimum_balance() -> Self::Balance {
		<Pallet<T>>::minimum_balance(GetCurrencyId::get())
	}

	fn total_issuance() -> Self::Balance {
		<Pallet<T>>::total_issuance(GetCurrencyId::get())
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::total_balance(GetCurrencyId::get(), who)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as Stp258Currency<T::AccountId>>::transfer(GetCurrencyId::get(), from, to, amount)
	}

	fn deposit(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::deposit(GetCurrencyId::get(), who, amount)
	}

	fn withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::withdraw(GetCurrencyId::get(), who, amount)
	}

	fn can_slash(who: &T::AccountId, amount: Self::Balance) -> bool {
		<Pallet<T>>::can_slash(GetCurrencyId::get(), who, amount)
	}

	fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		<Pallet<T>>::slash(GetCurrencyId::get(), who, amount)
	}
}

impl<T, GetCurrencyId> Stp258AssetExtended<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Amount = AmountOf<T>;

	fn update_balance(who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		<Pallet<T> as Stp258CurrencyExtended<T::AccountId>>::update_balance(GetCurrencyId::get(), who, by_amount)
	}
}

impl<T, GetCurrencyId> Stp258AssetLockable<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Moment = T::BlockNumber;

	fn set_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as Stp258CurrencyLockable<T::AccountId>>::set_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn extend_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as Stp258CurrencyLockable<T::AccountId>>::extend_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn remove_lock(lock_id: LockIdentifier, who: &T::AccountId) -> DispatchResult {
		<Pallet<T> as Stp258CurrencyLockable<T::AccountId>>::remove_lock(lock_id, GetCurrencyId::get(), who)
	}
}

impl<T, GetCurrencyId> Stp258AssetReservable<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::slash_reserved(GetCurrencyId::get(), who, value)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		<Pallet<T> as Stp258CurrencyReservable<T::AccountId>>::repatriate_reserved(
			GetCurrencyId::get(),
			slashed,
			beneficiary,
			value,
			status,
		)
	}
}

pub type Stp258NativeOf<T> = Currency<T, <T as Config>::GetStp258NativeId>;

/// Adapt other currency traits implementation to `Stp258Asset`.
pub struct Stp258AssetAdapter<T, Currency, Amount, Moment>(marker::PhantomData<(T, Currency, Amount, Moment)>);

type PalletBalanceOf<A, Currency> = <Currency as SetheumCurrency<A>>::Balance;

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> Stp258Asset<AccountId>
	for Stp258AssetAdapter<T, Currency, Amount, Moment>
where
	Currency: SetheumCurrency<AccountId>,
	T: Config,
{
	type Balance = PalletBalanceOf<AccountId, Currency>;

	fn minimum_balance() -> Self::Balance {
		Currency::minimum_balance()
	}

	fn total_issuance() -> Self::Balance {
		Currency::total_issuance()
	}

	fn total_balance(who: &AccountId) -> Self::Balance {
		Currency::total_balance(who)
	}

	fn free_balance(who: &AccountId) -> Self::Balance {
		Currency::free_balance(who)
	}

	fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let new_balance = Self::free_balance(who)
			.checked_sub(&amount)
			.ok_or(Error::<T>::BalanceTooLow)?;

		Currency::ensure_can_withdraw(who, amount, WithdrawReasons::all(), new_balance)
	}

	fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)
	}

	fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let _ = Currency::deposit_creating(who, amount);
		Ok(())
	}

	fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::withdraw(who, amount, WithdrawReasons::all(), ExistenceRequirement::AllowDeath).map(|_| ())
	}

	fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
		Currency::can_slash(who, amount)
	}

	fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash(who, amount);
		gap
	}
}

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> Stp258AssetExtended<AccountId>
	for Stp258AssetAdapter<T, Currency, Amount, Moment>
where
	Amount: Signed
		+ TryInto<PalletBalanceOf<AccountId, Currency>>
		+ TryFrom<PalletBalanceOf<AccountId, Currency>>
		+ SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default,
	Currency: SetheumCurrency<AccountId>,
	T: Config,
{
	type Amount = Amount;

	fn update_balance(who: &AccountId, by_amount: Self::Amount) -> DispatchResult {
		let by_balance = by_amount
			.abs()
			.try_into()
			.map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(who, by_balance)
		} else {
			Self::withdraw(who, by_balance)
		}
	}
}

// Adapt `frame_support::traits::LockableCurrency`
impl<T, AccountId, Currency, Amount, Moment> Stp258AssetLockable<AccountId>
	for Stp258AssetAdapter<T, Currency, Amount, Moment>
where
	Currency: SetheumLockableCurrency<AccountId>,
	T: Config,
{
	type Moment = Moment;

	fn set_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::set_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn extend_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::extend_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn remove_lock(lock_id: LockIdentifier, who: &AccountId) -> DispatchResult {
		Currency::remove_lock(lock_id, who);
		Ok(())
	}
}

// Adapt `frame_support::traits::ReservableCurrency`
impl<T, AccountId, Currency, Amount, Moment> Stp258AssetReservable<AccountId>
	for Stp258AssetAdapter<T, Currency, Amount, Moment>
where
	Currency: SetheumReservableCurrency<AccountId>,
	T: Config,
{
	fn can_reserve(who: &AccountId, value: Self::Balance) -> bool {
		Currency::can_reserve(who, value)
	}

	fn slash_reserved(who: &AccountId, value: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash_reserved(who, value);
		gap
	}

	fn reserved_balance(who: &AccountId) -> Self::Balance {
		Currency::reserved_balance(who)
	}

	fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult {
		Currency::reserve(who, value)
	}

	fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance {
		Currency::unreserve(who, value)
	}

	fn repatriate_reserved(
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		Currency::repatriate_reserved(slashed, beneficiary, value, status)
	}
}

impl<T: Config> MergeAccount<T::AccountId> for Pallet<T> {
	fn merge_account(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
		with_transaction_result(|| {
			// transfer non-native free to dest
			T::Stp258Currency::merge_account(source, dest)?;

			// unreserve all reserved currency
			T::Stp258Native::unreserve(source, T::Stp258Native::reserved_balance(source));

			// transfer all free to dest
			T::Stp258Native::transfer(source, dest, T::Stp258Native::free_balance(source))
		})
	}
}
