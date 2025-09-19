use crate::{Event, mock::*};
use frame_support::{assert_noop, assert_ok};

/// End-to-end test demonstrating Stwo backend integration
#[test]
fn stwo_backend_integration_test() {
    new_test_ext().execute_with(|| {
        // Step 1: Register a Stwo verification key
        let vk_id = 42;
        let stwo_vk = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        ]; // Even checksum
        
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id, stwo_vk));
        
        // Step 2: Submit a Stwo proof with public inputs using the VK directly
        let stwo_proof = vec![
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ]; // Even checksum
        
        let public_inputs = vec![
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
        ]; // Even checksum
        
        // Use the VK bytes directly (as expected by submit_proof)
        let vk_bytes = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        ];
        
        assert_ok!(ZkStarky::submit_proof(
            RuntimeOrigin::signed(1), 
            vk_bytes, 
            stwo_proof, 
            public_inputs
        ));
        
        // Step 3: Verify the proof was processed correctly
        let events = System::events();
        assert_eq!(events.len(), 2);
        
        // Check VK registration event
        match &events[0].event {
            RuntimeEvent::ZkStarky(Event::VkRegistered { id, owner }) => {
                assert_eq!(*id, vk_id);
                assert_eq!(*owner, 1);
            },
            _ => panic!("Expected VkRegistered event"),
        }
        
        // Check proof verification event
        match &events[1].event {
            RuntimeEvent::ZkStarky(Event::Verified { success }) => {
                assert_eq!(*success, true); // Should pass with even checksums
            },
            _ => panic!("Expected Verified event"),
        }
    });
}

/// Test demonstrating batch verification with Stwo backend
#[test]
fn stwo_batch_verification_test() {
    new_test_ext().execute_with(|| {
        // Register VK
        let vk_id = 100;
        let vk_data = vec![0x00, 0x02, 0x04, 0x06]; // Even checksum
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id, vk_data.clone()));
        
        // Submit batch of Stwo proofs
        let batch_proofs = vec![
            (
                vec![0x10, 0x12, 0x14, 0x16], // Even checksum
                vec![0x20, 0x22, 0x24, 0x26], // Even checksum
            ),
            (
                vec![0x18, 0x1a, 0x1c, 0x1e], // Even checksum
                vec![0x28, 0x2a, 0x2c, 0x2e], // Even checksum
            ),
            (
                vec![0x30, 0x32, 0x34, 0x36], // Even checksum
                vec![0x40, 0x42, 0x44, 0x46], // Even checksum
            ),
        ];
        
        assert_ok!(ZkStarky::submit_proofs_batch(
            RuntimeOrigin::signed(1), 
            vk_id, 
            batch_proofs
        ));
        
        // Verify all proofs were processed
        let events = System::events();
        assert_eq!(events.len(), 4); // VkRegistered + 3x Verified
        
        // All proofs should have passed verification
        for i in 1..4 {
            match &events[i].event {
                RuntimeEvent::ZkStarky(Event::Verified { success }) => {
                    assert_eq!(*success, true);
                },
                _ => panic!("Expected Verified event"),
            }
        }
    });
}

/// Test demonstrating failure cases with Stwo backend
#[test]
fn stwo_verification_failures_test() {
    new_test_ext().execute_with(|| {
        // Register VK
        let vk_id = 200;
        let vk_data = vec![0x00, 0x02, 0x04, 0x06]; // Even checksum
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id, vk_data.clone()));
        
        // Submit proof with odd checksum (should fail)
        let invalid_proof = vec![0x11, 0x13, 0x15, 0x17]; // Odd checksum
        let inputs = vec![0x20, 0x22, 0x24, 0x26]; // Even checksum
        
        assert_ok!(ZkStarky::submit_proof(
            RuntimeOrigin::signed(1), 
            vk_data, 
            invalid_proof, 
            inputs
        ));
        
        // Verify the proof failed
        let events = System::events();
        assert_eq!(events.len(), 2);
        
        match &events[1].event {
            RuntimeEvent::ZkStarky(Event::Verified { success }) => {
                assert_eq!(*success, false); // Should fail with odd checksum
            },
            _ => panic!("Expected Verified event"),
        }
    });
}

/// Test demonstrating VK registry functionality
#[test]
fn vk_registry_management_test() {
    new_test_ext().execute_with(|| {
        // Register multiple VKs
        let vk_ids = vec![1, 2, 3, 42, 100];
        let vk_data = vec![0x00, 0x02, 0x04, 0x06]; // Even checksum
        
        for vk_id in vk_ids.iter() {
            assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), *vk_id, vk_data.clone()));
        }
        
        // Verify all VKs are stored
        for vk_id in vk_ids.iter() {
            assert!(ZkStarky::vk_registry(*vk_id).is_some());
        }
        
        // Test submitting proof with different VK IDs using batch
        let batch_proofs = vec![
            (
                vec![0x10, 0x12, 0x14, 0x16], // Even checksum
                vec![0x20, 0x22, 0x24, 0x26], // Even checksum
            ),
        ];
        
        // Test with first VK ID
        assert_ok!(ZkStarky::submit_proofs_batch(
            RuntimeOrigin::signed(1), 
            vk_ids[0], 
            batch_proofs.clone()
        ));
        
        // Verify proof was processed successfully
        let events = System::events();
        assert_eq!(events.len(), 6); // 5x VkRegistered + 1x Verified
        
        // Proof should have passed
        match &events[5].event {
            RuntimeEvent::ZkStarky(Event::Verified { success }) => {
                assert_eq!(*success, true);
            },
            _ => panic!("Expected Verified event"),
        }
    });
}

/// Test demonstrating edge cases and error handling
#[test]
fn stwo_edge_cases_test() {
    new_test_ext().execute_with(|| {
        // Test with minimal valid data
        let vk_id = 1;
        let minimal_vk = vec![0x00]; // Single byte, even checksum
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id, minimal_vk.clone()));
        
        let minimal_proof = vec![0x02]; // Single byte, even checksum
        let minimal_inputs = vec![0x04]; // Single byte, even checksum
        
        assert_ok!(ZkStarky::submit_proof(
            RuntimeOrigin::signed(1), 
            minimal_vk, 
            minimal_proof, 
            minimal_inputs
        ));
        
        // Test duplicate VK registration (should fail)
        let vk_id_dup = 2;
        let vk_data = vec![0x00, 0x02, 0x04, 0x06];
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id_dup, vk_data.clone()));
        
        // Try to register same ID again
        assert_noop!(
            ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id_dup, vk_data),
            crate::Error::<Test>::InvalidInput
        );
        
        // Verify both proofs were processed
        let events = System::events();
        assert_eq!(events.len(), 3); // 2x VkRegistered + 1x Verified
        
        // Proof should have passed
        match &events[2].event {
            RuntimeEvent::ZkStarky(Event::Verified { success }) => {
                assert_eq!(*success, true);
            },
            _ => panic!("Expected Verified event"),
        }
    });
}

/// Test demonstrating batch verification with mixed results
#[test]
fn stwo_mixed_batch_results_test() {
    new_test_ext().execute_with(|| {
        // Register VK
        let vk_id = 300;
        let vk_data = vec![0x00, 0x02, 0x04, 0x06]; // Even checksum
        assert_ok!(ZkStarky::register_vk(RuntimeOrigin::signed(1), vk_id, vk_data));
        
        // Submit batch with mixed results (some pass, some fail)
        let mixed_batch = vec![
            (
                vec![0x10, 0x12, 0x14, 0x16], // Even checksum - should pass
                vec![0x20, 0x22, 0x24, 0x26], // Even checksum - should pass
            ),
            (
                vec![0x11, 0x13, 0x15, 0x17], // Odd checksum - should fail
                vec![0x21, 0x23, 0x25, 0x27], // Odd checksum - should fail
            ),
            (
                vec![0x30, 0x32, 0x34, 0x36], // Even checksum - should pass
                vec![0x40, 0x42, 0x44, 0x46], // Even checksum - should pass
            ),
        ];
        
        assert_ok!(ZkStarky::submit_proofs_batch(
            RuntimeOrigin::signed(1), 
            vk_id, 
            mixed_batch
        ));
        
        // Verify all proofs were processed
        let events = System::events();
        assert_eq!(events.len(), 4); // VkRegistered + 3x Verified
        
        // Check individual results
        let verification_events: Vec<_> = events.iter()
            .filter_map(|e| match &e.event {
                RuntimeEvent::ZkStarky(Event::Verified { success }) => Some(*success),
                _ => None,
            })
            .collect();
        
        // Should have mixed results: pass, fail, pass
        assert_eq!(verification_events, vec![true, false, true]);
    });
}