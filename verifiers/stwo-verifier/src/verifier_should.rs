#![cfg(test)]

use super::*;

struct Mock;

impl Config for Mock {}

include!("resources.rs");

#[test]
fn verify_valid_proof() {
    assert!(Stwo::<Mock>::verify_proof(&VALID_VK, &VALID_PROOF, &VALID_PUBS).is_ok());
}
