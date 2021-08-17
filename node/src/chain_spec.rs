use sp_core::{Pair, Public, sr25519/*, crypto::UncheckedInto*/};
use subsocial_primitives::Balance;
use subsocial_runtime::{
    AccountId, GenesisConfig, SessionKeys, StakerStatus,
    constants::currency::DOLLARS,
};
use subsocial_runtime::{
    SessionConfig, StakingConfig, BalancesConfig, SudoConfig,
    ElectionsConfig, TechnicalCommitteeConfig,
    SpacesConfig, SystemConfig, WASM_BINARY, Signature,
};
use sp_consensus_babe::AuthorityId as BabeId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    Perbill,
    traits::{Verify, IdentifyAccount},
};
use sc_service::{ChainType, Properties};
// use sc_telemetry::TelemetryEndpoints;
// use hex_literal::hex;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn testnet_endowed_accounts() -> Vec<AccountId> {
    vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
    ]
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            // Pre-funded accounts
            testnet_endowed_accounts().iter().cloned().map(|k| (k, 100_000)).collect(),
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some(DEFAULT_PROTOCOL_ID),
        // Properties
        Some(subsocial_properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || testnet_genesis(
            wasm_binary,
            // Initial PoA authorities
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
            ],
            // Sudo account
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            // Pre-funded accounts
            testnet_endowed_accounts().iter().cloned().map(|k| (k, 100_000)).collect(),
        ),
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

/*pub fn subsocial_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial.json")[..])
}*/

pub fn subsocial_staging_testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/staging.json")[..])
}

/*pub fn subsocial_staging_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Staging wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Subsocial",
        "subsocial",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            vec![
                (
                    /* AuraId SR25519 */
                    hex!["ac940b8ee399d42faeb7169f322e6623f8219d12ad4c42dfe0995fa9f9713a0d"].unchecked_into(),
                    /* GrandpaId ED25519 */
                    hex!["e97b51af33429b5c4ab8ddd9b3fc542d24154bbeef807d559eff3906afca8413"].unchecked_into()
                ),
                (
                    /* AuraId SR25519 */
                    hex!["0c053087dd7782de467228b5f826c5031be2faf315baa766a89b48bb6e2dfb71"].unchecked_into(),
                    /* GrandpaId ED25519 */
                    hex!["b48a83ed87ef39bc90c205fb551af3c076e1a952881d7fefec08cbb76e17ab8b"].unchecked_into()
                ),
            ],
            /* Sudo Account */
            hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(),
            vec![
                (
                    /* Sudo Account */
                    hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(),
                    /* Balance */
                    1_000
                ),
                (
                    /* Account X1 */
                    hex!["24d6d996a8bb42a63904afc36d610986e8d502f65898da62cb281cfe7f23b02f"].into(),
                    /* Balance */
                    2_499_000
                ),
                (
                    /* Account X2 */
                    hex!["24d6d8fc5d051fd471e275f14c83e95287d2b863e4cc802de1f78dea06c6ca78"].into(),
                    /* Balance */
                    2_500_000
                ),
                (
                    /* Account X3 */
                    hex!["24d6d901fb0531124040630e52cfd746ef7d037922c4baf290f513dbc3d47d66"].into(),
                    /* Balance */
                    2_500_000
                ),
                (
                    /* Account X4 */
                    hex!["24d6d22d63313e82f9461281cb69aacad1828dc74273274751fd24333b182c68"].into(),
                    /* Balance */
                    2_500_000
                ),
            ],
            // Treasury
            hex!["24d6d683750c4c10e90dd81430efec95133e1ec1f5be781d3267390d03174706"].into(),
            true,
        ),
        vec![],
        Some(TelemetryEndpoints::new(
            vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
        ).expect("Staging telemetry url is valid; qed")),
        Some(DEFAULT_PROTOCOL_ID),
        Some(subsocial_properties()),
        None,
    ))
}*/

fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, u128)>,
) -> GenesisConfig {
    const STASH: Balance = 100 * DOLLARS;
    let num_endowed_accounts = endowed_accounts.len();

    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts.iter().cloned().map(|(k, b)|(k, b * DOLLARS)).collect(),
        }),
        pallet_indices: Default::default(),
        pallet_babe: Default::default(),
        pallet_grandpa: Default::default(),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key.clone(),
        }),
        pallet_spaces: Some(SpacesConfig {
            endowed_account: root_key,
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.0.clone(), session_keys(
                    x.2.clone(),
                    x.3.clone(),
                    x.4.clone(),
                    x.5.clone(),
                ))
            }).collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities.iter().map(|x| {
                (
                    x.0.clone(),
                    x.1.clone(),
                    STASH,
                    StakerStatus::Validator
                )
            }).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|(member, _)| (member, STASH))
                .collect(),
        }),
        pallet_im_online: Default::default(),
        pallet_authority_discovery: Default::default(),
        pallet_democracy: Default::default(),
        pallet_collective_Instance1: Default::default(),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts.iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|(member, _)| member)
                .collect(),
            phantom: Default::default(),
        }),
        pallet_membership_Instance1: Default::default(),
        pallet_treasury: Default::default(),
        pallet_vesting: Default::default(),
    }
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
