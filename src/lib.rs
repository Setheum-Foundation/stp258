#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Codec;
use frame_support::{
	pallet_prelude::*,
	debug::native,
	traits::{
		Currency as SetheumCurrency, ExistenceRequirement, Get, 
		ReservableCurrency as SetheumReservableCurrency, WithdrawReasons,
	},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use stp258_traits::{
	account::MergeAccount,
	arithmetic::{Signed, SimpleArithmetic},
	BalanceStatus, SerpMarket, 
	Stp258Asset, Stp258AssetExtended, Stp258AssetReservable,
	Stp258Currency, Stp258CurrencyExtended, Stp258CurrencyReservable,
};
use orml_utilities::with_transaction_result;
use sp_runtime::{
	traits::{CheckedSub, MaybeSerializeDeserialize, StaticLookup, Zero},
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
	pub(crate) type AccountIdOf<T> =
		<<T as Config>::Stp258Currency as Stp258Currency<<T as frame_system::Config>::AccountId>>::AccountId;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Stp258Currency: MergeAccount<Self::AccountId>
			+ Stp258CurrencyExtended<Self::AccountId>
			+ Stp258CurrencyReservable<Self::AccountId>;

		type Stp258Native: Stp258AssetExtended<Self::AccountId, Balance = BalanceOf<Self>, Amount = AmountOf<Self>>
			+ Stp258AssetReservable<Self::AccountId, Balance = BalanceOf<Self>>;

		/// The SettPay Account type
		#[pallet::constant]
		type GetSettPayAcc: Get<AccountIdOf<Self>>;

		/// The Serpers Account type
		#[pallet::constant]
		type GetSerperAcc: Get<AccountIdOf<Self>>;

		/// The Serp quote multiple type for qUOTE, quoting 
		/// `(mintrate * SERP_QUOTE_MULTIPLE) = SerpQuotedPrice`.
		#[pallet::constant]
		type GetSerpQuoteMultiple: Get<BalanceOf<Self>>;

		/// The Serper ratio type getter
		#[pallet::constant]
		type GetSerperRatio: Get<BalanceOf<Self>>;

		/// The SettPay ratio type getter
		#[pallet::constant]
		type GetSettPayRatio: Get<BalanceOf<Self>>;

		/// The native asset (Dinar) Currency ID type
		#[pallet::constant]
		type GetStp258NativeId: Get<CurrencyIdOf<Self>>;

		/// The balance of an account.
		#[pallet::constant]
		type GetBaseUnit: Get<BalanceOf<Self>>;
		

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
		/// While trying to expand the supply, it overflowed.
		SupplyOverflow,
		/// While trying to contract the supply, it underflowed.
		SupplyUnderflow,
		/// Unable to convert the Amount type into Balance.
		AmountIntoBalanceFailed,
		/// Balance is too low.
		BalanceTooLow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency transfer success. [currency_id, from, to, amount]
		Transferred(CurrencyIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Update balance success. [currency_id, who, amount]
		/// Deposit success. [currency_id, who, amount]
		Deposited(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Withdraw success. [currency_id, who, amount]
		Withdrawn(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Serp Expand Supply successful. [currency_id, who, amount]
		SerpedUpSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// Serp Contract Supply successful. [currency_id, who, amount]
		SerpedDownSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// The New Price of Currency. [currency_id, price]
		NewPrice(CurrencyIdOf<T>, BalanceOf<T>),
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
	}
}

impl<T: Config> SerpMarket<CurrencyIdOf<T>, T::AccountId,  BalanceOf<T>> for Pallet<T> {
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;
	type AccountId = AccountIdOf<T>;

	/// A trait to provide relative `base_price` of `base_settcurrency_id`. 
	/// The settcurrency `Price` is `base_price * base_unit`.
	/// For example The `Price` of `JUSD` is `base_price: Price = $1.1 * base_unit: BaseUnit = 1_100`.
	/// Therefore, the `Price` is got by checking how much `base_currency_peg` can buy `base_unit`, 
	/// in our example, `1_100` in `base_currency_peg: USD` of `JUSD` can buy `base_unit` of `JUSD` in `USD`.
	fn get_stable_price(
		base_settcurrency_id: CurrencyIdOf<T>,
		base_price: BalanceOf<T>,
	) -> DispatchResult {
		let base_unit = T::GetBaseUnit::get();
		let amount_of_peg_to_buy_base_currency = base_price * base_unit;
		Self::deposit_event(Event::NewPrice(base_settcurrency_id, amount_of_peg_to_buy_base_currency));
		Ok(())
	}
	
	/// A trait to provide relative price for two currencies. 
	/// For example, the relative price of `DNAR-JUSD` is `$1_000 / $1.1 = JUSD 1_100`,
	/// meaning the price compared in `USD` as the peg of `JUSD` for example.. or,
	/// the relative price of `DNAR-JUSD` is `DNAR 1 / JUSD 0.001 = JUSD 1_000`,
	/// meaning `DNAR 1` can buy `JUSD 1_000` and therefore `1 DNAR = 0.001 JUSD`.
	/// But tyhe former is preffered and thus used.
	fn get_relative_price(
		base_currency_id: CurrencyIdOf<T>, 
		base_price: BalanceOf<T>, 
		quote_currency_id: CurrencyIdOf<T>, 
		quote_price: BalanceOf<T>,
	) -> DispatchResult {
		let amount_of_quote_to_buy_base = base_price / quote_price;
		let amount_of_base_to_buy_quote = quote_price / base_price;
		Ok(())
	}

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers, 
	/// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// `quotation + fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
	///  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	///
	/// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn quote_serp_price(
		currency_id: CurrencyIdOf<T>, 
		price: BalanceOf<T>,
	) -> Self::Balance{
		let base_unit = T::GetBaseUnit::get();
		let serp_quote_multiple = T::GetSerpQuoteMultiple::get();
		let fraction = price / base_unit;
		let fractioned = fraction.saturating_sub(1);
		let quotation = fractioned.saturating_mul_int(serp_quote_multiple);
		let serp_quoted_price =  fraction + quotation;

		Self::deposit_event(Event::NewPrice(currency_id, serp_quoted_price));
		Ok(())
	}


	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to expand settcurrency supply.
	fn expand_supply(
		currency_id: CurrencyIdOf<T>, 
		expand_by: BalanceOf<T>,
		quote_price: BalanceOf<T>, // the price of Dinar, so as to expand settcurrency supply.
	) -> DispatchResult{
		let supply = T::Stp258Currency::total_issuance(currency_id);
		// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
		// ↑ verify ↑
		let native_asset_id = T::GetStp258NativeId::get();
		let serper = &T::GetSerperAcc::get(); 
		let settpay = &T::GetSettPayAcc::get();
		let base_currency_id = currency_id;
		let quote_currency_id = native_asset_id;
		let new_supply = supply + expand_by; 
		let base_price = new_supply / supply;
		let price = Self::get_relative_price(
			native_asset_id,
			base_price,
			currency_id, 
			quote_price,
		);

		let supply_change = expand_by;
		let serp_quoted_price = Self::quote_serp_price(base_currency_id, price);
		let settpay_ratio = &T::GetSettPayRatio::get(); // 75% for SettPay. It was statically typed, now moved to runtime and can be set there.
		let serper_ratio = &T::GetSerperRatio::get(); // 25% for Serpers. It was statically typed, now moved to runtime and can be set there.
		let settpay_distro = expand_by.saturating_mul_int(settpay_ratio); // 75% distro for SettPay.
		let serper_distro = expand_by.saturating_mul_int(serper_ratio); // 25% distro for Serpers.
		let pay_by_quoted = serper_distro.saturating_div_int(serp_quoted_price);
		if currency_id == T::GetStp258NativeId::get() {
			debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
			return Err(http::Error::Unknown);
		} else {
			T::Stp258Currency::deposit(currency_id, settpay, settpay_distro);
			T::Stp258Currency::reserve(currency_id, serper, serper_distro);
			T::Stp258Native::slash_reserved(serper, pay_by_quoted);
		}
		// both slash and deposit take care of total issuance, therefore nothing more to do.
		Self::deposit_event(Event::SerpedUpSupply(currency_id, expand_by));
		Self::deposit_event(Event::NewPrice(currency_id, serp_quoted_price));
		Ok(())
	}

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to contract settcurrency supply.
	fn contract_supply(
		currency_id: Self::CurrencyId,
		contract_by: BalanceOf<T>,
		quote_price: BalanceOf<T>, // the price of Dinar, so as to contract settcurrency supply.
	) -> DispatchResult{
		let supply = T::Stp258Currency::total_issuance(currency_id);
		// Both slash and deposit will check whether the supply will overflow. Therefore no need to check twice.
		// ↑ verify ↑
		let serper = &T::GetSerperAcc::get();
		let settpay = &T::GetSettPayAcc::get();
		let native_asset_id = T::GetStp258NativeId::get();
		let base_currency_id = currency_id;
		let quote_currency_id = native_asset_id;
		let new_supply = supply + contract_by; 
		let base_price = new_supply / supply;
		let price = Self::get_relative_price(
			currency_id, 
			quote_price,
			native_asset_id,
			base_price, 
		);
		let supply_change = contract_by;
		let serp_quoted_price = Self::quote_serp_price(base_currency_id, price);
		let new_price = serp_quoted_price;
		let pay_by_quoted = serp_quoted_price * supply_change;
		if currency_id == T::GetStp258NativeId::get() {
			debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
			return Err(http::Error::Unknown);
		} else {
			T::Stp258Currency::slash_reserved(currency_id, serper, contract_by);
			T::Stp258Native::deposit(serper, pay_by_quoted);
			T::Stp258Native::reserve(serper, pay_by_quoted);
		}
		// both slash and deposit take care of total issuance, therefore nothing more to do.
		Self::deposit_event(Event::SerpedDownSupply(currency_id, contract_by));
		Self::deposit_event(Event::NewPrice(currency_id, serp_quoted_price));
		Ok(())
	}
}
