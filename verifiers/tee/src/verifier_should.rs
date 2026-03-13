// Copyright 2025-2026, Horizen Labs, Inc.
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
use hex_literal::hex;
use sp_core::{ConstU64, Get};

use super::*;
use pallet_crl::{CaNotFoundError, Crl, CrlProvider, RevokedCertId};

// Mock timestamps used to test the validity period of the tcb info
const PAST: u64 = 1737556187; // Thu, 22 Jan 2025 14:29:47 GMT
const PRESENT: u64 = 1769092187; // Thu, 22 Jan 2026 14:29:47 GMT
const FUTURE: u64 = 1800628187; // Thu, 22 Jan 2027 14:29:47 GMT

// Nitro attestation doc timestamp in seconds (2022-11-09T22:52:00Z)
const NITRO_NOW: u64 = 1668034320;

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

struct MockCaName;
impl CaNameProvider for MockCaName {
    fn ca_name_for(vk: &Vk) -> &'static str {
        match vk {
            Vk::Intel { .. } => "Intel_SGX_Processor",
            Vk::Nitro => "AWS_Nitro",
        }
    }
}

struct Mock<T: UnixTime, C: CrlProvider = EmptyCrl>(PhantomData<(T, C)>);
impl<T: UnixTime, C: CrlProvider> Config for Mock<T, C> {
    type UnixTime = T;
    type Crl = C;
    type CaName = MockCaName;
    type WeightInfo = ();
}

#[test]
fn verify_valid_proof() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert!(Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs).is_ok());
}

#[test]
fn reject_valid_proof_with_revoked_cert() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>, RevokedCrl>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_proof() {
    let proof = include_bytes!("resources/intel/invalid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_vk_signature() {
    let vk = Vk::Intel {
        tcb_response: include_bytes!("resources/intel/invalid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::validate_vk(&vk),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn reject_invalid_time() {
    let proof = include_bytes!("resources/intel/valid_quote.dat").to_vec();
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: include_bytes!("resources/intel/valid_tcbinfo.json").to_vec(),
        certificates: include_bytes!("resources/intel/valid_tcbinfo_certs.pem").to_vec(),
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PAST>>>>::validate_vk(&vk),
        Err(VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<FUTURE>>>>::validate_vk(&vk),
        Err(VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PAST>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<FUTURE>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn reject_too_long_proof() {
    let proof = vec![0u8; MAX_PROOF_LENGTH as usize + 1];
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: vec![],
        certificates: vec![],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::InvalidProofData)
    )
}

#[test]
fn reject_too_long_vk() {
    let proof = vec![];
    let pubs = vec![];
    let vk = Vk::Intel {
        tcb_response: vec![0u8; MAX_VK_LENGTH as usize + 1],
        certificates: vec![0u8; MAX_VK_LENGTH as usize + 1],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::InvalidVerificationKey)
    )
}

#[test]
fn reject_invalid_pubs() {
    let proof = vec![];
    let pubs = vec![0u8; crate::MAX_PUBS_LENGTH as usize + 1];
    let vk = Vk::Intel {
        tcb_response: vec![],
        certificates: vec![],
    };

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<PRESENT>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::InvalidInput)
    )
}

// =========================================================================
// Nitro tests
// =========================================================================

#[test]
fn verify_valid_nitro_proof() {
    let proof = include_bytes!("resources/nitro/valid_attestation.bin").to_vec();
    let pubs = vec![];
    let vk = Vk::Nitro;

    assert!(Tee::<Mock<MockTime<ConstU64<NITRO_NOW>>>>::verify_proof(&vk, &proof, &pubs).is_ok());
}

#[test]
fn reject_invalid_nitro_proof() {
    let proof = include_bytes!("resources/nitro/invalid_attestation.bin").to_vec();
    let pubs = vec![];
    let vk = Vk::Nitro;

    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<NITRO_NOW>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn reject_nitro_with_expired_timestamp() {
    let proof = include_bytes!("resources/nitro/valid_attestation.bin").to_vec();
    let pubs = vec![];
    let vk = Vk::Nitro;

    // One year after: certificates will have expired
    const NITRO_FUTURE: u64 = NITRO_NOW + 365 * 24 * 3600;
    assert_eq!(
        Tee::<Mock<MockTime<ConstU64<NITRO_FUTURE>>>>::verify_proof(&vk, &proof, &pubs),
        Err(VerifyError::VerifyError)
    );
}

#[test]
fn validate_nitro_vk_always_fails() {
    let vk = Vk::Nitro;
    assert!(Tee::<Mock<MockTime<ConstU64<NITRO_NOW>>>>::validate_vk(&vk).is_err());
}
