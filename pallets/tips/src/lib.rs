#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::{Currency, ExistenceRequirement}};
    use frame_system::pallet_prelude::*;
    use pallet_posts::{Module as Posts};
    use pallet_spaces::{Module as Spaces};
    use pallet_utils::{PostId, SpaceId, BalanceOf, Content};

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config + pallet_utils::Config + pallet_spaces::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tip from Account to owner of post with optional comment
        /// parameters. [fromAccount, toAccount, amount, toPostId, Option<CommentId>]
        PostTipped(T::AccountId, T::AccountId, BalanceOf<T>, PostId, Option<PostId>),

        /// Tip from Account to owner of space with optional comment
        /// parameters. [fromAccount, toAccount, amount, toSpaceId, Option<CommentId>]
        SpaceTipped(T::AccountId, T::AccountId, BalanceOf<T>, SpaceId, Option<PostId>),

        /// Tip from Account to another Account with optional comment
        /// parameters. [fromAccount, toAccount, amount, Option<CommentId>]
        AccountTipped(T::AccountId, T::AccountId, BalanceOf<T>, Option<PostId>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_post(origin: OriginFor<T>, post_id: PostId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            let from_account = ensure_signed(origin)?;

            let post = Posts::<T>::require_post(post_id)?;

            let to_account = post.owner;

            <T as pallet_utils::Config>::Currency::transfer(&from_account, &to_account, amount, ExistenceRequirement::KeepAlive)?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            Self::deposit_event(Event::PostTipped(from_account, to_account, amount, post_id, None));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_space(origin: OriginFor<T>, space_id: SpaceId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            let from_account = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;

            let to_account = space.owner;

            <T as pallet_utils::Config>::Currency::transfer(
                &from_account,
                &to_account,
                amount,
                ExistenceRequirement::KeepAlive
            )?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            Self::deposit_event(Event::SpaceTipped(from_account, to_account, amount, space_id, None));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_account(origin: OriginFor<T>, to_account: T::AccountId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            let from_account = ensure_signed(origin)?;

            <T as pallet_utils::Config>::Currency::transfer(&from_account, &to_account, amount, ExistenceRequirement::KeepAlive)?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            Self::deposit_event(Event::AccountTipped(from_account, to_account, amount, None));
            Ok(().into())
        }
    }
}