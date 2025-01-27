#[allow(unused_imports)]
use super::*;
use frame_support::traits::ConstU32;

struct MockConfig;

impl crate::Config for MockConfig {
    type MaxProofSize = ConstU32<1000000>;
    type MaxPubsSize = ConstU32<1000000>;
    type MaxVkSize = ConstU32<1000000>;
}

#[allow(dead_code)]
struct TestData<T: Config> {
    pub(crate) vk: Vk<T>,
    proof: Proof,
    pubs: Pubs,
}

#[allow(dead_code)]
fn get_valid_test_data<T: Config>() -> TestData<T> {
    TestData {
        vk: VkWithConfig::from_default_with_bytes(
            include_bytes!("resources/vk.bin").to_vec(),
        ),
        proof: include_bytes!("resources/proof.bin").to_vec(),
        pubs: include_bytes!("resources/pubs.bin").to_vec(),
    }
}
