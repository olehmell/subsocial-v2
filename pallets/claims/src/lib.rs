//! # Token Claim Module for DOT/KSM holders
//!
//! Pallet that allows DOT and KSM holders from historical snapshots to claim some tokens.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::IsSubType;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::fmt::Debug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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

        #[pallet::constant]
        type InitialClaimAmount: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type AccountsSetLimit: Get<u16>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rewards_sender)]
    pub(super) type RewardsSender<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn eligible_accounts)]
    pub(super) type EligibleAccounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn tokens_claimed_by_account)]
    pub(super) type TokensClaimedByAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_amount_claimed)]
    pub(super) type TotalAmountClaimed<T: Config> = StorageValue<_, BalanceOf<T>>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RewardsSenderSet(T::AccountId),
        RewardsSenderRemoved(),
        EligibleAccountsAdded(u16),
        TokensClaimed(T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoRewardsSenderSet,
        RewardsSenderHasInsufficientBalance,
        AddingTooManyAccountsAtOnce,
        AccountNotEligible,
        TokensAlreadyClaimed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(2, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn claim_tokens(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let rewards_sender: T::AccountId;

            if let Some(account) = Self::rewards_sender() {
                rewards_sender = account;
            } else {
                fail!(Error::<T>::NoRewardsSenderSet);
            }

            Self::ensure_allowed_to_claim_tokens(&who)?;
            Self::ensure_rewards_account_has_sufficient_balance(&rewards_sender)?;

            let amount = T::InitialClaimAmount::get();

            <T as pallet_utils::Config>::Currency::transfer(&rewards_sender, &who, amount, ExistenceRequirement::KeepAlive)?;

            <TokensClaimedByAccount<T>>::insert(&who, amount);
            <TotalAmountClaimed<T>>::mutate(|total_amount| total_amount.unwrap_or_default() + amount);

            Self::deposit_event(Event::TokensClaimed(who, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_rewards_sender(
            origin: OriginFor<T>,
            rewards_sender_opt: Option<T::AccountId>
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            if let Some(rewards_sender) = rewards_sender_opt {
                Self::ensure_rewards_account_has_sufficient_balance(&rewards_sender)?;

                <RewardsSender<T>>::put(&rewards_sender);
                Self::deposit_event(Event::RewardsSenderSet(rewards_sender));
            } else {
                <RewardsSender<T>>::kill();
                Self::deposit_event(Event::RewardsSenderRemoved());
            }

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn add_eligible_accounts(
            origin: OriginFor<T>,
            eligible_accounts: Vec<T::AccountId>
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let accounts_len = eligible_accounts.len();
            let accounts_set_limit = T::AccountsSetLimit::get();

            ensure!(
                accounts_len < accounts_set_limit.into(),
                Error::<T>::AddingTooManyAccountsAtOnce
            );

            for eligible_account in eligible_accounts {
                <EligibleAccounts<T>>::insert(&eligible_account, true);
            }

            Self::deposit_event(Event::EligibleAccountsAdded(accounts_len as u16));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(super) fn ensure_allowed_to_claim_tokens(who: &T::AccountId) -> DispatchResultWithPostInfo {
            ensure!(Self::eligible_accounts(who), Error::<T>::AccountNotEligible);
            ensure!(Self::tokens_claimed_by_account(who).is_zero(), Error::<T>::TokensAlreadyClaimed);
            Ok(().into())
        }

        pub(super) fn ensure_rewards_account_has_sufficient_balance(
            rewards_sender: &T::AccountId
        ) -> DispatchResultWithPostInfo {
            ensure!(
                T::Currency::free_balance(rewards_sender) >=
                T::Currency::minimum_balance() + T::InitialClaimAmount::get(),
                Error::<T>::RewardsSenderHasInsufficientBalance
             );
            Ok(().into())
        }
    }
}

/// Validate `claim_tokens` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct EnsureAllowedToClaimTokens<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>;

impl<T: Config + Send + Sync> Debug for EnsureAllowedToClaimTokens<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "EnsureAllowedToClaimTokens")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> EnsureAllowedToClaimTokens<T>
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

impl<T: Config + Send + Sync> SignedExtension for EnsureAllowedToClaimTokens<T>
    where
        <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    type AccountId = T::AccountId;
    type Call = <T as frame_system::Config>::Call;
    type AdditionalSigned = ();
    type Pre = ();

    const IDENTIFIER: &'static str = "EnsureAllowedToClaimTokens";

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::claim_tokens() = local_call {
                Pallet::<T>::ensure_allowed_to_claim_tokens(who)
                    .map_err(|_| InvalidTransaction::Custom(ValidityError::NotAllowedToClaim.into()))?;
            }
        }
        Ok(ValidTransaction::default())
    }
}