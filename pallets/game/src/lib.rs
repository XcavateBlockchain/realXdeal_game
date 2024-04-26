#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::{
	traits::{Incrementable, Currency},
	PalletId,
};

use frame_support::sp_runtime::traits::AccountIdConversion;

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use enumflags2::BitFlags;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Difficulty level of game enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum DifficultyLevel {
		Practice,
		Player,
		Pro,
	}

	/// AccountId storage.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Origin who can create a new game.
		type GameOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// Item id type from pallet nfts.
		type ItemId: IsType<<Self as pallet_nfts::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// The maximum amount of properties.
		#[pallet::constant]
		type MaxProperty: Get<u32>;
		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	/// Property Data.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PropertyData<ItemId, CollectionId, T: Config> {
		pub collection_id: CollectionId,
		pub item_id: ItemId,
		pub data: BoundedVec<u8, T::MaxProperty>,
	}

	#[pallet::storage]
	#[pallet::getter(fn next_color_id)]
	pub(super) type NextColorId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn player_points)]
	pub type PlayerPoints<T> = StorageMap<
		_, 
		Blake2_128Concat, 
		AccountIdOf<T>, 
		u32, 
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn properties)]
	pub type Properties<T: Config> = StorageValue<
		_,
		BoundedVec<PropertyData<<T as pallet::Config>::ItemId, <T as pallet::Config>::CollectionId, T>, T::MaxProperty>, 
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// A player has not enough points to play.
		NotEnoughPoints,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);

			weight
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn play_game(origin: OriginFor<T>, game_type: DifficultyLevel) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::check_enough_points(signer, game_type)?;

			/// Call crust or oracle to chose a property for a game
			todo!();

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn setup_game(origin: OriginFor<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			for x in 0..8 {
				if pallet_nfts::NextCollectionId::<T>::get().is_none() {
					pallet_nfts::NextCollectionId::<T>::set(
						<T as pallet_nfts::Config>::CollectionId::initial_value(),
					);
				};
				let collection_id = pallet_nfts::NextCollectionId::<T>::get().unwrap();
				let next_collection_id = collection_id.increment();
				pallet_nfts::NextCollectionId::<T>::set(next_collection_id);
				let collection_id: CollectionId<T> = collection_id.into();
				let pallet_id: AccountIdOf<T> =
					<T as pallet::Config>::PalletId::get().into_account_truncating();
				pallet_nfts::Pallet::<T>::do_create_collection(
					collection_id.into(),
					pallet_id.clone(),
					pallet_id.clone(),
					Self::default_collection_config(),
					T::CollectionDeposit::get(),
					pallet_nfts::Event::Created {
						creator: pallet_id.clone(),
						owner: pallet_id,
						collection: collection_id.into(),
					},
				)?;
			}
			Ok(())
		}

	}


	impl<T: Config> Pallet<T> {

		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}
	
		fn check_enough_points(
			signer: AccountIdOf<T>,
			game_type: DifficultyLevel
		) -> DispatchResult {
			if game_type == DifficultyLevel::Pro {
				ensure!(Self::player_points(signer) >= Some(50), Error::<T>::NotEnoughPoints);
			} else if game_type == DifficultyLevel::Player {
				ensure!(Self::player_points(signer) >= Some(25), Error::<T>::NotEnoughPoints);
			} 
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
		fn default_collection_config() -> CollectionConfig<
			BalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			Self::collection_config_from_disabled_settings(
				CollectionSetting::DepositRequired.into(),
			)
		}

		fn collection_config_from_disabled_settings(
			settings: BitFlags<CollectionSetting>,
		) -> CollectionConfig<
			BalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			CollectionConfig {
				settings: CollectionSettings::from_disabled(settings),
				max_supply: None,
				mint_settings: MintSettings::default(),
			}
		}
	}
}

