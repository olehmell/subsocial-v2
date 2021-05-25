use crate::{Module, Trait, Role, RoleIdsByUserInSpace};

use frame_support::storage::IterableStorageDoubleMap;
use sp_std::prelude::*;
use sp_std::collections::{ btree_set::BTreeSet };

use pallet_utils::{SpaceId, User};
use pallet_permissions::{SpacePermission};

impl<T: Trait> Module<T> {
    pub fn get_space_permissions_by_account(
        account: T::AccountId,
        space_id: SpaceId
    ) -> Vec<SpacePermission> {

        Self::role_ids_by_user_in_space(User::Account(account), space_id)
            .iter()
            .filter_map(Self::role_by_id)
            .flat_map(|role: Role<T>| role.permissions.into_iter())
            .collect::<BTreeSet<_>>()
            .iter().cloned().collect()
    }

    pub fn get_accounts_with_any_role_in_space(space_id: SpaceId) -> Vec<T::AccountId> {

        Self::role_ids_by_space_id(space_id)
            .iter()
            .flat_map(Self::users_by_role_id)
            .filter_map(|user| user.maybe_account())
            .collect::<BTreeSet<_>>()
            .iter().cloned().collect()
    }

    pub fn get_space_ids_for_account_with_any_role(account_id: T::AccountId) -> Vec<SpaceId> {
        let user = &User::Account(account_id);

        RoleIdsByUserInSpace::<T>::iter_prefix(user)
            .map(|(space_id, _)| space_id)
            .collect()
    }
}