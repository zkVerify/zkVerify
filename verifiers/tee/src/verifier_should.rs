// Copyright 2025, Horizen Labs, Inc.
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

#![cfg(test)]

use core::time::Duration;
use frame_support::parameter_types;
use hex_literal::hex;
use sp_core::{ConstU64, Get};

use super::*;
use pallet_crl::{CaNotFoundError, Crl, CrlProvider, RevokedCertId};

// Mock timestamps used to test the validity period of the tcb info
const PAST: u64 = 1737556187; // Thu, 22 Jan 2025 14:29:47 GMT
const PRESENT: u64 = 1769092187; // Thu, 22 Jan 2026 14:29:47 GMT
const FUTURE: u64 = 1800628187; // Thu, 22 Jan 2027 14:29:47 GMT

struct MockTime<T: Get<u64>>(PhantomData<T>);

impl<T: Get<u64>> UnixTime for MockTime<T> {
    fn now() -> Duration {
        Duration::new(T::get(), 0)
    }
}

/// Empty CRL for tests that don't need revoked certificates.
struct EmptyCrl;
impl CrlProvider for EmptyCrl {
    fn get_crl(_ca_name: &str) -> Result<Crl, CaNotFoundError> {
        Ok(vec![])
    }
}

/// CRL containing a revoked certificate matching the test quote.
struct RevokedCrl;
impl CrlProvider for RevokedCrl {
    fn get_crl(_ca_name: &str) -> Result<Crl, CaNotFoundError> {
        Ok(vec![RevokedCertId {
            issuer: hex!(
                "3068311a301806035504030c11496e74656c2053475820526f6f74204341311a3018060355040a0c11496e74656c20436f72706f726174696f6e3114301206035504070c0b53616e746120436c617261310b300906035504080c024341310b3009060355040613025553"
            )
            .to_vec(),
            serial_number: hex!("00956f5dcdbd1be1e94049c9d4f433ce01570bde54").to_vec(),
        }])
    }
}

parameter_types! {
    pub const CaName: &'static str = "foo";
}

struct Mock<T: UnixTime, C: CrlProvider = EmptyCrl>(PhantomData<(T, C)>);
impl<T: UnixTime, C: CrlProvider> Config for Mock<T, C> {
    type UnixTime = T;
    type Crl = C;
    type CaName = CaName;
}

#[test]
fn verify_valid_proof() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert!(Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs).is_ok());
}

#[test]
fn reject_valid_proof_with_revoked_cert() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>, RevokedCrl>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_proof() {
    let proof = include_bytes!("resources/intel/invalid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_vk_signature() {
    let vk = Vk {
        tcb_response: include_bytes!("resources/intel/invalid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::validate_vk(&vk),
        Err(hp_verifiers::VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_time() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PAST>>>>::validate_vk(&vk),
        Err(hp_verifiers::VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<FUTURE>>>>::validate_vk(&vk),
        Err(hp_verifiers::VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PAST>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<FUTURE>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::VerifyError)
    );
}

#[test]
fn reject_too_long_proof() {
    let proof = vec![0u8; crate::MAX_PROOF_LENGTH as usize + 1];
    let pubs = vec![];
    let vk = Vk {
        tcb_response: vec![],
        certificates: vec![],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::InvalidProofData)
    )
}

#[test]
fn reject_too_long_vk() {
    let proof = vec![];
    let pubs = vec![];
    let vk = Vk {
        tcb_response: vec![0u8; MAX_VK_LENGTH as usize + 1],
        certificates: vec![0u8; MAX_VK_LENGTH as usize + 1],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::InvalidVerificationKey)
    )
}

#[test]
fn reject_invalid_pubs() {
    let proof = vec![];
    let pubs = vec![0u8; crate::MAX_PUBS_LENGTH as usize + 1];
    let vk = Vk {
        tcb_response: vec![],
        certificates: vec![],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(hp_verifiers::VerifyError::InvalidInput)
    )
}
