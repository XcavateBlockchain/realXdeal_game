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

pub mod properties;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::{
	traits::{Currency, Incrementable},
	PalletId,
};

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, StaticLookup},
	Saturating,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

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

	/// Nft color enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum NftColor {
		Xorange,
		Xpink,
		Xblue,
		Xcyan,
		Xcoral,
		Xpurple,
		Xleafgreen,
		Xgreen,
	}

	impl NftColor {
		pub fn from_index(index: usize) -> Option<Self> {
			match index {
				0 => Some(NftColor::Xorange),
				1 => Some(NftColor::Xpink),
				2 => Some(NftColor::Xblue),
				3 => Some(NftColor::Xcyan),
				4 => Some(NftColor::Xcoral),
				5 => Some(NftColor::Xpurple),
				6 => Some(NftColor::Xleafgreen),
				7 => Some(NftColor::Xgreen),
				_ => None,
			}
		}
	}

	/// AccountId storage.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// Game Data.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct GameData<T: Config> {
		pub difficulty: DifficultyLevel,
		pub player: AccountIdOf<T>,
		pub property: PropertyInfoData<T>,
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

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PropertyInfoData<T: Config> {
		pub id: u32,
		pub property_type: BoundedVec<u8, <T as Config>::StringLimit>,
		pub bedrooms: u32,
		pub bathrooms: u32,
		pub city: BoundedVec<u8, <T as Config>::StringLimit>,
		pub post_code: BoundedVec<u8, <T as Config>::StringLimit>,
		pub key_features: BoundedVec<u8, <T as Config>::StringLimit>,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct User {
		pub points: u32,
		pub wins: u32,
		pub losses: u32,
		pub practise_rounds: u8,
		pub nfts: CollectedColors,
	}

	impl User {
		pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
			self.nfts.add_nft_color(color)?;
			Ok(())
		}

		pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
			self.nfts.sub_nft_color(color)?;
			Ok(())
		}

		pub fn has_four_of_all_colors(&self) -> bool {
			self.nfts.has_four_of_all_colors()
		}

		pub fn calculate_points(&mut self, color: NftColor) -> u32 {
			match color {
				NftColor::Xorange if self.nfts.xorange == 1 => 100,
				NftColor::Xorange if self.nfts.xorange == 2 => 120,
				NftColor::Xorange if self.nfts.xorange == 3 => 220,
				NftColor::Xorange if self.nfts.xorange == 4 => 340,
				NftColor::Xpink if self.nfts.xpink == 1 => 100,
				NftColor::Xpink if self.nfts.xpink == 2 => 120,
				NftColor::Xpink if self.nfts.xpink == 3 => 220,
				NftColor::Xpink if self.nfts.xpink == 4 => 340,
				NftColor::Xblue if self.nfts.xblue == 1 => 100,
				NftColor::Xblue if self.nfts.xblue == 2 => 120,
				NftColor::Xblue if self.nfts.xblue == 3 => 220,
				NftColor::Xblue if self.nfts.xblue == 4 => 340,
				NftColor::Xcyan if self.nfts.xcyan == 1 => 100,
				NftColor::Xcyan if self.nfts.xcyan == 2 => 120,
				NftColor::Xcyan if self.nfts.xcyan == 3 => 220,
				NftColor::Xcyan if self.nfts.xcyan == 4 => 340,
				NftColor::Xcoral if self.nfts.xcoral == 1 => 100,
				NftColor::Xcoral if self.nfts.xcoral == 2 => 120,
				NftColor::Xcoral if self.nfts.xcoral == 3 => 220,
				NftColor::Xcoral if self.nfts.xcoral == 4 => 340,
				NftColor::Xpurple if self.nfts.xpurple == 1 => 100,
				NftColor::Xpurple if self.nfts.xpurple == 2 => 120,
				NftColor::Xpurple if self.nfts.xpurple == 3 => 220,
				NftColor::Xpurple if self.nfts.xpurple == 4 => 340,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 100,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 120,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 220,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 4 => 340,
				NftColor::Xgreen if self.nfts.xgreen == 1 => 100,
				NftColor::Xgreen if self.nfts.xgreen == 2 => 120,
				NftColor::Xgreen if self.nfts.xgreen == 3 => 220,
				NftColor::Xgreen if self.nfts.xgreen == 4 => 340,
				_ => 0,
			}
		}

		pub fn subtracting_calculate_points(&mut self, color: NftColor) -> u32 {
			match color {
				NftColor::Xorange if self.nfts.xorange == 1 => 120,
				NftColor::Xorange if self.nfts.xorange == 2 => 100,
				NftColor::Xorange if self.nfts.xorange == 3 => 120,
				NftColor::Xpink if self.nfts.xpink == 1 => 120,
				NftColor::Xpink if self.nfts.xpink == 2 => 100,
				NftColor::Xpink if self.nfts.xpink == 3 => 120,
				NftColor::Xblue if self.nfts.xblue == 1 => 120,
				NftColor::Xblue if self.nfts.xblue == 2 => 100,
				NftColor::Xblue if self.nfts.xblue == 3 => 120,
				NftColor::Xcyan if self.nfts.xcyan == 1 => 120,
				NftColor::Xcyan if self.nfts.xcyan == 2 => 100,
				NftColor::Xcyan if self.nfts.xcyan == 3 => 120,
				NftColor::Xcoral if self.nfts.xcoral == 1 => 120,
				NftColor::Xcoral if self.nfts.xcoral == 2 => 100,
				NftColor::Xcoral if self.nfts.xcoral == 3 => 120,
				NftColor::Xpurple if self.nfts.xpurple == 1 => 120,
				NftColor::Xpurple if self.nfts.xpurple == 2 => 100,
				NftColor::Xpurple if self.nfts.xpurple == 3 => 120,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 120,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 100,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 120,
				NftColor::Xgreen if self.nfts.xgreen == 1 => 120,
				NftColor::Xgreen if self.nfts.xgreen == 2 => 100,
				NftColor::Xgreen if self.nfts.xgreen == 3 => 120,
				_ => 0,
			}
		}
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo, Default,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct CollectedColors {
		pub xorange: u32,
		pub xpink: u32,
		pub xblue: u32,
		pub xcyan: u32,
		pub xcoral: u32,
		pub xpurple: u32,
		pub xleafgreen: u32,
		pub xgreen: u32,
	}

	impl CollectedColors {
		pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
			match color {
				NftColor::Xorange => {
					self.xorange = self.xorange.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xpink => {
					self.xpink = self.xpink.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xblue => {
					self.xblue = self.xblue.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xcyan => {
					self.xcyan = self.xcyan.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xcoral => {
					self.xcoral = self.xcoral.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xpurple => {
					self.xpurple = self.xpurple.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xleafgreen => {
					self.xleafgreen =
						self.xleafgreen.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xgreen => {
					self.xgreen = self.xgreen.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
			}
		}

		pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
			match color {
				NftColor::Xorange => {
					self.xorange = self.xorange.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xpink => {
					self.xpink = self.xpink.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xblue => {
					self.xblue = self.xblue.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xcyan => {
					self.xcyan = self.xcyan.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xcoral => {
					self.xcoral = self.xcoral.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xpurple => {
					self.xpurple = self.xpurple.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xleafgreen => {
					self.xleafgreen =
						self.xleafgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xgreen => {
					self.xgreen = self.xgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
			}
		}

		pub fn has_four_of_all_colors(&self) -> bool {
			self.xorange >= 4 &&
				self.xpink >= 4 && self.xblue >= 4 &&
				self.xcyan >= 4 && self.xcoral >= 4 &&
				self.xpurple >= 4 &&
				self.xleafgreen >= 4 &&
				self.xgreen >= 4
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config //+ pallet_babe::Config
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
		type MaxProperty: Get<u32> + Clone + PartialEq + Eq;
		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The maximum amount of games that can be played at the same time.
		#[pallet::constant]
		type MaxOngoingGames: Get<u32>;
		/// Randomness used for choosing a random property.
		type GameRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		/// The maximum length of data stored in string.
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	#[pallet::storage]
	#[pallet::getter(fn stored_hash)]
	pub type StoredHash<T: Config> = StorageValue<_, T::Hash, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub(super) type CurrentRound<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round_active)]
	pub(super) type RoundActive<T> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round_champion)]
	pub(super) type RoundChampion<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_color_id)]
	pub(super) type NextColorId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn collection_color)]
	pub(super) type CollectionColor<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, NftColor, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_lising_id)]
	pub(super) type NextListingId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_offer_id)]
	pub(super) type NextOfferId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_id)]
	pub type GameId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, User, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_info)]
	pub type GameInfo<T: Config> = StorageMap<_, Blake2_128Concat, u32, GameData<T>, OptionQuery>;

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

	/// A List of test properties
	#[pallet::storage]
	#[pallet::getter(fn test_properties)]
	pub type TestProperties<T: Config> =
		StorageValue<_, BoundedVec<PropertyInfoData<T>, T::MaxProperty>, ValueQuery>;

	/// Test for properties
	#[pallet::storage]
	#[pallet::getter(fn test_prices)]
	pub type TestPrices<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has received points.
		PointsReceived { receiver: AccountIdOf<T>, amount: u32 },
		/// A game has started.
		GameStarted { player: AccountIdOf<T>, game_id: u32 },
		/// An answer has been submitted.
		AnswerSubmitted { player: AccountIdOf<T>, game_id: u32 },
		/// A nft has been listed.
		NftListed { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// A nft has been delisted.
		NftDelisted { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// An offer has been made.
		OfferMade {
			owner: AccountIdOf<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		},
		/// An offer has been withdrawn.
		OfferWithdrawn { owner: AccountIdOf<T>, offer_id: u32 },
		/// An offer has been handled.
		OfferHandeld { offer_id: u32, offer: Offer },
		/// A new player has been registered
		NewPlayerRegistered { player: AccountIdOf<T> },
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
		/// The caller has no permission.
		NoThePlayer,
		/// This is not an active game.
		NoActiveGame,
		NoPermission,
		/// This listing is not listed.
		ListingDoesNotExist,
		/// This offer does not exist.
		OfferDoesNotExist,
		/// There are too many test properties.
		TooManyTest,
		/// No property could be found.
		NoProperty,
		/// The user has not yet been registered.
		UserNotRegistered,
		/// The user has already made 5 practise rounds.
		TooManyPractise,
		/// The user has not yet made a practise round.
		NoPractise,
		InvalidIndex,
		/// The color for this collection is not known.
		CollectionUnknown,
		/// There is no active round at the moment.
		NoActiveRound,
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
					let _ = Self::no_answer_result(game_info);
				}
			});
			weight
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates the setup for a new game.
		///
		/// The origin must be the sudo.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::setup_game())]
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
				let color = NftColor::from_index(x).ok_or(Error::<T>::InvalidIndex)?;
				CollectionColor::<T>::insert(collection_id, color);
			}
			Self::create_test_properties()?;
			let mut round = Self::current_round();
			round = round.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			CurrentRound::<T>::put(round);
			RoundActive::<T>::put(true);
			Ok(())
		}

		/// Registers a player and gives him initialy 50 points.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `player`: The AccountId of the user who gets registered.
		///
		/// Emits `NewPlayerRegistered` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_user())]
		pub fn register_user(origin: OriginFor<T>, player: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let user = User {
				points: 50,
				wins: Default::default(),
				losses: Default::default(),
				practise_rounds: Default::default(),
				nfts: CollectedColors::default(),
			};
			Users::<T>::insert(player.clone(), user);
			Self::deposit_event(Event::<T>::NewPlayerRegistered { player });
			Ok(())
		}

		/// Gives points to a user.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `receiver`: The AccountId of the user who gets points.
		///
		/// Emits `LocationCreated` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::give_points())]
		pub fn give_points(origin: OriginFor<T>, receiver: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let mut user = Self::users(receiver.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_add(100).ok_or(Error::<T>::ArithmeticOverflow)?;
			Users::<T>::insert(receiver.clone(), user);
			Self::deposit_event(Event::<T>::PointsReceived { receiver, amount: 100 });
			Ok(())
		}

		/// Starts a game for the player.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `game_type`: The difficulty level of the game.
		///
		/// Emits `GameStarted` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::play_game())]
		pub fn play_game(origin: OriginFor<T>, game_type: DifficultyLevel) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::check_enough_points(signer.clone(), game_type.clone())?;
			ensure!(Self::round_active(), Error::<T>::NoActiveRound);
			let game_id = Self::game_id();
			if game_type == DifficultyLevel::Player {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(8u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else if game_type == DifficultyLevel::Pro {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(5u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(10u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			}
			let (hashi, _) = T::GameRandomness::random(&[(game_id % 256) as u8]);
			StoredHash::<T>::put(hashi);
			let u32_value = u32::from_le_bytes(
				hashi.as_ref()[4..8].try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			let random_number = u32_value as usize %
				Self::test_properties()
					.len();
			let property = Self::test_properties()[random_number].clone();
			let game_datas = GameData { difficulty: game_type, player: signer.clone(), property };
			GameInfo::<T>::insert(game_id, game_datas);
			let next_game_id = game_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			GameId::<T>::put(next_game_id);
			Self::deposit_event(Event::<T>::GameStarted { player: signer, game_id });
			Ok(())
		}

		/// Checks the answer of the player and handles rewards accordingly.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `guess`: The answer of the player.
		/// - `game_id`: The id of the game that the player wants to answer to.
		///
		/// Emits `AnswerSubmitted` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_answer())]
		pub fn submit_answer(origin: OriginFor<T>, guess: u32, game_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let game_info = Self::game_info(game_id).ok_or(Error::<T>::NoActiveGame)?;
			ensure!(signer == game_info.player, Error::<T>::NoThePlayer);
			let property_id = game_info.property.id;
			let result: u32 = Self::test_prices(property_id).ok_or(Error::<T>::NoProperty)?;
			let difference_value = ((result as i32)
				.checked_sub(guess as i32)
				.ok_or(Error::<T>::ArithmeticUnderflow)?)
			.checked_mul(1000)
			.ok_or(Error::<T>::MultiplyError)?
			.checked_div(result as i32)
			.ok_or(Error::<T>::DivisionError)?
			.abs();
			Self::check_result(difference_value.try_into().unwrap(), game_id)?;
			Self::deposit_event(Event::<T>::AnswerSubmitted { player: signer, game_id });
			Ok(())
		}

		/// Lists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection id of the nft that will be listed.
		/// - `item_id`: The item id of the nft that will be listed.
		///
		/// Emits `NftListed` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_nft())]
		pub fn list_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			ensure!(
				pallet_nfts::Pallet::<T>::owner(collection_id.into(), item_id.into()) ==
					Some(signer.clone()),
				Error::<T>::NoPermission
			);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let listing_info = ListingInfo { owner: signer.clone(), collection_id, item_id };
			let mut listing_id = Self::next_lising_id();
			Listings::<T>::insert(listing_id, listing_info);
			listing_id = listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextListingId::<T>::put(listing_id);
			Self::deposit_event(Event::<T>::NftListed { owner: signer, collection_id, item_id });
			Ok(())
		}

		/// Delists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		///
		/// Emits `NftDelisted` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_nft())]
		pub fn delist_nft(origin: OriginFor<T>, listing_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let listing_info =
				Listings::<T>::take(listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_info.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
			)?;
			Self::deposit_event(Event::<T>::NftDelisted {
				owner: signer,
				collection_id: listing_info.collection_id,
				item_id: listing_info.item_id,
			});
			Ok(())
		}

		/// Makes an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		/// - `collection_id`: The collection id of the nft that will be offered.
		/// - `item_id`: The item id of the nft that will be offered.
		///
		/// Emits `OfferMade` event when succesfful.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(Self::listings(listing_id).is_some(), Error::<T>::ListingDoesNotExist);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let offer_info =
				OfferInfo { owner: signer.clone(), listing_id, collection_id, item_id };
			let offer_id = Self::next_offer_id();
			Offers::<T>::insert(offer_id, offer_info);
			let offer_id = offer_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextOfferId::<T>::put(offer_id);
			Self::deposit_event(Event::<T>::OfferMade {
				owner: signer,
				listing_id,
				collection_id,
				item_id,
			});
			Ok(())
		}

		/// Withdraw an offer.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		///
		/// Emits `OfferWithdrawn` event when succesfful.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn withdraw_offer(origin: OriginFor<T>, offer_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Self::offers(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			ensure!(offer_details.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
			)?;
			Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			Self::deposit_event(Event::<T>::OfferWithdrawn { owner: signer, offer_id });
			Ok(())
		}

		/// Handles an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		/// - `offer`: Must be either Accept or Reject.
		///
		/// Emits `OfferHandeld` event when succesfful.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::handle_offer())]
		pub fn handle_offer(origin: OriginFor<T>, offer_id: u32, offer: Offer) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			let listing_details =
				Self::listings(offer_details.listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_details.owner == signer, Error::<T>::NoPermission);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			if offer == Offer::Accept {
				pallet_nfts::Pallet::<T>::do_transfer(
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
					offer_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin.clone(),
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
				)?;
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					listing_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
				Listings::<T>::take(offer_details.listing_id)
					.ok_or(Error::<T>::ListingDoesNotExist)?;
				let mut user_offer = Self::users(offer_details.owner.clone())
					.ok_or(Error::<T>::UserNotRegistered)?;
				let color_listing = Self::collection_color(listing_details.collection_id)
					.ok_or(Error::<T>::CollectionUnknown)?;
				let color_offer = Self::collection_color(offer_details.collection_id)
					.ok_or(Error::<T>::CollectionUnknown)?;
				user_offer.add_nft_color(color_listing.clone())?;
				let points = user_offer.calculate_points(color_listing.clone());
				user_offer.points =
					user_offer.points.checked_add(points).ok_or(Error::<T>::ArithmeticOverflow)?;
 				user_offer.sub_nft_color(color_offer.clone())?;
				let points = user_offer.subtracting_calculate_points(color_offer.clone());
				user_offer.points =
					user_offer.points.checked_sub(points).ok_or(Error::<T>::ArithmeticOverflow)?; 
				Users::<T>::insert(offer_details.owner.clone(), user_offer.clone());

				let mut user_listing =
					Self::users(signer.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user_listing.add_nft_color(color_offer.clone())?;
				let points = user_listing.calculate_points(color_offer);
				user_listing.points = user_listing
					.points
					.checked_add(points)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				user_listing.sub_nft_color(color_listing.clone())?;
				let points = user_listing.subtracting_calculate_points(color_listing);
				user_listing.points = user_listing
					.points
					.checked_sub(points)
					.ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(signer.clone(), user_listing.clone());
				if user_listing.has_four_of_all_colors() {
					Self::end_game(signer.clone());
				}
				if user_offer.has_four_of_all_colors() {
					Self::end_game(offer_details.owner.clone());
				}
			} else {
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					offer_details.owner,
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
			}
			Self::deposit_event(Event::<T>::OfferHandeld { offer_id, offer });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		/// checks if the signer has enough points to start a game.
		fn check_enough_points(
			signer: AccountIdOf<T>,
			game_type: DifficultyLevel,
		) -> DispatchResult {
			if game_type == DifficultyLevel::Pro {
				ensure!(
					Self::users(signer.clone())
						.ok_or(Error::<T>::UserNotRegistered)?
						.practise_rounds > 0,
					Error::<T>::NoPractise
				);
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 50,
					Error::<T>::NotEnoughPoints
				);
			} else if game_type == DifficultyLevel::Player {
				ensure!(
					Self::users(signer.clone())
						.ok_or(Error::<T>::UserNotRegistered)?
						.practise_rounds > 0,
					Error::<T>::NoPractise
				);
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 25,
					Error::<T>::NotEnoughPoints
				);
			} else {
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.practise_rounds < 5,
					Error::<T>::TooManyPractise
				);
			}
			Ok(())
		}

		/// checks the answer and distributes the rewards accordingly.
		fn check_result(difference: u16, game_id: u32) -> DispatchResult {
			let game_info = GameInfo::<T>::take(game_id).ok_or(Error::<T>::NoActiveGame)?;
			if game_info.difficulty == DifficultyLevel::Pro {
				match difference {
					0..=10 => {
						let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
						StoredHash::<T>::put(hashi);
						let u32_value = u32::from_le_bytes(
							hashi.as_ref()[4..8]
								.try_into()
								.map_err(|_| Error::<T>::ConversionError)?,
						);
						let random_number = (u32_value % 8)
							.checked_add(
								8 * (Self::current_round()
									.checked_sub(1)
									.ok_or(Error::<T>::ArithmeticUnderflow)?),
							)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						let collection_id: <T as pallet::Config>::CollectionId =
							random_number.into();
						let next_item_id = Self::next_color_id(collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id =
							next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player.clone(),
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
						let pallet_origin: OriginFor<T> =
							RawOrigin::Signed(Self::account_id()).into();
						pallet_nfts::Pallet::<T>::lock_item_transfer(
							pallet_origin,
							collection_id.into(),
							item_id.into(),
						)?;
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						let color = Self::collection_color(collection_id)
							.ok_or(Error::<T>::CollectionUnknown)?;
						user.add_nft_color(color.clone())?;
						let points = user.calculate_points(color);
						user.points = user
							.points
							.checked_add(points)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user.clone());
						if user.has_four_of_all_colors() {
							Self::end_game(game_info.player.clone());
						}
					},
					11..=30 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(50).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					31..=50 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(30).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					51..=100 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(10).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					101..=150 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					151..=200 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					201..=250 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(30).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					251..=300 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(40).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					_ => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
				}
			} else if game_info.difficulty == DifficultyLevel::Player {
				match difference {
					0..=10 => {
						let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
						StoredHash::<T>::put(hashi);
						let u32_value = u32::from_le_bytes(
							hashi.as_ref()[4..8]
								.try_into()
								.map_err(|_| Error::<T>::ConversionError)?,
						);
						let random_number = (u32_value % 8)
							.checked_add(
								8 * (Self::current_round()
									.checked_sub(1)
									.ok_or(Error::<T>::ArithmeticUnderflow)?),
							)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						let collection_id: <T as pallet::Config>::CollectionId =
							random_number.into();
						let next_item_id = Self::next_color_id(collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id =
							next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player.clone(),
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
						let pallet_origin: OriginFor<T> =
							RawOrigin::Signed(Self::account_id()).into();
						pallet_nfts::Pallet::<T>::lock_item_transfer(
							pallet_origin,
							collection_id.into(),
							item_id.into(),
						)?;
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						let color = Self::collection_color(collection_id)
							.ok_or(Error::<T>::CollectionUnknown)?;
						user.add_nft_color(color.clone())?;
						let points = user.calculate_points(color);
						user.points = user
							.points
							.checked_add(points)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user.clone());
						if user.has_four_of_all_colors() {
							Self::end_game(game_info.player.clone());
						}
					},
					11..=30 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					31..=50 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					51..=100 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					101..=150 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					151..=200 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					201..=250 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					251..=300 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					_ => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
				}
			} else {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
				user.practise_rounds =
					user.practise_rounds.checked_add(1).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			}
			Ok(())
		}

		/// Handles the case if the player did not answer on time.
		fn no_answer_result(game_info: GameData<T>) -> DispatchResult {
			if game_info.difficulty == DifficultyLevel::Pro {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			} else if game_info.difficulty == DifficultyLevel::Player {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			} else {
			}
			Ok(())
		}

		fn end_game(winner: AccountIdOf<T>) -> DispatchResult {
			RoundActive::<T>::put(false);
			RoundChampion::<T>::insert(Self::current_round(), winner);
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