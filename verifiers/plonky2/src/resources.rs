#[allow(unused_imports)]

use super::*;

#[allow(dead_code)]
struct TestData {
    pub(crate) vk: Vk,
    proof: Proof,
    pubs: Pubs,
}

#[allow(dead_code)]
fn get_valid_test_data() -> TestData {
    let vk = include_bytes!("resources/vk.bin");
    let vk = BoundedVec::try_from(vk.to_vec()).unwrap();

    TestData {
        vk,
        proof: include_bytes!("resources/proof.bin").to_vec(),
        pubs: include_bytes!("resources/pubs.bin").to_vec(),
    }
}
