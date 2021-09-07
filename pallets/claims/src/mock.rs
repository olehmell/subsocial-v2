// Creating mock runtime here
use super::*;

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header, Storage
};

use crate as claims;

use frame_support::{
    parameter_types,
    assert_ok,
    dispatch::DispatchResultWithPostInfo,
};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Utils: pallet_utils::{Module, Event<T>},
        Claims: claims::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 28;
}
impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
}

parameter_types! {
    pub const MinHandleLen: u32 = 5;
    pub const MaxHandleLen: u32 = 50;
}

impl pallet_utils::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

parameter_types! {
    pub const InitialClaimAmount: Balance = 10;
    pub const AccountsSetLimit: u16 = 30000;
}

impl claims::Config for Test {
    type Event = Event;
    type InitialClaimAmount = InitialClaimAmount;
    type AccountsSetLimit = AccountsSetLimit;
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u64;

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: [ACCOUNT1].iter().cloned().map(|k|(k, 100_000)).collect(),
        }.assimilate_storage(storage);
    }

    pub fn build() -> TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub fn build_with_account1_as_rewards_account() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| {
            RewardsAccount::<Test>::put(ACCOUNT1);
            let eligible_accounts = vec![ACCOUNT2];
            assert_ok!(Claims::add_eligible_claim_accounts(Origin::root(), eligible_accounts));
            // Add something to storage 1
            // Add something to storage to
            // Maybe assert something
        });

        ext
    }
}

pub(crate) const ACCOUNT1: AccountId = 11;
pub(crate) const ACCOUNT2: AccountId = 12;

pub(crate) fn _claim_tokens_to_account2() -> DispatchResultWithPostInfo {
    _claim_tokens(None)
}

pub(crate) fn _claim_tokens(
    origin: Option<Origin>,
) -> DispatchResultWithPostInfo {
    Claims::claim_tokens(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
    )
}
