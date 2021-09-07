#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::IsSubType;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction, InvalidTransaction}
};
use sp_std::fmt::Debug;

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure, fail,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement},
        weights::{DispatchClass, Pays},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use sp_std::vec::Vec;

    use pallet_utils::BalanceOf;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_utils::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type InitialClaimAmount: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn claim_by_account)]
    pub(super) type ClaimByAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn eligible_claim_account)]
    pub(super) type EligibleClaimAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_account)]
    pub(super) type RewardsAccount<T: Config> = StorageValue<_, T::AccountId>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Claimed(T::AccountId, BalanceOf<T>),
        RewardsAccountAdded(T::AccountId),
        EligibleAccountsAdded(u16),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        TokensAlreadyClaimed,
        AccountNotEligible,
        NoRewardsAccount,
        NoFreeBalanceOnRewardsAccount,
        ToManyItems,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().writes(1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn tokens_claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(Self::eligible_claim_account(&who), Error::<T>::AccountNotEligible);
            ensure!(!Self::claim_by_account(&who).is_zero(), Error::<T>::TokensAlreadyClaimed);

            let rewards_account;

            if let Some(account) = Self::rewards_account() {
                rewards_account = account;
            } else {
                fail!(Error::<T>::NoRewardsAccount);
            }

            let amount = T::InitialClaimAmount::get();

            <T as pallet_utils::Config>::Currency::transfer(&rewards_account, &who, amount, ExistenceRequirement::KeepAlive)?;

            <ClaimByAccount<T>>::insert(&who, amount);

            Self::deposit_event(Event::Claimed(who, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_account(origin: OriginFor<T>, reward_account: T::AccountId) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(
                T::Currency::free_balance(&reward_account) >=
                T::Currency::minimum_balance(),
                Error::<T>::NoFreeBalanceOnRewardsAccount
            );

            <RewardsAccount<T>>::put(&reward_account);

            Self::deposit_event(Event::RewardsAccountAdded(reward_account));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_eligible_claim_accounts(origin: OriginFor<T>, eligible_claim_accounts: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let accounts_len = eligible_claim_accounts.len();

            // ensure!(
            //     accounts_len < 10000,
            //     Error::<T>::ToManyItems
            // );

            for eligible_claim_account in eligible_claim_accounts {
                <EligibleClaimAccounts<T>>::insert(&eligible_claim_account, true);
            }

            Self::deposit_event(Event::EligibleAccountsAdded(accounts_len as u16));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(super) fn prevalidate_tokens_claim(who: &T::AccountId) -> DispatchResultWithPostInfo {
            ensure!(Self::eligible_claim_account(who), Error::<T>::AccountNotEligible);
            ensure!(!Self::claim_by_account(who).is_zero(), Error::<T>::TokensAlreadyClaimed);
            Ok(().into())
        }
    }
}

/// Validate `tokens_claim` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PrevalidateTokenClaim<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>;

impl<T: Config + Send + Sync> Debug for PrevalidateTokenClaim<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "PrevalidateTokenClaim")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> PrevalidateTokenClaim<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    /// Create new `SignedExtension` to check runtime version.
    pub fn new() -> Self {
        Self(sp_std::marker::PhantomData)
    }
}

#[repr(u8)]
enum ValidityError {
    NotAllowedToClaim = 0,
}

impl From<ValidityError> for u8 {
    fn from(err: ValidityError) -> Self {
        err as u8
    }
}

impl<T: Config + Send + Sync> SignedExtension for PrevalidateTokenClaim<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    type AccountId = T::AccountId;
    type Call = <T as frame_system::Config>::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "PrevalidateTokenClaim";

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    // <weight>
    // The weight of this logic is included in the `attest` dispatchable.
    // </weight>
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::tokens_claim() = local_call {
                Pallet::<T>::prevalidate_tokens_claim(who)
                    .map_err(|_| InvalidTransaction::Custom(ValidityError::NotAllowedToClaim.into()))?;
            }
        }
        Ok(ValidTransaction::default())
    }
}