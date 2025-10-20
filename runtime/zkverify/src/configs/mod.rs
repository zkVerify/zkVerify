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

pub use commons::*;
pub use runtime::*;
mod commons;

cfg_if::cfg_if! {
    if #[cfg(feature = "volta")] {
        #[path="volta.rs"]
        mod runtime;
    } else {
        #[path="zkverify.rs"]
        mod runtime;
    }
}
