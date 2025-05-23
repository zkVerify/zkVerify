// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.
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

pub mod pallet_xcm_benchmarks_fungible;
pub mod pallet_xcm_benchmarks_generic;

use frame_support::weights::Weight;

use crate::Runtime;
use core::marker::PhantomData;
use sp_runtime::BoundedVec;
use xcm::latest::AssetTransferFilter;
use xcm::{latest::prelude::*, latest::Error, DoubleEncoded};

use pallet_xcm_benchmarks_fungible::WeightInfo as XcmBalancesWeight;
use pallet_xcm_benchmarks_generic::WeightInfo as XcmGenericWeight;

/// Types of asset supported by the ZKV runtime.
pub enum AssetTypes {
    /// An asset backed by `pallet-balances`.
    Balances,
    /// Unknown asset.
    Unknown,
}

impl From<&Asset> for AssetTypes {
    fn from(asset: &Asset) -> Self {
        match asset {
            Asset {
                id:
                    AssetId(Location {
                        parents: 0,
                        interior: Here,
                    }),
                ..
            } => AssetTypes::Balances,
            _ => AssetTypes::Unknown,
        }
    }
}

trait WeighAssets {
    fn weigh_multi_assets(&self, balances_weight: Weight) -> Weight;
}

// ZKV only knows about one asset, the balances pallet.
const MAX_ASSETS: u64 = 1;

impl WeighAssets for AssetFilter {
    fn weigh_multi_assets(&self, balances_weight: Weight) -> Weight {
        match self {
            Self::Definite(assets) => assets
                .inner()
                .iter()
                .map(From::from)
                .map(|t| match t {
                    AssetTypes::Balances => balances_weight,
                    AssetTypes::Unknown => Weight::MAX,
                })
                .fold(Weight::zero(), |acc, x| acc.saturating_add(x)),
            // We don't support any NFTs on ZKV, so these two variants will always match
            // only 1 kind of fungible asset.
            Self::Wild(AllOf { .. } | AllOfCounted { .. }) => balances_weight,
            Self::Wild(AllCounted(count)) => {
                balances_weight.saturating_mul(MAX_ASSETS.min(*count as u64))
            }
            Self::Wild(All) => balances_weight.saturating_mul(MAX_ASSETS),
        }
    }
}

impl WeighAssets for Assets {
    fn weigh_multi_assets(&self, balances_weight: Weight) -> Weight {
        self.inner()
            .iter()
            .map(<AssetTypes as From<&Asset>>::from)
            .map(|t| match t {
                AssetTypes::Balances => balances_weight,
                AssetTypes::Unknown => Weight::MAX,
            })
            .fold(Weight::zero(), |acc, x| acc.saturating_add(x))
    }
}

/// Weights for `pallet_xcm` using the zkVerify node and recommended hardware.
pub struct ZKVWeight<T>(PhantomData<T>);

impl<T> xcm::latest::XcmWeightInfo<T> for ZKVWeight<T> {
    fn withdraw_asset(assets: &Assets) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::withdraw_asset())
    }
    fn reserve_asset_deposited(assets: &Assets) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::reserve_asset_deposited())
    }
    fn receive_teleported_asset(assets: &Assets) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::receive_teleported_asset())
    }
    fn query_response(
        _id: &u64,
        _response: &Response,
        _max_weight: &sp_weights::Weight,
        _src: &core::option::Option<Location>,
    ) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::query_response()
    }
    fn transfer_asset(assets: &Assets, _dest: &Location) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::transfer_asset())
    }
    fn transfer_reserve_asset(
        assets: &Assets,
        _dest: &Location,
        _xcm: &Xcm<()>,
    ) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::transfer_reserve_asset())
    }
    fn transact(
        _origin_kind: &OriginKind,
        _require_weight_at_most: &Option<Weight>,
        _call: &DoubleEncoded<T>,
    ) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::transact()
    }
    fn hrmp_new_channel_open_request(_: &u32, _: &u32, _: &u32) -> sp_weights::Weight {
        // XCM Executor does not currently support HRMP channel operations
        Weight::MAX
    }
    fn hrmp_channel_accepted(_: &u32) -> sp_weights::Weight {
        // XCM Executor does not currently support HRMP channel operations
        Weight::MAX
    }
    fn hrmp_channel_closing(_: &u32, _: &u32, _: &u32) -> sp_weights::Weight {
        // XCM Executor does not currently support HRMP channel operations
        Weight::MAX
    }
    fn clear_origin() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::clear_origin()
    }
    fn descend_origin(_: &Junctions) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::descend_origin()
    }
    fn report_error(_: &QueryResponseInfo) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::report_error()
    }
    fn deposit_asset(assets: &AssetFilter, _dest: &Location) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::deposit_asset())
    }
    fn deposit_reserve_asset(
        assets: &AssetFilter,
        _dest: &Location,
        _xcm: &Xcm<()>,
    ) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::deposit_reserve_asset())
    }
    fn exchange_asset(_: &AssetFilter, _: &Assets, _: &bool) -> sp_weights::Weight {
        // ZKV does not currently support exchange asset operations
        Weight::MAX
    }
    fn initiate_reserve_withdraw(
        assets: &AssetFilter,
        _reserve: &Location,
        _xcm: &Xcm<()>,
    ) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::initiate_reserve_withdraw())
    }
    fn initiate_teleport(
        assets: &AssetFilter,
        _dest: &Location,
        _xcm: &Xcm<()>,
    ) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmBalancesWeight::<Runtime>::initiate_teleport())
    }
    fn report_holding(_: &QueryResponseInfo, _: &AssetFilter) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::report_holding()
    }
    fn buy_execution(_: &Asset, _: &WeightLimit) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::buy_execution()
    }
    fn refund_surplus() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::refund_surplus()
    }
    fn set_error_handler(_: &Xcm<T>) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::set_error_handler()
    }
    fn set_appendix(_: &Xcm<T>) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::set_appendix()
    }
    fn clear_error() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::clear_error()
    }
    fn claim_asset(_: &Assets, _: &Location) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::claim_asset()
    }
    fn trap(_: &u64) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::trap()
    }
    fn subscribe_version(_: &u64, _: &sp_weights::Weight) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::subscribe_version()
    }
    fn unsubscribe_version() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::unsubscribe_version()
    }
    fn burn_asset(assets: &Assets) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmGenericWeight::<Runtime>::burn_asset())
    }
    fn expect_asset(assets: &Assets) -> sp_weights::Weight {
        assets.weigh_multi_assets(XcmGenericWeight::<Runtime>::expect_asset())
    }
    fn expect_origin(_: &core::option::Option<Location>) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::expect_origin()
    }
    fn expect_error(_: &core::option::Option<(u32, Error)>) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::expect_error()
    }
    fn expect_transact_status(_: &MaybeErrorCode) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::expect_transact_status()
    }
    fn query_pallet(_: &pallet_referenda::Vec<u8>, _: &QueryResponseInfo) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::query_pallet()
    }
    fn expect_pallet(
        _: &u32,
        _: &pallet_referenda::Vec<u8>,
        _: &pallet_referenda::Vec<u8>,
        _: &u32,
        _: &u32,
    ) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::expect_pallet()
    }
    fn report_transact_status(_: &QueryResponseInfo) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::report_transact_status()
    }
    fn clear_transact_status() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::clear_transact_status()
    }
    fn universal_origin(_: &Junction) -> sp_weights::Weight {
        // ZKV does not currently support universal origin operations
        Weight::MAX
    }
    fn export_message(_: &NetworkId, _: &Junctions, _: &Xcm<()>) -> sp_weights::Weight {
        // ZKV does not currently support asset locking operations
        Weight::MAX
    }
    fn lock_asset(_: &Asset, _: &Location) -> sp_weights::Weight {
        // ZKV does not currently support asset locking operations
        Weight::MAX
    }
    fn unlock_asset(_: &Asset, _: &Location) -> sp_weights::Weight {
        // ZKV does not currently support asset locking operations
        Weight::MAX
    }
    fn note_unlockable(_: &Asset, _: &Location) -> sp_weights::Weight {
        // ZKV does not currently support asset locking operations
        Weight::MAX
    }
    fn request_unlock(_: &Asset, _: &Location) -> sp_weights::Weight {
        // ZKV does not currently support asset locking operations
        Weight::MAX
    }
    fn set_fees_mode(_: &bool) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::set_fees_mode()
    }
    fn set_topic(_: &[u8; 32]) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::set_topic()
    }
    fn clear_topic() -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::clear_topic()
    }
    fn alias_origin(_: &Location) -> sp_weights::Weight {
        // XCM Executor does not currently support alias origin operations
        Weight::MAX
    }
    fn unpaid_execution(_: &WeightLimit, _: &core::option::Option<Location>) -> sp_weights::Weight {
        XcmGenericWeight::<Runtime>::unpaid_execution()
    }
    fn pay_fees(_asset: &Asset) -> Weight {
        unimplemented!("This should probably be implemented!")
        //XcmGenericWeight::<Runtime>::pay_fees()
    }
    fn initiate_transfer(
        _dest: &Location,
        _remote_fees: &Option<AssetTransferFilter>,
        _preserve_origin: &bool,
        _assets: &BoundedVec<AssetTransferFilter, MaxAssetTransferFilters>,
        _xcm: &Xcm<()>,
    ) -> Weight {
        // XCM Executor does not currently support initiate_transfer operations
        Weight::MAX
    }
    fn execute_with_origin(_: &Option<InteriorLocation>, _: &Xcm<T>) -> Weight {
        // XCM Executor does not currently support execute_with_origin operations
        Weight::MAX
    }
    fn set_hints(_hints: &BoundedVec<Hint, HintNumVariants>) -> Weight {
        // XCM Executor does not currently support set_hints operations
        Weight::MAX
    }
}
