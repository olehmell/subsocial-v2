#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use pallet_utils::{SpaceId};
use pallet_spaces::{Module as Spaces};

pub mod rpc;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount {
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<SpaceId>,
}

// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// pub struct Profile<T: Config> {
//     pub created: WhoAndWhen<T>,
//     pub updated: Option<WhoAndWhen<T>>,
//     pub content: Content
// }
//
// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// pub struct ProfileUpdate {
//     pub content: Option<Content>,
// }

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_utils::Config
    + pallet_spaces::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ProfilesModule {
        pub SocialAccountById get(fn social_account_by_id):
            map hasher(blake2_128_concat) T::AccountId => Option<SocialAccount>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
    {
      ProfileSet(AccountId, SpaceId),
      ProfileRemoved(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Profile already set
        ProfileAlreadySet
  }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000 + T::DbWeight::get().reads_writes(1, 2)]
    pub fn set_profile(origin, space_id: SpaceId) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let mut social_account = Self::get_or_new_social_account(owner.clone());

      ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadySet);

      let space = Spaces::<T>::require_space(space_id)?;
      space.ensure_space_owner(owner.clone())?;

      social_account.profile = Some(space_id);

      <SocialAccountById<T>>::insert(owner.clone(), social_account);

      Self::deposit_event(RawEvent::ProfileSet(owner, space_id));
      Ok(())
    }

    #[weight = 100_000 + T::DbWeight::get().reads_writes(1, 2)]
    pub fn remove_profile(origin) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Self::try_remove_profile(&owner)
    }
  }
}

impl SocialAccount {
    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn inc_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_add(1);
    }

    pub fn dec_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_sub(1);
    }

    pub fn inc_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_add(1);
    }

    pub fn dec_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_sub(1);
    }
}

impl SocialAccount {
    #[allow(clippy::comparison_chain)]
    pub fn change_reputation(&mut self, diff: i16) {
        if diff > 0 {
            self.reputation = self.reputation.saturating_add(diff.abs() as u32);
        } else if diff < 0 {
            self.reputation = self.reputation.saturating_sub(diff.abs() as u32);
        }
    }
}

impl<T: Config> Module<T> {
    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount {
        Self::social_account_by_id(account).unwrap_or(
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
                reputation: 1,
                profile: None,
            }
        )
    }

    pub fn try_remove_profile(owner: &T::AccountId) -> DispatchResult {
      let mut social_account = Self::social_account_by_id(&owner).ok_or(Error::<T>::SocialAccountNotFound)?;

      if social_account.profile.is_some() {
        social_account.profile = None;
        <SocialAccountById<T>>::insert(owner, social_account);

        Self::deposit_event(RawEvent::ProfileRemoved(owner.clone()));
      }

      Ok(())
    }
}