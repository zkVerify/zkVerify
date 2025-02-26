// Copyright 2025, Horizen Labs, Inc.

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


use std::sync::OnceLock;
use sc_sysinfo::Requirements;

/// The hardware requirements as measured on reference hardware.
///
/// These values are provided by Horizenlabs, however it is possible
/// to use your own requirements if you are running a custom chain.
pub fn zkv_reference_hardware() -> &'static Requirements {
    static REFERENCE_HW: OnceLock<Requirements> = OnceLock::new();
    REFERENCE_HW.get_or_init(|| {
        let raw = include_bytes!("reference_hardware.json").as_slice();
        serde_json::from_slice(raw).expect("Hardcoded data is known good; qed")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sysinfo::Requirements;

    /// `zkv_reference_hardware()` can be decoded.
    #[test]
    fn json_static_data() {
        let raw = serde_json::to_string(zkv_reference_hardware()).unwrap();
        let decoded: Requirements = serde_json::from_str(&raw).unwrap();

        assert_eq!(&decoded, zkv_reference_hardware());
    }
}
