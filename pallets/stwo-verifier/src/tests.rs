use crate::{mock::*, Error, Event as StwoEvent};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

#[test]
fn store_verifying_key_works() {
    new_test_ext().execute_with(|| {
        // Generate test data
        let key_id = vec![1; 32];
        let key = vec![2; 64];
        
        // Store key with root origin
        assert_ok!(StwoVerifier::store_verifying_key(
            RuntimeOrigin::root(),
            key_id.clone(),
            key.clone()
        ));
        
        // Verify event was emitted
        System::assert_has_event(RuntimeEvent::StwoVerifier(StwoEvent::VerifyingKeyStored { 
            key_id: key_id.try_into().unwrap(),
            hash: sp_core::H256(sp_io::hashing::keccak_256(&key)),
        }));
    });
}

#[test]
fn store_verifying_key_fails_non_root() {
    new_test_ext().execute_with(|| {
        let key_id = vec![1; 32];
        let key = vec![2; 64];
        
        assert_noop!(
            StwoVerifier::store_verifying_key(
                RuntimeOrigin::signed(1),
                key_id,
                key
            ),
            BadOrigin
        );
    });
}

#[test]
fn verify_proof_works() {
    new_test_ext().execute_with(|| {
        // First store a key
        let key_id = vec![1; 32];
        let key = vec![2; 64];
        assert_ok!(StwoVerifier::store_verifying_key(
            RuntimeOrigin::root(),
            key_id.clone(),
            key.clone()
        ));
        
        // Then verify a proof
        let proof = vec![3; 128];
        let public_inputs = vec![4; 32];
        
        assert_ok!(StwoVerifier::verify_proof(
            RuntimeOrigin::signed(1),
            key_id,
            proof.clone(),
            public_inputs.clone()
        ));

        // Get the last event and verify it's what we expect
        let events = System::events();
        assert!(matches!(
            events.last().unwrap().event,
            RuntimeEvent::StwoVerifier(StwoEvent::ProofVerified { who: 1, .. })
        ));
    });
}

#[test]
fn verify_proof_fails_with_invalid_key() {
    new_test_ext().execute_with(|| {
        let key_id = vec![1; 32];
        let proof = vec![3; 128];
        let public_inputs = vec![4; 32];
        
        assert_noop!(
            StwoVerifier::verify_proof(
                RuntimeOrigin::signed(1),
                key_id,
                proof,
                public_inputs
            ),
            Error::<Test>::VerifyingKeyNotFound
        );
    });
}
