// Copyright 2026, Horizen Labs, Inc.
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

use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::*;
use pallet::{CaName, CertificateAuthorities, Error, Event, Revoked};

// ---------------------------------------------------------------------------
// register_ca
// ---------------------------------------------------------------------------

#[test]
fn register_ca() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 0);
        assert!(ca_info.crl_versions.is_empty());

        System::assert_has_event(Event::CaRegistered { name: bounded_name }.into());
    })
}

#[test]
fn not_register_ca_empty_name() {
    test().execute_with(|| {
        assert_noop!(
            CrlPallet::register_ca(Origin::Root.into(), vec![], root_cert()),
            Error::<Test>::CaNameEmpty
        );
    })
}

#[test]
fn not_register_ca_empty_root_cert() {
    test().execute_with(|| {
        assert_noop!(
            CrlPallet::register_ca(Origin::Root.into(), CA_NAME.to_vec(), vec![]),
            Error::<Test>::RootCertEmpty
        );
    })
}

#[test]
fn not_register_ca_duplicate() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));
        assert_noop!(
            CrlPallet::register_ca(Origin::Root.into(), CA_NAME.to_vec(), root_cert()),
            Error::<Test>::CaAlreadyRegistered
        );
    })
}

#[test]
fn not_register_ca_wrong_origin() {
    test().execute_with(|| {
        assert_noop!(
            CrlPallet::register_ca(Origin::Signed(ALICE).into(), CA_NAME.to_vec(), root_cert()),
            sp_runtime::traits::BadOrigin
        );
    })
}

// ---------------------------------------------------------------------------
// unregister_ca
// ---------------------------------------------------------------------------

#[test]
fn unregister_ca() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));
        assert_ok!(CrlPallet::unregister_ca(
            Origin::Root.into(),
            CA_NAME.to_vec()
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        assert!(CertificateAuthorities::<Test>::get(&bounded_name).is_none());
        assert!(Revoked::<Test>::get(&bounded_name).is_none());

        System::assert_has_event(Event::CaUnregistered { name: bounded_name }.into());
    })
}

#[test]
fn unregister_ca_wrong_origin() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));
        assert_noop!(
            CrlPallet::unregister_ca(Origin::Signed(ALICE).into(), CA_NAME.to_vec()),
            sp_runtime::traits::BadOrigin
        );
    })
}

// ---------------------------------------------------------------------------
// update_crl — basic
// ---------------------------------------------------------------------------

#[test]
fn update_crl() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 3);
        assert_eq!(ca_info.crl_versions.len(), 1);

        let revoked = Revoked::<Test>::get(&bounded_name).unwrap();
        assert_eq!(revoked.len(), 3);
    })
}

#[test]
fn update_crl_ca_not_found() {
    test().execute_with(|| {
        assert_noop!(
            CrlPallet::update_crl(
                Origin::Signed(ALICE).into(),
                b"NonExistent".to_vec(),
                crl_inter1_v1(),
                chain1(),
            ),
            Error::<Test>::CaNotFound
        );
    })
}

#[test]
fn update_crl_not_newer_same_issuer() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        // First update succeeds
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));

        // Same CRL (same thisUpdate) should be rejected
        assert_noop!(
            CrlPallet::update_crl(
                Origin::Signed(ALICE).into(),
                CA_NAME.to_vec(),
                crl_inter1_v1(),
                chain1(),
            ),
            Error::<Test>::NotNewerCrl
        );
    })
}

// ---------------------------------------------------------------------------
// update_crl — per-issuer version tracking
// ---------------------------------------------------------------------------

#[test]
fn update_crl_two_issuers_independently() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        // Update from issuer 1 (3 revoked certs)
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 3);
        assert_eq!(ca_info.crl_versions.len(), 1);

        // Update from issuer 2 (2 revoked certs) — should merge, not replace
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));

        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 5); // 3 from issuer1 + 2 from issuer2
        assert_eq!(ca_info.crl_versions.len(), 2); // two distinct issuers tracked

        let revoked = Revoked::<Test>::get(&bounded_name).unwrap();
        assert_eq!(revoked.len(), 5);
    })
}

#[test]
fn update_crl_replaces_same_issuer_entries() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        // Update from issuer 1 v1 (3 revoked certs)
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));

        // Update from issuer 2 (2 revoked certs)
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        assert_eq!(
            CertificateAuthorities::<Test>::get(&bounded_name)
                .unwrap()
                .revoked_count,
            5
        );

        // Update issuer 1 with v2 (5 revoked certs) — should replace issuer1's 3 entries
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v2(),
            chain1(),
        ));

        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 7); // 5 from issuer1 v2 + 2 from issuer2
        assert_eq!(ca_info.crl_versions.len(), 2); // still two issuers

        let revoked = Revoked::<Test>::get(&bounded_name).unwrap();
        assert_eq!(revoked.len(), 7);
    })
}

#[test]
fn update_crl_not_newer_does_not_affect_other_issuer() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        // Update issuer 1 v2 (newer) first
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v2(),
            chain1(),
        ));

        // Update issuer 2 — should succeed even though issuer 2's thisUpdate may be
        // older than issuer 1's, since version tracking is per-issuer
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        let ca_info = CertificateAuthorities::<Test>::get(&bounded_name).unwrap();
        assert_eq!(ca_info.revoked_count, 7); // 5 + 2
        assert_eq!(ca_info.crl_versions.len(), 2);
    })
}

#[test]
fn update_crl_older_rejected_for_same_issuer_only() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));

        // Update issuer 1 with v2 (newer)
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v2(),
            chain1(),
        ));

        // Try issuer 1 v1 (older) — should be rejected
        assert_noop!(
            CrlPallet::update_crl(
                Origin::Signed(ALICE).into(),
                CA_NAME.to_vec(),
                crl_inter1_v1(),
                chain1(),
            ),
            Error::<Test>::NotNewerCrl
        );

        // Issuer 2 should still work
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));
    })
}

// ---------------------------------------------------------------------------
// unregister_ca clears multi-issuer data
// ---------------------------------------------------------------------------

#[test]
fn unregister_ca_clears_all_issuers() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));

        let bounded_name: CaName<Test> = CA_NAME.to_vec().try_into().unwrap();
        assert_eq!(Revoked::<Test>::get(&bounded_name).unwrap().len(), 5);

        assert_ok!(CrlPallet::unregister_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
        ));

        assert!(CertificateAuthorities::<Test>::get(&bounded_name).is_none());
        assert!(Revoked::<Test>::get(&bounded_name).is_none());
    })
}

// ---------------------------------------------------------------------------
// CrlProvider trait
// ---------------------------------------------------------------------------

#[test]
fn get_crl_returns_all_issuers() {
    test().execute_with(|| {
        assert_ok!(CrlPallet::register_ca(
            Origin::Root.into(),
            CA_NAME.to_vec(),
            root_cert(),
        ));
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter1_v1(),
            chain1(),
        ));
        assert_ok!(CrlPallet::update_crl(
            Origin::Signed(ALICE).into(),
            CA_NAME.to_vec(),
            crl_inter2(),
            chain2(),
        ));

        let ca_name_str = core::str::from_utf8(CA_NAME).unwrap();
        let crl =
            <CrlPallet as CrlProvider>::get_crl(ca_name_str).expect("CRL should be available");
        assert_eq!(crl.len(), 5); // 3 from issuer1 + 2 from issuer2
    })
}

#[test]
fn get_crl_not_found() {
    test().execute_with(|| {
        let result = <CrlPallet as CrlProvider>::get_crl("NonExistent");
        assert!(result.is_err());
    })
}
