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

//! Test environment for Assets pallet.

use super::*;
use crate as pallet_assets;

use frame_support::{
    construct_runtime,
    traits::{ConstU32, ConstU64, GenesisBuild},
};
use frame_support_test::TestRandomness;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const CUSTODIAN: u64 = 1;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<2>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type FreezeIdentifier = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u64;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU64<1>;
    type AssetAccountDeposit = ConstU64<10>;
    type MetadataDepositBase = ConstU64<1>;
    type MetadataDepositPerByte = ConstU64<1>;
    type ApprovalDeposit = ConstU64<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = TestFreezer;
    type WeightInfo = ();
    type Extra = ();
    type Randomness = TestRandomness<Self>;
}

use std::{cell::RefCell, collections::HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Hook {
    Died(AssetId, u64),
}
thread_local! {
    static FROZEN: RefCell<HashMap<(AssetId, u64), u64>> = RefCell::new(Default::default());
    static HOOKS: RefCell<Vec<Hook>> = RefCell::new(Default::default());
}

pub struct TestFreezer;
impl FrozenBalance<AssetId, u64, u64> for TestFreezer {
    fn frozen_balance(asset: AssetId, who: &u64) -> Option<u64> {
        FROZEN.with(|f| f.borrow().get(&(asset, *who)).cloned())
    }

    fn died(asset: AssetId, who: &u64) {
        HOOKS.with(|h| h.borrow_mut().push(Hook::Died(asset, *who)));
        // Sanity check: dead accounts have no balance.
        assert!(Assets::balance(asset, *who).is_zero());
    }
}

pub(crate) fn set_frozen_balance(asset: AssetId, who: u64, amount: u64) {
    FROZEN.with(|f| f.borrow_mut().insert((asset, who), amount));
}

pub(crate) fn clear_frozen_balance(asset: AssetId, who: u64) {
    FROZEN.with(|f| f.borrow_mut().remove(&(asset, who)));
}

pub(crate) fn hooks() -> Vec<Hook> {
    HOOKS.with(|h| h.borrow().clone())
}

pub(crate) fn take_hooks() -> Vec<Hook> {
    HOOKS.with(|h| h.take())
}

pub const PREEXIST_ASSET: [u8; 24] = [99u8; 24];

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let config: pallet_assets::GenesisConfig<Test> = pallet_assets::GenesisConfig {
        custodian: Some(CUSTODIAN),
        assets: vec![
            // id, owner, is_sufficient, min_balance
            (PREEXIST_ASSET, 0, true, 1),
        ],
        metadata: vec![
            // id, name, symbol, decimals
            (PREEXIST_ASSET, "Token Name".into(), "TOKEN".into(), 10),
        ],
        accounts: vec![
            // id, account_id, balance
            (PREEXIST_ASSET, 1, 100),
        ],
    };

    config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();
    // Clear thread local vars for https://github.com/paritytech/substrate/issues/10479.
    ext.execute_with(take_hooks);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub(crate) fn test_ext_no_custodian() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let config: pallet_assets::GenesisConfig<Test> = pallet_assets::GenesisConfig {
        ..Default::default()
    };

    config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();
    // Clear thread local vars for https://github.com/paritytech/substrate/issues/10479.
    ext.execute_with(take_hooks);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
