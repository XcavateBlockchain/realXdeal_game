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

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, StaticLookup}, Saturating,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use enumflags2::BitFlags;

use frame_support::traits::Randomness;

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

	/// Offer enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Offer {
		Accept,
		Reject,
	}

	/// AccountId storage.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// Property Data.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PropertyData<CollectionId, T: Config> {
		pub collection_id: CollectionId,
		pub data: BoundedVec<u8, T::MaxProperty>,
	}

	/// Game Data.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct GameData<CollectionId, T: Config> {
		pub difficulty: DifficultyLevel,
		pub player: AccountIdOf<T>,
		pub property: PropertyData<CollectionId, T>,
	}

	/// Listing infos of a NFT.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct ListingInfo<CollectionId, ItemId, T: Config> {
		pub owner: AccountIdOf<T>,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	/// Offer infos of a listing.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct OfferInfo<CollectionId, ItemId, T: Config> {
		pub owner: AccountIdOf<T>,
		pub listing_id: u32,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config 
		+ pallet_nfts::Config 
		+ pallet_babe::Config 
	{
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
		type MaxProperty: Get<u32>
			+ Clone
			+ PartialEq
			+ Eq;
		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The maximum amount of games that can be played at the same time.
		#[pallet::constant]
		type MaxOngoingGames: Get<u32>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	#[pallet::storage]
	#[pallet::getter(fn stored_hash)]
	pub type StoredHash<T: Config> =
		StorageValue<_, Option<T::Hash>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn stored_number)]
	pub type StoredNumber<T: Config> =
		StorageValue<_, u32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_color_id)]
	pub(super) type NextColorId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_lising_id)]
	pub(super) type NextListingId<T> = 
		StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_offer_id)]
	pub(super) type NextOfferId<T> = 
		StorageValue<_, u32, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn game_id)]
	pub type GameId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn player_points)]
	pub type PlayerPoints<T> = StorageMap<
		_, 
		Blake2_128Concat, 
		AccountIdOf<T>, 
		u32, 
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn game_info)]
	pub type GameInfo<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		GameData<<T as pallet::Config>::CollectionId, T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn listings)]
	pub type Listings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		ListingInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		OfferInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	/// Stores the project keys and round types ending on a given block.
	#[pallet::storage]
	pub type GamesExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<u32, T::MaxOngoingGames>,
		ValueQuery,
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
		ConversionError,
		ArithmeticOverflow,
		ArithmeticUnderflow,
		MultiplyError,
		DivisionError,
		/// There are too many games active.
		TooManyGames,
		/// This is not an active game.
		NoAcitveGame,
		/// The caller has no permission.
		NoThePlayer,
		/// This game is not active.
		NoActiveGame,
		NoPermission,
		/// This listing is not listed.
		ListingDoesNotExist,
		/// This offer does not exist.
		OfferDoesNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);
			let ended_games = GamesExpiring::<T>::take(n);

			// Checks if there is a voting for a loan expiring in this block.
			ended_games.iter().for_each(|index| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let game_info = <GameInfo<T>>::take(index);
				if let Some(game_info) = game_info {
					let _ = Self::NoAnswerResult(game_info);
				}
				///check for result.
				todo!();
			});
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
		pub fn play_game(origin: OriginFor<T>, game_type: DifficultyLevel, rand: u8) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::check_enough_points(signer, game_type.clone())?;
			let mut game_id = Self::game_id();
			let (hashi, _) = pallet_babe::ParentBlockRandomness::<T>::random(&[rand]);
            StoredHash::<T>::put(hashi);
			let u32_value = u32::from_le_bytes(
                hashi.unwrap().as_ref()[4..8]
                    .try_into()
                    .map_err(|_| Error::<T>::ConversionError)?
            );
			StoredNumber::<T>::put(u32_value);
			if game_type == DifficultyLevel::Player {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block =
					current_block_number.saturating_add(8u32.into());
	
				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else if game_type == DifficultyLevel::Pro {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block =
				current_block_number.saturating_add(5u32.into());
	
				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block =
				current_block_number.saturating_add(10u32.into());
	
				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			}
			/// Call crust or oracle to chose a property for a game
			todo!();
			let random_number = u32_value % 8;
			let property = PropertyData {
				collection_id: random_number.into(),
				data: Default::default(),
			};
			let game_datas = GameData {
				difficulty: game_type,
				player: signer,
				property,
			};
			GameInfo::<T>::insert(game_id, game_datas);
			game_id = game_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			GameId::<T>::put(game_id);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn submit_answer(origin: OriginFor<T>, guess: u32, game_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let game_info = GameInfo::<T>::take(game_id).ok_or(Error::<T>::NoActiveGame)?;
			ensure!(signer == game_info.player, Error::<T>::NoThePlayer);
			let result: u32 = 100_000;
			let difference_value = ((result as i32).checked_sub(guess as i32).ok_or(Error::<T>::ArithmeticUnderflow)?)
				.checked_mul(1000).ok_or(Error::<T>::MultiplyError)?
				.checked_div(result as i32).ok_or(Error::<T>::DivisionError)?;
			Self::check_result(difference_value.try_into().unwrap(), game_id)?;
			Ok(())
		}

		
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn list_nft(origin: OriginFor<T>, collection_id: CollectionId<T>, item_id: ItemId<T>) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			pallet_nfts::Pallet::<T>::transfer(origin, collection_id.into(), item_id.into(), pallet_lookup)?;
			let listing_info = ListingInfo {
				owner: signer,
				collection_id,
				item_id,
			};
			let mut listing_id = Self::next_lising_id();
			Listings::<T>::insert(listing_id, listing_info);
			listing_id = listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextListingId::<T>::put(listing_id);
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn delist_nft(origin: OriginFor<T>, listing_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let listing_info = Listings::<T>::take(listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_info.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn make_offer(origin: OriginFor<T>, listing_id: u32, collection_id: CollectionId<T>, item_id: ItemId<T>) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(Self::listings(listing_id).is_some(), Error::<T>::ListingDoesNotExist);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			pallet_nfts::Pallet::<T>::transfer(origin, collection_id.into(), item_id.into(), pallet_lookup)?;
			let offer_info = OfferInfo {
				owner: signer,
				listing_id,
				collection_id,
				item_id,
			};
			let mut offer_id = Self::next_offer_id();
			Offers::<T>::insert(offer_id, offer_info);
			let offer_id = offer_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextOfferId::<T>::put(offer_id);
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn handle_offer(origin: OriginFor<T>, offer_id: u32, offer: Offer) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			let listing_details = Self::listings(offer_details.listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_details.owner == signer, Error::<T>::NoPermission);
			if offer == Offer::Accept {
				pallet_nfts::Pallet::<T>::do_transfer(
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
					offer_details.owner,
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					listing_details.owner,
					|_, _| Ok(()),
				)?;
				Listings::<T>::take(offer_details.listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			} else {
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					offer_details.owner,
					|_, _| Ok(()),
				)?;
			}
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn setup_game(origin: OriginFor<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			for _x in 0..8 {
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
				ensure!(Self::player_points(signer) >= 50, Error::<T>::NotEnoughPoints);
			} else if game_type == DifficultyLevel::Player {
				ensure!(Self::player_points(signer) >= 25, Error::<T>::NotEnoughPoints);
			} 
			Ok(())
		}

 		fn check_result(
			difference: u16,
			game_id: u32,
		) -> DispatchResult {
			let game_info = GameInfo::<T>::take(game_id).ok_or(Error::<T>::NoAcitveGame)?;
			if game_info.difficulty == DifficultyLevel::Pro {
				match difference {
					0..=10 => {
						let mut next_item_id = Self::next_color_id(game_info.property.collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id = next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(game_info.property.collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							game_info.property.collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player.clone(),
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
					}
					11..=30 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(50).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					} 
					31..=50 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(30).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					} 
					51..=100 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(10).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					101..=150 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					151..=200 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					201..=250 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(30).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					251..=300 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(40).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					_ => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
				}
			}
			else if game_info.difficulty == DifficultyLevel::Player {
				match difference {
					0..=10 => {
						let mut next_item_id = Self::next_color_id(game_info.property.collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id = next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(game_info.property.collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							game_info.property.collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player,
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
					}
					11..=30 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(25).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					} 
					31..=50 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(15).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					} 
					51..=100 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_add(5).ok_or(Error::<T>::ArithmeticOverflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					101..=150 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					151..=200 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					201..=250 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					251..=300 => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player.clone(), points);
					}
					_ => {
						let mut points = Self::player_points(game_info.player.clone());
						points = points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
						PlayerPoints::<T>::insert(game_info.player, points);
					}
				}
			}
			Ok(())
		} 

		fn NoAnswerResult(game_info: GameData<<T as pallet::Config>::CollectionId, T>) -> DispatchResult {
			if game_info.difficulty == DifficultyLevel::Pro {
				let mut points = Self::player_points(game_info.player.clone());
				points = points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
				PlayerPoints::<T>::insert(game_info.player, points);
			} else if game_info.difficulty == DifficultyLevel::Player {
				let mut points = Self::player_points(game_info.player.clone());
				points = points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
				PlayerPoints::<T>::insert(game_info.player, points);
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

		/// Set the default item configuration for minting a nft.
		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}
	}
}

