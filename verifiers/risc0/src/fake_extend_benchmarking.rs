// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
#![cfg(all(feature = "runtime-benchmarks", not(feature = "extend-benchmarks")))]

//! That's a placeholder for the extended benchmarking module. If you run
//! them they'll not fail but just log some error messages.

use super::Risc0;
use frame_benchmarking::v2::*;

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub use crate::benchmarking::{Call, Config};

#[benchmarks(where T: pallet_verifiers::Config<Risc0<T>> + pallet_aggregate::Config)]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn fake() {
        log::error!("ERROR: ***** You should enable extend-benchmarks feature to run these benchmarks. ***** ");
        #[block]
        {
            ()
        }
    }
}
