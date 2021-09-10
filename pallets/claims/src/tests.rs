use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::DispatchError::BadOrigin;

// Test claim_tokens()

#[test]
fn claim_tokens_should_work() {
    ExtBuilder::build_with_account1_as_rewards_account_and_eligible_accounts().execute_with(|| {
        assert_ok!(_claim_tokens_to_account2());
    });
}

#[test]
fn claim_tokens_should_fail_when_reward_signer_is_none() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_claim_tokens_to_account2(), Error::<Test>::NoRewardsSenderSet);
    });
}


#[test]
fn claim_tokens_should_fail_when_reward_signer_has_insufficient_balance() {
    ExtBuilder::build_with_account1_as_rewards_account_and_eligible_accounts().execute_with(|| {
        assert_ok!(_claim_tokens_to_account2());
        assert_noop!(_claim_tokens_to_account3(), Error::<Test>::RewardsSenderHasInsufficientBalance);
    });
}

#[test]
fn claim_tokens_should_fail_when_account_not_eligible() {
    ExtBuilder::build_with_rewards_account().execute_with(|| {
        assert_noop!(_claim_tokens_to_account2(), Error::<Test>::AccountNotEligible);
    });
}

#[test]
fn claim_tokens_should_fail_when_the_account_already_claimed_tokens() {
    ExtBuilder::build_with_account1_as_rewards_account_and_eligible_accounts().execute_with(|| {
        assert_ok!(_claim_tokens_to_account2());
        assert_noop!(_claim_tokens_to_account2(), Error::<Test>::TokensAlreadyClaimed);
    });
}

// Test set_rewards_sender()

#[test]
fn set_rewards_sender_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_set_account1_as_rewards_sender_to_root());
    });
}

#[test]
fn set_rewards_sender_should_fail_when_account_has_insufficient_balance() {
    ExtBuilder::build_with_insufficient_balances_for_account1().execute_with(|| {
        assert_noop!(_set_account1_as_rewards_sender_to_root(), Error::<Test>::RewardsSenderHasInsufficientBalance);
    });
}

// Test add_eligible_accounts_should()

#[test]
fn add_eligible_accounts_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_add_one_eligible_account_to_root());
    });
}

#[test]
fn add_eligible_accounts_should_fail_when_origin_is_not_root() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_add_eligible_accounts_to_account2(), BadOrigin);
    });
}

#[test]
fn add_eligible_accounts_should_fail_when_trying_to_add_many_accounts() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_add_many_eligible_account_to_root(), Error::<Test>::AddingTooManyAccountsAtOnce);
    });
}
