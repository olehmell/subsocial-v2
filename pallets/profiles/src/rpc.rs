use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use pallet_utils::{bool_to_option, rpc::{FlatContent, FlatWhoAndWhen, ShouldSkip}};
use frame_system::Module as SystemModule;

use crate::{Module, Profile, SocialAccount, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatProfile<AccountId, BlockNumber> {
    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatSocialAccount<AccountId, BlockNumber> {
    pub id: AccountId,
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip", flatten))]
    pub profile: Option<FlatProfile<AccountId, BlockNumber>>,
    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub has_profile: Option<bool>,
}

impl<T: Trait> From<Profile<T>> for FlatProfile<T::AccountId, T::BlockNumber> {
    fn from(from: Profile<T>) -> Self {
        let Profile { created, updated, content } = from;

        Self {
            who_and_when: (created, updated).into(),
            content: content.into(),
        }
    }
}

impl<T: Trait> From<SocialAccount<T>> for FlatSocialAccount<T::AccountId, T::BlockNumber> {
    fn from(from: SocialAccount<T>) -> Self {
        let SocialAccount {
            followers_count, following_accounts_count, following_spaces_count, reputation, profile
        } = from;

        let has_profile = bool_to_option(profile.is_some());
        Self {
            id: T::AccountId::default(),
            followers_count,
            following_accounts_count,
            following_spaces_count,
            reputation,
            profile: profile.map(|profile| profile.into()),
            has_profile
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_social_accounts_by_ids(
        account_ids: Vec<T::AccountId>
    ) -> Vec<FlatSocialAccount<T::AccountId, T::BlockNumber>> {
        account_ids.iter()
                   .filter_map(|account| {
                       Self::social_account_by_id(account)
                           .map(|social_account| social_account.into())
                           .map(|mut flat_social_account: FlatSocialAccount<T::AccountId, T::BlockNumber>| {
                               flat_social_account.id = account.clone();
                               flat_social_account
                           })
                   })
                   .collect()
    }

    pub fn get_account_data(account: T::AccountId) -> T::AccountData {
        SystemModule::<T>::account(&account).data
    }
}