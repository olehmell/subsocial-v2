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
    fn build_configure_balances_fn(balance: Balance) -> impl Fn(&mut Storage) {
        move |storage: &mut Storage| {
            let _ = pallet_balances::GenesisConfig::<Test> {
                balances: [ACCOUNT1].iter().cloned().map(|k| (k, balance)).collect(),
            }.assimilate_storage(storage);
        }
    }

    fn _build<F>(configure_balances: F) -> TestExternalities
        where F: Fn(&mut Storage)
    {
        let storage = &mut system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        configure_balances(storage);

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub fn build() -> TestExternalities
    {
        let sufficient_balance = InitialClaimAmount::get() + ExistentialDeposit::get();
        Self::_build(Self::build_configure_balances_fn(sufficient_balance))
    }

    pub fn build_with_insufficient_balances_for_account1() -> TestExternalities
    {
        let insufficient_balance = ExistentialDeposit::get() + InitialClaimAmount::get() - 1;
        Self::_build(Self::build_configure_balances_fn(insufficient_balance))
    }

    pub(crate) fn build_with_account1_as_rewards_account_and_eligible_accounts() -> TestExternalities
    {
        let mut ext = Self::build();
        ext.execute_with(|| {
            RewardsSender::<Test>::put(ACCOUNT1);
            let eligible_accounts = vec![ACCOUNT2, ACCOUNT3, ACCOUNT4];
            assert_ok!(Claims::add_eligible_accounts(Origin::root(), eligible_accounts));
        });

        ext
    }

    fn _build_with_account1_as_rewards_account(build: fn () -> TestExternalities) -> TestExternalities {
        let mut ext = build();
        ext.execute_with(|| {
            RewardsSender::<Test>::put(ACCOUNT1);
        });

        ext
    }

    pub fn build_with_rewards_account() -> TestExternalities {
        Self::_build_with_account1_as_rewards_account(Self::build)
    }

    // pub fn build_with_insufficient_balances_and_rewards_account() -> TestExternalities {
    //     Self::_build_with_account1_as_rewards_account(Self::build_with_insufficient_balances_for_account1)
    // }

}

pub(crate) const ACCOUNT1: AccountId = 11;
pub(crate) const ACCOUNT2: AccountId = 12;
pub(crate) const ACCOUNT3: AccountId = 13;
pub(crate) const ACCOUNT4: AccountId = 14;

pub(crate) fn _claim_tokens_to_account2() -> DispatchResultWithPostInfo {
    _claim_tokens(None)
}
pub(crate) fn _claim_tokens_to_account3() -> DispatchResultWithPostInfo {
    _claim_tokens(Some(Origin::signed(ACCOUNT3)))
}
pub(crate) fn _claim_tokens_to_account4() -> DispatchResultWithPostInfo {
    _claim_tokens(Some(Origin::signed(ACCOUNT4)))
}


pub(crate) fn _claim_tokens(
    origin: Option<Origin>,
) -> DispatchResultWithPostInfo {
    Claims::claim_tokens(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
    )
}

pub(crate) fn _set_account1_as_rewards_sender_to_root() -> DispatchResultWithPostInfo {
    _set_rewards_sender(None)
}
pub(crate) fn _set_account1_as_rewards_sender_to_account2() -> DispatchResultWithPostInfo {
    _set_rewards_sender(Some(Origin::signed(ACCOUNT2)))
}

pub(crate) fn _set_rewards_sender(
    origin: Option<Origin>,
) -> DispatchResultWithPostInfo {
    Claims::set_rewards_sender(
        origin.unwrap_or_else(|| Origin::root()),
        Some(ACCOUNT1)
    )
}

pub(crate) fn _add_one_eligible_account_to_root() -> DispatchResultWithPostInfo {
    _add_eligible_accounts(None, vec![ ACCOUNT1 ])
}

pub(crate) fn _add_many_eligible_account_to_root() -> DispatchResultWithPostInfo {
    _add_eligible_accounts(None, vec![ACCOUNT1; AccountsSetLimit::get() as usize + 1])
}

pub(crate) fn _add_eligible_accounts_to_account2() -> DispatchResultWithPostInfo {
    _add_eligible_accounts(Some(Origin::signed(ACCOUNT2)), vec![])
}

pub(crate) fn _add_eligible_accounts(
    origin: Option<Origin>,
    eligible_accounts: Vec<AccountId>
) -> DispatchResultWithPostInfo {
    Claims::add_eligible_accounts(
        origin.unwrap_or_else(|| Origin::root()),
        eligible_accounts
    )
}
