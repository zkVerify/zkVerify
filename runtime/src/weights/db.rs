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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.2.0
//! DATE: 2025-06-06 (Y/M/D)
//! HOSTNAME: `6e834d009cf8`, CPU: `AMD Ryzen 7 7700 8-Core Processor`
//!
//! DATABASE: `RocksDb`, RUNTIME: `zkVerify Volta`
//! BLOCK-NUM: `BlockId::Number(0)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `10`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `/data/benchmark/runtime/src/weights`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   /usr/local/bin/zkv-relay
//   benchmark
//   storage
//   --weight-path=/data/benchmark/runtime/src/weights
//   --header=/data/benchmark/HEADER-APACHE2
//   --warmups=10
//   --mul
//   1.1
//   --state-version
//   1
//   --base-path
//   /data/benchmark/synced_root/

/// Storage DB weights for the `zkVerify Volta` runtime and `RocksDb`.
pub mod constants {
	use frame_support::weights::constants;
	use sp_core::parameter_types;
	use sp_weights::RuntimeDbWeight;

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			// Time to read one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 3_386, 149_864
			//   Average:  8_520
			//   Median:   8_556
			//   Std-Dev:  9440.35
			//
			// Percentiles nanoseconds:
			//   99th: 13_015
			//   95th: 11_862
			//   75th: 9_328
			read: 9_372 * constants::WEIGHT_REF_TIME_PER_NANOS,

			// Time to write one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 13_716, 8_069_713
			//   Average:  65_119
			//   Median:   30_678
			//   Std-Dev:  520015.57
			//
			// Percentiles nanoseconds:
			//   99th: 68_921
			//   95th: 40_456
			//   75th: 36_379
			write: 71_631 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use sp_weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
