//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use subsocial_primitives::{AccountId, Balance, Index, BlockNumber, Hash, Block};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sp_block_builder::BlockBuilder;
use sp_consensus_babe::BabeApi;
use sp_keystore::SyncCryptoStorePtr;
pub use sc_rpc_api::DenyUnsafe;
use sp_transaction_pool::TransactionPool;
use sc_client_api::AuxStore;

use sc_consensus_babe::{Config, Epoch};
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_consensus_epochs::SharedEpochChanges;
use sc_finality_grandpa::{
    SharedVoterState, SharedAuthoritySet, FinalityProofProvider, GrandpaJustificationStream
};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
use sc_rpc::SubscriptionTaskExecutor;
use sp_consensus::SelectChain;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
}

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, SC, B>(
    deps: FullDeps<C, P, SC, B>,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + AuxStore + HeaderMetadata<Block, Error=BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: posts_rpc::PostsRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: profile_follows_rpc::ProfileFollowsRuntimeApi<Block, AccountId>,
    C::Api: profiles_rpc::ProfilesRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: reactions_rpc::ReactionsRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: roles_rpc::RolesRuntimeApi<Block, AccountId>,
    C::Api: space_follows_rpc::SpaceFollowsRuntimeApi<Block, AccountId>,
    C::Api: spaces_rpc::SpacesRuntimeApi<Block, AccountId, BlockNumber>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

    use posts_rpc::{Posts, PostsApi};
    use profile_follows_rpc::{ProfileFollows, ProfileFollowsApi};
    use profiles_rpc::{Profiles, ProfilesApi};
    use reactions_rpc::{Reactions, ReactionsApi};
    use roles_rpc::{Roles, RolesApi};
    use space_follows_rpc::{SpaceFollows, SpaceFollowsApi};
    use spaces_rpc::{Spaces, SpacesApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        select_chain,
        chain_spec,
        deny_unsafe,
        babe,
        grandpa,
    } = deps;

    let BabeDeps {
        keystore,
        babe_config,
        shared_epoch_changes,
    } = babe;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    io.extend_with(
        SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))
    );

    io.extend_with(
        TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
    );

    io.extend_with(
        sc_consensus_babe_rpc::BabeApi::to_delegate(
            BabeRpcHandler::new(
                client.clone(),
                shared_epoch_changes.clone(),
                keystore,
                babe_config,
                select_chain,
                deny_unsafe,
            ),
        )
    );
    io.extend_with(
        sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
            GrandpaRpcHandler::new(
                shared_authority_set.clone(),
                shared_voter_state,
                justification_stream,
                subscription_executor,
                finality_provider,
            )
        )
    );

    io.extend_with(
        sc_sync_state_rpc::SyncStateRpcApi::to_delegate(
            sc_sync_state_rpc::SyncStateRpcHandler::new(
                chain_spec,
                client.clone(),
                shared_authority_set,
                shared_epoch_changes,
                deny_unsafe,
            )
        )
    );

    io.extend_with(
        PostsApi::to_delegate(Posts::new(client.clone()),
    ));

    io.extend_with(
        ProfileFollowsApi::to_delegate(ProfileFollows::new(client.clone()),
    ));

    io.extend_with(
        ProfilesApi::to_delegate(Profiles::new(client.clone()),
    ));

    io.extend_with(
        ReactionsApi::to_delegate(Reactions::new(client.clone()),
    ));

    io.extend_with(
        RolesApi::to_delegate(Roles::new(client.clone()),
    ));

    io.extend_with(
        SpacesApi::to_delegate(Spaces::new(client.clone()),
    ));

    io.extend_with(
        SpaceFollowsApi::to_delegate(SpaceFollows::new(client),
    ));

    io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(
    deps: LightDeps<C, F, P>,
) -> jsonrpc_core::IoHandler<M> where
    C: sp_blockchain::HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: sc_client_api::light::Fetcher<Block> + 'static,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use substrate_frame_rpc_system::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(
        SystemApi::<Hash, AccountId, Index>::to_delegate(LightSystem::new(client, remote_blockchain, fetcher, pool))
    );

    io
}
