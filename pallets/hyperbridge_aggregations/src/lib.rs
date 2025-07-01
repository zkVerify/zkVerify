// Copyright 2024, Horizen Labs, Inc.
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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ismp::module::IsmpModule;
use ismp::router::{PostRequest, Response, Timeout};
pub use pallet::*;

mod benchmarking;
#[cfg(test)]
mod mock;
mod weight;

// Export the benchmarking utils.
#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::utils::*;
pub use weight::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::WeightInfo;
    use alloc::vec;
    use alloy_dyn_abi::DynSolValue;
    use alloy_primitives::{B256, U256};
    use frame_support::{pallet_prelude::*, PalletId};
    use ismp::dispatcher::{DispatchPost, DispatchRequest, FeeMetadata, IsmpDispatcher};
    use ismp::host::StateMachine;
    use pallet_ismp::ModuleId;

    pub const ZKV_MODULE_ID: ModuleId = ModuleId::Pallet(PalletId(*b"ZKVE-MOD"));
    pub const PALLET_ID: PalletId = PalletId(*b"HYP-AGR!");

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_ismp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// [`IsmpDispatcher`] implementation
        type IsmpDispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = Self::Balance>
            + Default;
        /// The weight definition for this pallet
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MessageReceived,
        TimeoutProcessed,
    }

    #[pallet::error]
    pub enum Error<T> {
        MessageDispatchFailed,
    }

    // Hack for implementing the [`Default`] bound needed for
    // [`IsmpDispatcher`](ismp::dispatcher::IsmpDispatcher) and
    // [`IsmpModule`](ismp::module::IsmpModule)
    impl<T> Default for Pallet<T> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    /// Extrinsic params for evm dispatch
    #[derive(
        Clone, codec::Encode, codec::Decode, scale_info::TypeInfo, PartialEq, Eq, RuntimeDebug,
    )]
    pub struct Params<Balance> {
        /// Domain id
        pub domain_id: u32,

        /// Aggregation id
        pub aggregation_id: u64,

        /// Aggregation receipt
        pub aggregation: sp_core::H256,

        /// Destination contract
        pub module: sp_core::H160,

        /// Destination State Machine
        pub destination: StateMachine,

        /// Timeout timestamp on destination chain in seconds
        pub timeout: u64,

        /// A relayer fee for message delivery
        pub fee: Balance,
    }

    impl<T: Config> Pallet<T> {
        /// Dispatch aggregation to given EVM chain
        pub fn dispatch_aggregation(
            account: T::AccountId,
            params: Params<T::Balance>,
        ) -> DispatchResult {
            let data = DynSolValue::Tuple(vec![
                DynSolValue::Uint(U256::from(params.domain_id), 256),
                DynSolValue::Uint(U256::from(params.aggregation_id), 256),
                DynSolValue::FixedBytes(B256::from_slice(params.aggregation.as_ref()), 32),
            ]);

            let body = data.abi_encode();

            let post = DispatchPost {
                dest: params.destination,
                from: ZKV_MODULE_ID.to_bytes(),
                to: params.module.0.to_vec(),
                timeout: params.timeout,
                body,
            };

            let dispatcher = T::IsmpDispatcher::default();

            // dispatch the request
            // This call will attempt to collect the protocol fee and relayer fee from the user's account
            dispatcher
                .dispatch_request(
                    DispatchRequest::Post(post),
                    FeeMetadata {
                        payer: account,
                        fee: params.fee,
                    },
                )
                .inspect_err(|e| log::error!("ISMP dispatch failed with error: {e:?}"))
                .map_err(|_| Error::<T>::MessageDispatchFailed)?;
            Ok(())
        }
    }
}

impl<T: Config> IsmpModule for Pallet<T> {
    fn on_accept(&self, _request: PostRequest) -> Result<(), anyhow::Error> {
        // Here you would perform validations on the post request data
        // Ensure it can be executed successfully before making any state changes
        // You can also dispatch a post response after execution
        Self::deposit_event(Event::<T>::MessageReceived);
        Ok(())
    }

    fn on_response(&self, _response: Response) -> Result<(), anyhow::Error> {
        // Here you would perform validations on the post request data
        // Ensure it can be executed successfully before making any state changes
        Self::deposit_event(Event::<T>::MessageReceived);
        Ok(())
    }

    fn on_timeout(&self, _request: Timeout) -> Result<(), anyhow::Error> {
        // Here you would revert all the state changes that were made when the
        // request was initially dispatched
        Self::deposit_event(Event::<T>::TimeoutProcessed);
        Ok(())
    }
}
