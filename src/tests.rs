// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for Assets pallet.

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency, error::BadOrigin};
use pallet_balances::Error as BalancesError;
use sp_runtime::{traits::ConvertInto, TokenError};

pub const ZERO_ID: [u8;24] = [0; 24];
pub const ONE_ID: [u8;24] = [1; 24];
pub const TWO_ID: [u8;24] = [2; 24];

#[test]
fn can_mint_only_to_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 2), 0);
		assert_eq!(Assets::balance(ZERO_ID, 1), 200);
	});
}

#[test]
fn minting_too_many_insufficient_assets_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		assert_ok!(Assets::force_create(Origin::root(), ONE_ID, 1, false, 1));
		assert_ok!(Assets::force_create(Origin::root(), TWO_ID, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::mint(Origin::signed(1), ONE_ID, 100));
		assert_noop!(Assets::mint(Origin::signed(1), TWO_ID, 100), TokenError::CannotCreate);

		Balances::make_free_balance_be(&2, 1);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 100));
		assert_ok!(Assets::mint(Origin::signed(1), TWO_ID, 100));
	});
}

#[test]
fn minting_insufficient_asset_with_deposit_should_work_when_consumers_exhausted() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		assert_ok!(Assets::force_create(Origin::root(), ONE_ID, 1, false, 1));
		assert_ok!(Assets::force_create(Origin::root(), TWO_ID, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::mint(Origin::signed(1), ONE_ID, 100));
		assert_noop!(Assets::mint(Origin::signed(1), TWO_ID, 100), TokenError::CannotCreate);

		assert_ok!(Assets::touch(Origin::signed(1), TWO_ID));
		assert_eq!(Balances::reserved_balance(&1), 10);

		assert_ok!(Assets::mint(Origin::signed(1), TWO_ID, 100));
	});
}

#[test]
fn minting_insufficient_assets_with_deposit_without_consumer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		assert_noop!(Assets::mint(Origin::signed(1), ZERO_ID, 100), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_eq!(System::consumers(&1), 0);
	});
}

#[test]
fn refunding_asset_deposit_with_burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::refund(Origin::signed(1), ZERO_ID, true));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(Assets::balance(ONE_ID, 0), 0);
	});
}

#[test]
fn refunding_asset_deposit_with_burn_disallowed_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_noop!(Assets::refund(Origin::signed(1), ZERO_ID, false), Error::<Test>::WouldBurn);
	});
}

#[test]
fn refunding_asset_deposit_without_burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		assert_noop!(Assets::mint(Origin::signed(1), ZERO_ID, 100), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 100));
		assert_eq!(Assets::balance(ZERO_ID, 2), 100);
		assert_eq!(Assets::balance(ZERO_ID, 1), 0);
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_ok!(Assets::refund(Origin::signed(1), ZERO_ID, false));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(Assets::balance(ONE_ID, 0), 0);
	});
}

/// Refunding reaps an account and calls the `FrozenBalance::died` hook.
#[test]
fn refunding_calls_died_hook() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::refund(Origin::signed(1), ZERO_ID, true));

		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().accounts, 0);
		assert_eq!(hooks(), vec![Hook::Died(ZERO_ID, 1)]);
	});
}

#[test]
fn approval_lifecycle_works() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50), Error::<Test>::Unknown);
		// so we create it :)
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert_ok!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 40));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(Origin::signed(1), ZERO_ID, 2));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 0);
		assert_eq!(Assets::balance(ZERO_ID, 1), 60);
		assert_eq!(Assets::balance(ZERO_ID, 3), 40);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn transfer_approved_all_funds() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50), Error::<Test>::Unknown);
		// so we create it :)
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);

		// transfer the full amount, which should trigger auto-cleanup
		assert_ok!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 50));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 0);
		assert_eq!(Assets::balance(ZERO_ID, 1), 50);
		assert_eq!(Assets::balance(ZERO_ID, 3), 50);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn approval_deposits_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		let e = BalancesError::<Test>::InsufficientBalance;
		assert_noop!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50), e);

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Balances::reserved_balance(&1), 1);

		assert_ok!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 50));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_ok!(Assets::cancel_approval(Origin::signed(1), ZERO_ID, 2));
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn cannot_transfer_more_than_approved() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		let e = Error::<Test>::Unapproved;
		assert_noop!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 51), e);
	});
}

#[test]
fn cannot_transfer_more_than_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 101));
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 101), e);
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_noop!(Assets::cancel_approval(Origin::signed(1), ONE_ID, 2), Error::<Test>::Unknown);
		assert_noop!(Assets::cancel_approval(Origin::signed(2), ZERO_ID, 2), Error::<Test>::Unknown);
		assert_noop!(Assets::cancel_approval(Origin::signed(1), ZERO_ID, 3), Error::<Test>::Unknown);
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(Origin::signed(1), ZERO_ID, 2));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 0);
		assert_noop!(Assets::cancel_approval(Origin::signed(1), ZERO_ID, 2), Error::<Test>::Unknown);
	});
}

#[test]
fn force_cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		let e = Error::<Test>::NoPermission;
		assert_noop!(Assets::force_cancel_approval(Origin::signed(2), ZERO_ID, 1, 2), e);
		assert_noop!(
			Assets::force_cancel_approval(Origin::signed(1), ONE_ID, 1, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::force_cancel_approval(Origin::signed(1), ZERO_ID, 2, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::force_cancel_approval(Origin::signed(1), ZERO_ID, 1, 3),
			Error::<Test>::Unknown
		);
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 1);
		assert_ok!(Assets::force_cancel_approval(Origin::signed(1), ZERO_ID, 1, 2));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().approvals, 0);
		assert_noop!(
			Assets::force_cancel_approval(Origin::signed(1), ZERO_ID, 1, 2),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn lifecycle_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::create(Origin::signed(1), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		assert_eq!(Balances::reserved_balance(&1), 12);
		let id = Assets::get_current_asset_id(&1).unwrap();
		// assert_eq!(101, id);
		assert!(Asset::<Test>::contains_key(id));

		assert_eq!(Balances::reserved_balance(&1), 12);
		assert!(Metadata::<Test>::contains_key(id));

		Balances::make_free_balance_be(&10, 100);
		assert_ok!(Assets::mint(Origin::signed(1), id, 100));
		Balances::make_free_balance_be(&20, 100);
		assert_ok!(Assets::mint(Origin::signed(1), id, 100));
		assert_eq!(Account::<Test>::iter_prefix(id).count(), 1);

		let w = Asset::<Test>::get(id).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(Origin::signed(1), id, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(!Asset::<Test>::contains_key(id));
		assert!(!Metadata::<Test>::contains_key(id));
		assert_eq!(Account::<Test>::iter_prefix(id).count(), 0);

		assert_ok!(Assets::create(Origin::signed(1), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let second_id = Assets::get_current_asset_id(&1).unwrap();
		// assert_eq!(102, second_id);
		assert_eq!(Balances::reserved_balance(&1), 12);
		assert!(Asset::<Test>::contains_key(second_id));

		assert_eq!(Balances::reserved_balance(&1), 12);
		assert!(Metadata::<Test>::contains_key(second_id));

		assert_ok!(Assets::mint(Origin::signed(1), second_id, 100));
		assert_ok!(Assets::mint(Origin::signed(1), second_id, 100));
		assert_eq!(Account::<Test>::iter_prefix(second_id).count(), 1);

		let w = Asset::<Test>::get(second_id).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(Origin::root(), second_id, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(!Asset::<Test>::contains_key(second_id));
		assert!(!Metadata::<Test>::contains_key(second_id));
		assert_eq!(Account::<Test>::iter_prefix(second_id).count(), 0);
	});
}

#[test]
fn destroy_with_bad_witness_should_not_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		let mut w = Asset::<Test>::get(ZERO_ID).unwrap().destroy_witness();
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		// witness too low
		assert_noop!(Assets::destroy(Origin::signed(1), ZERO_ID, w), Error::<Test>::BadWitness);
		// witness too high is okay though
		w.accounts += 2;
		w.sufficients += 2;
		assert_ok!(Assets::destroy(Origin::signed(1), ZERO_ID, w));
	});
}

#[test]
fn destroy_should_refund_approvals() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 3, 50));
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 4, 50));
		assert_eq!(Balances::reserved_balance(&1), 3);

		let w = Asset::<Test>::get(ZERO_ID).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(Origin::signed(1), ZERO_ID, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		// all approvals are removed
		assert!(Approvals::<Test>::iter().count().is_zero())
	});
}

#[test]
fn non_providing_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 0, false, 1));

		Balances::make_free_balance_be(&0, 100);
		assert_ok!(Assets::mint(Origin::signed(0), ZERO_ID, 100));

		// Cannot transfer into account 1 since it doesn't (yet) exist.
		assert_noop!(Assets::transfer(Origin::signed(0), ZERO_ID, 1, 50), TokenError::CannotCreate);
		// ...or force-transfer
		assert_noop!(
			Assets::force_transfer(Origin::signed(1), ZERO_ID, 0, 1, 50),
			TokenError::CannotCreate
		);

		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(Origin::signed(0), ZERO_ID, 1, 25));
		assert_ok!(Assets::force_transfer(Origin::signed(0), ZERO_ID, 1, 2, 25));
	});
}

#[test]
fn min_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 10));
		
		// Cannot create a new account with a balance that is below minimum...
		assert_noop!(Assets::mint(Origin::signed(1), ZERO_ID, 9), TokenError::BelowMinimum);
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().accounts, 1);
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 9), TokenError::BelowMinimum);
		assert_noop!(
			Assets::force_transfer(Origin::signed(1), ZERO_ID, 1, 2, 9),
			TokenError::BelowMinimum
		);

		// When deducting from an account to below minimum, it should be reaped.
		// Death by `transfer`.
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 91));
		assert!(Assets::maybe_balance(ZERO_ID, 1).is_none());
		assert_eq!(Assets::balance(ZERO_ID, 2), 100);
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().accounts, 1);
		assert_eq!(take_hooks(), vec![Hook::Died(ZERO_ID, 1)]);

		// Death by `force_transfer`.
		assert_ok!(Assets::force_transfer(Origin::signed(1), ZERO_ID, 2, 1, 91));
		assert!(Assets::maybe_balance(ZERO_ID, 2).is_none());
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().accounts, 1);
		assert_eq!(take_hooks(), vec![Hook::Died(ZERO_ID, 2)]);

		// Death by `burn`.
		assert_ok!(Assets::burn(Origin::signed(1), ZERO_ID, 1, 91));
		assert!(Assets::maybe_balance(ZERO_ID, 1).is_none());
		assert_eq!(Asset::<Test>::get(ZERO_ID).unwrap().accounts, 0);
		assert_eq!(take_hooks(), vec![Hook::Died(ZERO_ID, 1)]);

		// Death by `transfer_approved`.
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 100));
		assert_ok!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 3, 91));
		assert_eq!(take_hooks(), vec![Hook::Died(ZERO_ID, 1)]);
	});
}

#[test]
fn querying_total_supply_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Assets::balance(ZERO_ID, 1), 50);
		assert_eq!(Assets::balance(ZERO_ID, 2), 50);
		assert_ok!(Assets::transfer(Origin::signed(2), ZERO_ID, 3, 31));
		assert_eq!(Assets::balance(ZERO_ID, 1), 50);
		assert_eq!(Assets::balance(ZERO_ID, 2), 19);
		assert_eq!(Assets::balance(ZERO_ID, 3), 31);
		assert_ok!(Assets::burn(Origin::signed(1), ZERO_ID, 3, 31));
		assert_eq!(Assets::total_supply(ZERO_ID), 69);
	});
}

#[test]
fn transferring_amount_below_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Assets::balance(ZERO_ID, 1), 50);
		assert_eq!(Assets::balance(ZERO_ID, 2), 50);
	});
}

#[test]
fn transferring_enough_to_kill_source_when_keep_alive_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 10));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_noop!(
			Assets::transfer_keep_alive(Origin::signed(1), ZERO_ID, 2, 91),
			Error::<Test>::BalanceLow
		);
		assert_ok!(Assets::transfer_keep_alive(Origin::signed(1), ZERO_ID, 2, 90));
		assert_eq!(Assets::balance(ZERO_ID, 1), 10);
		assert_eq!(Assets::balance(ZERO_ID, 2), 90);
		assert!(hooks().is_empty());
	});
}

#[test]
fn transferring_frozen_user_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::freeze(Origin::signed(1), ZERO_ID, 1));
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50), Error::<Test>::Frozen);
		assert_ok!(Assets::thaw(Origin::signed(1), ZERO_ID, 1));
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
	});
}

#[test]
fn transferring_frozen_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::freeze_asset(Origin::signed(1), ZERO_ID));
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50), Error::<Test>::Frozen);
		assert_ok!(Assets::thaw_asset(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
	});
}

#[test]
fn approve_transfer_frozen_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::freeze_asset(Origin::signed(1), ZERO_ID));
		assert_noop!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50), Error::<Test>::Frozen);
		assert_ok!(Assets::thaw_asset(Origin::signed(1), ZERO_ID));
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
	});
}

#[test]
fn origin_guards_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_noop!(
			Assets::transfer_ownership(Origin::signed(2), ZERO_ID, 2),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::freeze(Origin::signed(2), ZERO_ID, 1), Error::<Test>::NoPermission);
		assert_noop!(Assets::thaw(Origin::signed(2), ZERO_ID, 2), Error::<Test>::NoPermission);
		assert_noop!(Assets::mint(Origin::signed(2), ZERO_ID, 100), Error::<Test>::NoPermission);
		assert_noop!(Assets::burn(Origin::signed(2), ZERO_ID, 1, 100), Error::<Test>::NoPermission);
		assert_noop!(
			Assets::force_transfer(Origin::signed(2), ZERO_ID, 1, 2, 100),
			Error::<Test>::NoPermission
		);
		let w = Asset::<Test>::get(ZERO_ID).unwrap().destroy_witness();
		assert_noop!(Assets::destroy(Origin::signed(2), ZERO_ID, w), Error::<Test>::NoPermission);
	});
}

#[test]
fn transfer_owner_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::create(Origin::signed(1), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&1).unwrap();
		assert_eq!(Balances::reserved_balance(&1), 12);

		assert_ok!(Assets::transfer_ownership(Origin::signed(1), id, 2));
		assert_ok!(Assets::transfer_ownership(Origin::signed(2), id, 2));
		assert_eq!(Balances::reserved_balance(&2), 12);
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_noop!(
			Assets::transfer_ownership(Origin::signed(1), id, 1),
			Error::<Test>::NoPermission
		);

		// Set metadata now and make sure that deposit gets transferred back.
		assert_ok!(Assets::transfer_ownership(Origin::signed(2), id, 1));
		assert_eq!(Balances::reserved_balance(&1), 12);
		assert_eq!(Balances::reserved_balance(&2), 0);
	});
}

#[test]
fn transferring_to_frozen_account_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 200);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 5));
		assert_ok!(Assets::freeze(Origin::signed(1), ZERO_ID, 2));
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Assets::balance(ZERO_ID, 2), 55);
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(Assets::balance(ZERO_ID, 1), 50);
		assert_eq!(Assets::balance(ZERO_ID, 2), 50);
		assert_ok!(Assets::burn(Origin::signed(1), ZERO_ID, 1, 50));
		assert_eq!(Assets::balance(ZERO_ID, 1), 0);
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 1, 50), Error::<Test>::NoAccount);
		assert_noop!(Assets::transfer(Origin::signed(2), ZERO_ID, 1, 51), Error::<Test>::BalanceLow);
	});
}

#[test]
fn transferring_less_than_one_unit_is_fine() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 0));
		// `ForceCreated` and `Issued` but no `Transferred` event.
		assert_eq!(System::events().len(), 2);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 101), Error::<Test>::BalanceLow);
	});
}

#[test]
fn burning_asset_balance_with_zero_balance_does_nothing() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 2), 0);
		assert_noop!(Assets::burn(Origin::signed(1), ZERO_ID, 2, u64::MAX), Error::<Test>::NoAccount);
		assert_eq!(Assets::balance(ZERO_ID, 2), 0);
		assert_eq!(Assets::total_supply(ZERO_ID), 100);
	});
}

/// Destroying an asset calls the `FrozenBalance::died` hooks of all accounts.
#[test]
fn destroy_calls_died_hooks() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 50));
		// Create account 1 and 2.
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		// Destroy the asset.
		let w = Asset::<Test>::get(ZERO_ID).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(Origin::signed(1), ZERO_ID, w));

		// Asset is gone and accounts 1 and 2 died.
		assert!(Asset::<Test>::get(ZERO_ID).is_none());
		assert_eq!(hooks(), vec![Hook::Died(ZERO_ID, 1)]);
	})
}

#[test]
fn freezer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 10));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		assert_eq!(Assets::balance(ZERO_ID, 1), 100);

		// freeze 50 of it.
		set_frozen_balance(ZERO_ID, 1, 50);

		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 20));
		// cannot transfer another 21 away as this would take the non-frozen balance (30) to below
		// the minimum balance (10).
		assert_noop!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 21), Error::<Test>::BalanceLow);

		// create an approved transfer...
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::approve_transfer(Origin::signed(1), ZERO_ID, 2, 50));
		let e = Error::<Test>::BalanceLow;
		// ...but that wont work either:
		assert_noop!(Assets::transfer_approved(Origin::signed(2), ZERO_ID, 1, 2, 21), e);
		// a force transfer won't work also.
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::force_transfer(Origin::signed(1), ZERO_ID, 1, 2, 21), e);

		// reduce it to only 49 frozen...
		set_frozen_balance(ZERO_ID, 1, 49);
		// ...and it's all good:
		assert_ok!(Assets::force_transfer(Origin::signed(1), ZERO_ID, 1, 2, 21));

		// and if we clear it, we can remove the account completely.
		clear_frozen_balance(ZERO_ID, 1);
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, 50));
		assert_eq!(hooks(), vec![Hook::Died(ZERO_ID, 1)]);
	});
}

#[test]
fn imbalances_should_work() {
	use frame_support::traits::tokens::fungibles::Balanced;

	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));

		let imb = Assets::issue(ZERO_ID, 100);
		assert_eq!(Assets::total_supply(ZERO_ID), 100);
		assert_eq!(imb.peek(), 100);

		let (imb1, imb2) = imb.split(30);
		assert_eq!(imb1.peek(), 30);
		assert_eq!(imb2.peek(), 70);

		drop(imb2);
		assert_eq!(Assets::total_supply(ZERO_ID), 30);

		assert!(Assets::resolve(&1, imb1).is_ok());
		assert_eq!(Assets::balance(ZERO_ID, 1), 30);
		assert_eq!(Assets::total_supply(ZERO_ID), 30);
	});
}

#[test]
fn force_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// force set metadata works
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::force_set_metadata(
			Origin::root(),
			ZERO_ID,
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 10],
			vec![0u8; 10],
			8,
			false
		));
		assert!(Metadata::<Test>::contains_key(ZERO_ID));

		// overwrites existing metadata
		let asset_original_metadata = Metadata::<Test>::get(ZERO_ID);
		assert_ok!(Assets::force_set_metadata(
			Origin::root(),
			ZERO_ID,
			vec![1u8; 10],
			vec![1u8; 10],
			vec![0u8; 10],
			vec![0u8; 10],
			8,
			false
		));
		assert_ne!(Metadata::<Test>::get(ZERO_ID), asset_original_metadata);

		// attempt to set metadata for non-existent asset class
		assert_noop!(
			Assets::force_set_metadata(Origin::root(), ONE_ID, vec![0u8; 10], vec![0u8; 10], vec![0u8; 10],
			vec![0u8; 10], 8, false),
			Error::<Test>::Unknown
		);

		// string length limit check
		let limit = 50usize;
		assert_noop!(
			Assets::force_set_metadata(
				Origin::root(),
				ZERO_ID,
				vec![0u8; limit + 1],
				vec![0u8; 10],
				vec![0u8; 10],
				vec![0u8; 10],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);
		assert_noop!(
			Assets::force_set_metadata(
				Origin::root(),
				ZERO_ID,
				vec![0u8; 10],
				vec![0u8; limit + 1],
				vec![0u8; 10],
				vec![0u8; 10],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);
		assert_noop!(
			Assets::force_set_metadata(
				Origin::root(),
				ZERO_ID,
				vec![0u8; 10],
				vec![0u8; 10],
				vec![0u8; limit + 1],
				vec![0u8; 10],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);
		assert_noop!(
			Assets::force_set_metadata(
				Origin::root(),
				ZERO_ID,
				vec![0u8; 10],
				vec![0u8; 10],
				vec![0u8; 10],
				vec![0u8; limit + 1],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);

		// force clear metadata works
		assert!(Metadata::<Test>::contains_key(ZERO_ID));
		assert_ok!(Assets::force_clear_metadata(Origin::root(), ZERO_ID));
		assert!(!Metadata::<Test>::contains_key(ZERO_ID));

		// Error handles clearing non-existent asset class
		assert_noop!(Assets::force_clear_metadata(Origin::root(), ONE_ID), Error::<Test>::Unknown);
	});
}

#[test]
fn force_asset_status_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 10);
		assert_ok!(Assets::create(Origin::signed(1), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&1).unwrap();
		assert_ok!(Assets::mint(Origin::signed(1), id, 200));
		assert_ok!(Assets::transfer(Origin::signed(1), id, 2, 150));

		// force asset status to change min_balance > balance
		assert_ok!(Assets::force_asset_status(Origin::root(), id, 1, 1, 1, 1, 100, true, false));
		assert_eq!(Assets::balance(id, 1), 50);

		// account can recieve assets for balance < min_balance
		assert_ok!(Assets::transfer(Origin::signed(2), id, 1, 1));
		assert_eq!(Assets::balance(id, 1), 51);

		// account on outbound transfer will cleanup for balance < min_balance
		assert_ok!(Assets::transfer(Origin::signed(1), id, 2, 1));
		assert_eq!(Assets::balance(id, 1), 0);

		// won't create new account with balance below min_balance
		assert_noop!(Assets::transfer(Origin::signed(2), id, 3, 50), TokenError::BelowMinimum);

		// force asset status will not execute for non-existent class
		assert_noop!(
			Assets::force_asset_status(Origin::root(), ONE_ID, 1, 1, 1, 1, 90, true, false),
			Error::<Test>::Unknown
		);

		// account drains to completion when funds dip below min_balance
		assert_ok!(Assets::force_asset_status(Origin::root(), id, 1, 1, 1, 1, 110, true, false));
		assert_ok!(Assets::transfer(Origin::signed(2), id, 1, 110));
		assert_eq!(Assets::balance(id, 1), 200);
		assert_eq!(Assets::balance(id, 2), 0);
		assert_eq!(Assets::total_supply(id), 200);
	});
}

#[test]
fn balance_conversion_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::BalanceConversion;

		let id = [42u8; 24];
		assert_ok!(Assets::force_create(Origin::root(), id, 1, true, 10));
		let not_sufficient = [23u8; 24];
		assert_ok!(Assets::force_create(Origin::root(), not_sufficient, 1, false, 10));

		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, [123u8; 24]),
			Err(ConversionError::AssetMissing)
		);
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(
				100,
				not_sufficient
			),
			Err(ConversionError::AssetNotSufficient)
		);
		// 10 / 1 == 10 -> the conversion should 10x the value
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, id),
			Ok(100 * 10)
		);
	});
}

#[test]
fn assets_from_genesis_should_exist() {
	new_test_ext().execute_with(|| {
		assert!(Asset::<Test>::contains_key(PREEXIST_ASSET));
		assert!(Metadata::<Test>::contains_key(PREEXIST_ASSET));
		assert_eq!(Assets::balance(PREEXIST_ASSET, 1), 100);
		assert_eq!(Assets::total_supply(PREEXIST_ASSET), 100);
	});
}

#[test]
fn querying_name_symbol_and_decimals_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::metadata::Inspect;
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::force_set_metadata(
			Origin::root(),
			ZERO_ID,
			vec![0u8; 10],
			vec![1u8; 10],
			vec![0u8; 10],
			vec![0u8; 10],
			12,
			false
		));
		assert_eq!(Assets::name(ZERO_ID), vec![0u8; 10]);
		assert_eq!(Assets::symbol(ZERO_ID), vec![1u8; 10]);
		assert_eq!(Assets::decimals(ZERO_ID), 12);
	});
}

#[test]
fn querying_allowance_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::approvals::{Inspect, Mutate};
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve(ZERO_ID, &1, &2, 50));
		assert_eq!(Assets::allowance(ZERO_ID, &1, &2), 50);
		// Transfer asset 0, from owner 1 and delegate 2 to destination 3
		assert_ok!(Assets::transfer_from(ZERO_ID, &1, &2, &3, 50));
		assert_eq!(Assets::allowance(ZERO_ID, &1, &2), 0);
	});
}

#[test]
fn transfer_large_asset() {
	new_test_ext().execute_with(|| {
		let amount = u64::pow(2, 63) + 2;
		assert_ok!(Assets::force_create(Origin::root(), ZERO_ID, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), ZERO_ID, amount));
		assert_ok!(Assets::transfer(Origin::signed(1), ZERO_ID, 2, amount - 1));
	})
}

//Carbon assets flow tests

#[test]
fn set_custodian_ok() {
	new_test_ext().execute_with(|| {
		let custodian = 10;
		assert_ok!(Assets::set_custodian(Origin::root(), custodian));
		
		assert_eq!(Assets::get_custodian(), Some(custodian));
	})
}

#[test]
fn set_custodian_fail() {
	new_test_ext().execute_with(|| {
		let custodian = 10;
		assert_noop!(
			Assets::set_custodian(Origin::signed(1), custodian), 
			BadOrigin);
		
		assert!(Assets::get_custodian() != Some(custodian));
	})
}

#[test]
fn create_asset_with_generated_name() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();

		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() == 5);
		assert!(metadata.symbol.len() == 5);
		assert_eq!(metadata.deposit, 11);
		let aseet_details = Asset::<Test>::get(id).unwrap();
		assert_eq!(aseet_details.owner, user);
		assert_eq!(aseet_details.deposit, 1);
		assert_eq!(aseet_details.is_sufficient, false);
		assert_eq!(aseet_details.accounts, 0);
		assert_eq!(aseet_details.sufficients, 0);
		assert_eq!(aseet_details.approvals, 0);
		assert_eq!(aseet_details.is_frozen, false);
	})
}

#[test]
fn create_asset_ensure_user_cannot_mint() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_noop!(Assets::mint(Origin::signed(user), id, 500), 
			Error::<Test>::NoPermission);
	})
}

#[test]
fn create_asset_failed_no_custodian() {
	test_ext_no_custodian().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_noop!(
			Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()),
			Error::<Test>::NoCustodian
		);	
	})
}

#[test]
fn create_asset_failed_no_balance() {
	new_test_ext().execute_with(|| {
		let user = 4;
		assert_noop!(
			Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()),
			BalancesError::<Test>::InsufficientBalance
		);	
	})
}

#[test]
fn set_project_data_by_user() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 4);
	})
}

#[test]
fn set_project_data_by_custodian() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(CUSTODIAN), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 4);
	})
}

#[test]
fn set_project_data_second_time() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 4);

		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8, 'f' as u8]));
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 5);
	})
}

#[test]
fn set_project_data_after_mint_fail() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 4);

		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 100));
		assert_noop!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8, 'f' as u8]), 
			Error::<Test>::CannotChangeAfterMint);
		let metadata = Metadata::<Test>::get(id);
		assert!(metadata.name.len() > 0);
		assert!(metadata.symbol.len() > 0);
		assert!(metadata.url.len() == 4);
		assert!(metadata.data_ipfs.len() == 4);
	})
}

#[test]
fn set_project_data_failed() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_noop!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 "123456789012345678901234567890123456789012345678901234567".as_bytes().to_vec()),
			Error::<Test>::BadMetadata);

		assert_noop!(Assets::set_project_data(
			Origin::signed(5), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
				"1234".as_bytes().to_vec()),
			Error::<Test>::NoPermission);
	})
}

#[test]
fn custodian_mint() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));
	})
}

#[test]
fn not_custodian_cannot_mint() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_noop!(Assets::mint(Origin::signed(3), id, 500),
			Error::<Test>::NoPermission);
	})
}

#[test]
fn custodian_full_circle() {
	new_test_ext().execute_with(|| {
		let user1 = 4;
		let user2 = 5;
		Balances::make_free_balance_be(&user1, 1000);
		Balances::make_free_balance_be(&user2, 1000);
		Balances::make_free_balance_be(&CUSTODIAN, 1000);
		assert_ok!(Assets::create(Origin::signed(CUSTODIAN), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&CUSTODIAN).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(CUSTODIAN), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 1500));
		assert_eq!(1500, Assets::balance(id, CUSTODIAN));

		assert_ok!(Assets::transfer(Origin::signed(CUSTODIAN), id, user1, 500));
		assert_ok!(Assets::transfer(Origin::signed(CUSTODIAN), id, user2, 700));

		assert_ok!(Assets::burn(Origin::signed(CUSTODIAN), id, user1, 100));
		assert_eq!(400, Assets::balance(id, user1));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user1, id));

		assert_ok!(Assets::burn(Origin::signed(CUSTODIAN), id, user2, 100));
		assert_eq!(600, Assets::balance(id, user2));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user2, id));
	})
}

#[test]
fn custodian_burn() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));

		assert_ok!(Assets::burn(Origin::signed(CUSTODIAN), id, user, 100));
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));
	})
}

#[test]
fn custodian_burn_several_times() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));

		assert_ok!(Assets::burn(Origin::signed(CUSTODIAN), id, user, 100));
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));

		assert_ok!(Assets::burn(Origin::signed(CUSTODIAN), id, user, 111));
		assert_eq!(289, Assets::balance(id, user));
		assert_eq!(Some(211), BurnCertificate::<Test>::get(user, id));
	})
}

#[test]
fn user_self_burn() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));

		assert_ok!(Assets::self_burn(Origin::signed(user), id, 100));
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));

		// burn second time
		assert_ok!(Assets::self_burn(Origin::signed(user), id, 100));
		assert_eq!(300, Assets::balance(id, user));
		assert_eq!(Some(200), BurnCertificate::<Test>::get(user, id));
	})
}

#[test]
fn user_cannot_self_burn_more() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));

		assert_ok!(Assets::self_burn(Origin::signed(user), id, 100));
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));

		// burn more than owned
		assert_noop!(Assets::self_burn(Origin::signed(user), id, 500),
			Error::<Test>::BalanceLow);
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));
	})
}

#[test]
fn custodian_cannot_burn_more() {
	new_test_ext().execute_with(|| {
		let user = 4;
		Balances::make_free_balance_be(&user, 1000);
		assert_ok!(Assets::create(Origin::signed(user), "Token".as_bytes().to_vec(), "Token".as_bytes().to_vec()));
		let id = Assets::get_current_asset_id(&user).unwrap();
		
		assert_ok!(Assets::set_project_data(
			Origin::signed(user), id, vec!['h' as u8,'t' as u8,'t'  as u8 ,'p' as u8],
			 vec!['4' as u8,'h' as u8,'6' as u8,'g' as u8]));
			
		assert_ok!(Assets::mint(Origin::signed(CUSTODIAN), id, 500));
		assert_eq!(500, Assets::balance(id, user));

		assert_ok!(Assets::self_burn(Origin::signed(user), id, 100));
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));

		// burn more than owned
		assert_noop!(Assets::burn(Origin::signed(CUSTODIAN), id, user, 500),
			Error::<Test>::BalanceLow);
		assert_eq!(400, Assets::balance(id, user));
		assert_eq!(Some(100), BurnCertificate::<Test>::get(user, id));
	})
}