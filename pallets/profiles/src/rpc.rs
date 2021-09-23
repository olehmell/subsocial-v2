use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use frame_system::Module as SystemModule;

use crate::{Module, SocialAccount, Config};
use pallet_utils::SpaceId;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatSocialAccount<AccountId> {
    pub id: AccountId,
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<SpaceId>,
}

impl<AccountId: Default> From<SocialAccount> for FlatSocialAccount<AccountId> {
    fn from(from: SocialAccount) -> Self {
        let SocialAccount {
            followers_count, following_accounts_count, following_spaces_count, reputation, profile
        } = from;

        Self {
            id: AccountId::default(),
            followers_count,
            following_accounts_count,
            following_spaces_count,
            reputation,
            profile: profile.map(|profile| profile.into()),
        }
    }
}

impl<T: Config> Module<T> {
    pub fn get_social_accounts_by_ids(
        account_ids: Vec<T::AccountId>
    ) -> Vec<FlatSocialAccount<T::AccountId>> {
        account_ids.iter()
                   .filter_map(|account| {
                       Self::social_account_by_id(account)
                           .map(|social_account| social_account.into())
                           .map(|mut flat_social_account: FlatSocialAccount<T::AccountId>| {
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