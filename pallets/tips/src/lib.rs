#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

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

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config + pallet_utils::Config + pallet_spaces::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tip from Account to owner of post with optional comment
        /// parameters. [fromAccount, toPostId, amount, Option<CommentId>]
        TipToPost(T::AccountId, PostId, BalanceOf<T>, Option<PostId>),

        /// Tip from Account to owner of space with optional comment
        /// parameters. [fromAccount, toSpaceId, amount, Option<CommentId>]
        TipToSpace(T::AccountId, SpaceId, BalanceOf<T>, Option<PostId>),

        /// Tip from Account to another Account with optional comment
        /// parameters. [fromAccount, toAccount, amount, Option<CommentId>]
        TipToAccount(T::AccountId, T::AccountId, BalanceOf<T>, Option<PostId>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T:Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_post(origin: OriginFor<T>, post_id: PostId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let from_account = ensure_signed(origin)?;

            let post = Posts::<T>::require_post(post_id)?;

            let to_account = post.owner;

            // transfer
            <T as pallet_utils::Config>::Currency::transfer(&from_account, &to_account, amount, ExistenceRequirement::KeepAlive)?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            // Emit an event.
            Self::deposit_event(Event::TipToPost(from_account, post_id, amount, None));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_space(origin: OriginFor<T>, space_id: SpaceId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let from_account = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;

            let to_account = space.owner;

            // transfer
            <T as pallet_utils::Config>::Currency::transfer(&from_account, &to_account, amount, ExistenceRequirement::KeepAlive)?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            // Emit an event.
            Self::deposit_event(Event::TipToSpace(from_account, space_id, amount, None));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn tip_account(origin: OriginFor<T>, to_account: T::AccountId, amount: BalanceOf<T>, content_opt: Option<Content>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let from_account = ensure_signed(origin)?;

            // transfer
            <T as pallet_utils::Config>::Currency::transfer(&from_account, &to_account, amount, ExistenceRequirement::KeepAlive)?;

            if let Some(_content) = content_opt {
                // TODO: create comment here
            }

            // Emit an event.
            Self::deposit_event(Event::TipToAccount(from_account, to_account, amount, None));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }
    }
}