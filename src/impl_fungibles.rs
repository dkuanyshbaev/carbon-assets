// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
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

//! Implementations for fungibles trait.

use super::*;
use frame_support::{
    defensive,
    traits::tokens::{
        Fortitude,
        Precision::{self, BestEffort},
        Preservation::{self, Expendable},
        Provenance::{self, Minted},
    },
};

use frame_support::traits::fungibles::Mutate;
use frame_support::traits::tokens::Fortitude::Force;

impl<T: Config<I>, I: 'static> fungibles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T, I> {
    type AssetId = types::AssetId;
    type Balance = T::Balance;

    fn total_issuance(asset: AssetId) -> Self::Balance {
        Asset::<T, I>::get(asset)
            .map(|x| x.supply)
            .unwrap_or_else(Zero::zero)
    }

    fn minimum_balance(asset: AssetId) -> Self::Balance {
        Asset::<T, I>::get(asset)
            .map(|x| x.min_balance)
            .unwrap_or_else(Zero::zero)
    }

    fn balance(asset: AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
        Pallet::<T, I>::balance(asset, who)
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &<T as SystemConfig>::AccountId,
        preservation: Preservation,
        _: Fortitude,
    ) -> Self::Balance {
        Pallet::<T, I>::reducible_balance(asset, who, !matches!(preservation, Expendable))
            .unwrap_or(Zero::zero())
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &<T as SystemConfig>::AccountId,
        amount: Self::Balance,
        provenance: Provenance,
    ) -> DepositConsequence {
        Pallet::<T, I>::can_increase(asset, who, amount, provenance == Minted)
    }

    fn can_withdraw(
        asset: AssetId,
        who: &<T as SystemConfig>::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        Pallet::<T, I>::can_decrease(asset, who, amount, false)
    }

    fn asset_exists(asset: AssetId) -> bool {
        Asset::<T, I>::contains_key(asset)
    }

    fn total_balance(asset: AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
        Pallet::<T, I>::balance(asset, who)
    }
}

impl<T: Config<I>, I: 'static> fungibles::Mutate<<T as SystemConfig>::AccountId> for Pallet<T, I> {
    fn done_mint_into(
        asset_id: Self::AssetId,
        beneficiary: &<T as SystemConfig>::AccountId,
        amount: Self::Balance,
    ) {
        Self::deposit_event(Event::Issued {
            asset_id,
            owner: beneficiary.clone(),
            amount,
        })
    }

    fn done_burn_from(
        asset_id: Self::AssetId,
        target: &<T as SystemConfig>::AccountId,
        balance: Self::Balance,
    ) {
        Self::deposit_event(Event::Burned {
            asset_id,
            owner: target.clone(),
            balance,
        });
    }

    fn done_transfer(
        asset_id: Self::AssetId,
        source: &<T as SystemConfig>::AccountId,
        dest: &<T as SystemConfig>::AccountId,
        amount: Self::Balance,
    ) {
        Self::deposit_event(Event::Transferred {
            asset_id,
            from: source.clone(),
            to: dest.clone(),
            amount,
        });
    }
}

impl<T: Config<I>, I: 'static> fungibles::Unbalanced<T::AccountId> for Pallet<T, I> {
    fn set_total_issuance(id: AssetId, amount: Self::Balance) {
        Asset::<T, I>::mutate_exists(id, |maybe_asset| {
            if let Some(ref mut asset) = maybe_asset {
                asset.supply = amount
            }
        });
    }
    fn handle_dust(_: fungibles::Dust<T::AccountId, Self>) {
        defensive!("`decrease_balance` and `increase_balance` have non-default impls; nothing else calls this; qed");
    }
    fn write_balance(
        _: Self::AssetId,
        _: &T::AccountId,
        _: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError> {
        defensive!("write_balance is not used if other functions are impl'd");
        Err(DispatchError::Unavailable)
    }

    /// Simple infallible function to force an account to have a particular balance, good for use
    /// in tests and benchmarks but not recommended for production code owing to the lack of
    /// error reporting.
    ///
    /// Returns the new balance.
    fn set_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        let b = Self::balance(asset.clone(), who);
        if b > amount {
            Self::burn_from(asset, who, b - amount, BestEffort, Force).map(|d| b.saturating_sub(d))
        } else {
            Self::mint_into(asset, who, amount - b).map(|d| b.saturating_add(d))
        }
        .unwrap_or(b)
    }
}

impl<T: Config<I>, I: 'static> fungibles::Create<T::AccountId> for Pallet<T, I> {
    fn create(
        id: AssetId,
        admin: T::AccountId,
        is_sufficient: bool,
        min_balance: Self::Balance,
    ) -> DispatchResult {
        Self::do_force_create(id, admin, is_sufficient, min_balance)
    }
}

// impl<T: Config<I>, I: 'static> fungibles::Destroy<T::AccountId> for Pallet<T, I> {
//     // fn get_destroy_witness(asset: &AssetId) -> Option<Self::DestroyWitness> {
//     //     Asset::<T, I>::get(asset).map(|asset_details| asset_details.destroy_witness())
//     // }
//
//     // fn destroy(
//     //     id: AssetId,
//     //     witness: Self::DestroyWitness,
//     //     maybe_check_owner: Option<T::AccountId>,
//     // ) -> Result<Self::DestroyWitness, DispatchError> {
//     //     Self::do_destroy(id, witness, maybe_check_owner)
//     // }
// }

impl<T: Config<I>, I: 'static> fungibles::metadata::Inspect<<T as SystemConfig>::AccountId>
    for Pallet<T, I>
{
    fn name(asset: AssetId) -> Vec<u8> {
        Metadata::<T, I>::get(asset).name.to_vec()
    }

    fn symbol(asset: AssetId) -> Vec<u8> {
        Metadata::<T, I>::get(asset).symbol.to_vec()
    }

    fn decimals(asset: AssetId) -> u8 {
        Metadata::<T, I>::get(asset).decimals
    }
}

impl<T: Config<I>, I: 'static> fungibles::metadata::Mutate<<T as SystemConfig>::AccountId>
    for Pallet<T, I>
{
    fn set(
        asset: AssetId,
        from: &<T as SystemConfig>::AccountId,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
    ) -> DispatchResult {
        Self::do_set_metadata(asset, from, name, symbol, decimals)
    }
}

impl<T: Config<I>, I: 'static> fungibles::approvals::Inspect<<T as SystemConfig>::AccountId>
    for Pallet<T, I>
{
    // Check the amount approved to be spent by an owner to a delegate
    fn allowance(
        asset: AssetId,
        owner: &<T as SystemConfig>::AccountId,
        delegate: &<T as SystemConfig>::AccountId,
    ) -> T::Balance {
        Approvals::<T, I>::get((asset, &owner, &delegate))
            .map(|x| x.amount)
            .unwrap_or_else(Zero::zero)
    }
}

impl<T: Config<I>, I: 'static> fungibles::approvals::Mutate<<T as SystemConfig>::AccountId>
    for Pallet<T, I>
{
    fn approve(
        asset: AssetId,
        owner: &<T as SystemConfig>::AccountId,
        delegate: &<T as SystemConfig>::AccountId,
        amount: T::Balance,
    ) -> DispatchResult {
        Self::do_approve_transfer(asset, owner, delegate, amount)
    }

    // Aprove spending tokens from a given account
    fn transfer_from(
        asset: AssetId,
        owner: &<T as SystemConfig>::AccountId,
        delegate: &<T as SystemConfig>::AccountId,
        dest: &<T as SystemConfig>::AccountId,
        amount: T::Balance,
    ) -> DispatchResult {
        Self::do_transfer_approved(asset, owner, delegate, dest, amount)
    }
}
