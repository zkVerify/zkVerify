use frame_support::traits::ConstU32;

struct MockConfig;

impl crate::Config for MockConfig {
    type MaxProofSize = ConstU32<1000000>;
    type MaxPubsSize = ConstU32<1000000>;
    type MaxVkSize = ConstU32<1000000>;
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
struct TestData<T: crate::Config> {
    pub(crate) vk: crate::Vk<T>,
    pub(crate) proof: crate::Proof<T>,
    pub(crate) pubs: crate::Pubs,
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn get_valid_test_data<T: crate::Config>() -> TestData<T> {
    TestData {
        vk: crate::VkWithConfig::from_default_with_bytes(
            include_bytes!("resources/vk.bin").to_vec(),
        ),
        proof: crate::Proof::from_default_with_bytes(
            include_bytes!("resources/proof.bin").to_vec(),
        ),
        pubs: include_bytes!("resources/pubs.bin").to_vec(),
    }
}
