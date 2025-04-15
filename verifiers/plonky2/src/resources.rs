use frame_support::traits::ConstU32;

struct MockConfig;

impl crate::Config for MockConfig {
    type MaxProofSize = ConstU32<1000000>;
    type MaxPubsSize = ConstU32<1000000>;
    type MaxVkSize = ConstU32<1000000>;
    type WeightInfo = ();
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

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn get_valid_compressed_poseidon_test_data<T: crate::Config>() -> TestData<T> {
    TestData {
        vk: crate::VkWithConfig::new(
            crate::vk::Plonky2Config::Poseidon,
            include_bytes!("resources/degree_19/compressed/poseidon/vk.bin").to_vec(),
        ),
        proof: crate::Proof::new(
            true,
            include_bytes!("resources/degree_19/compressed/poseidon/proof.bin").to_vec(),
        ),
        pubs: include_bytes!("resources/degree_19/compressed/poseidon/pubs.bin").to_vec(),
    }
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn get_valid_uncompressed_poseidon_test_data<T: crate::Config>() -> TestData<T> {
    TestData {
        vk: crate::VkWithConfig::new(
            crate::vk::Plonky2Config::Poseidon,
            include_bytes!("resources/degree_19/uncompressed/poseidon/vk.bin").to_vec(),
        ),
        proof: crate::Proof::new(
            false,
            include_bytes!("resources/degree_19/uncompressed/poseidon/proof.bin").to_vec(),
        ),
        pubs: include_bytes!("resources/degree_19/uncompressed/poseidon/pubs.bin").to_vec(),
    }
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn get_valid_compressed_keccak_test_data<T: crate::Config>() -> TestData<T> {
    TestData {
        vk: crate::VkWithConfig::new(
            crate::vk::Plonky2Config::Keccak,
            include_bytes!("resources/degree_19/compressed/keccak/vk.bin").to_vec(),
        ),
        proof: crate::Proof::new(
            true,
            include_bytes!("resources/degree_19/compressed/keccak/proof.bin").to_vec(),
        ),
        pubs: include_bytes!("resources/degree_19/compressed/keccak/pubs.bin").to_vec(),
    }
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn get_valid_uncompressed_keccak_test_data<T: crate::Config>() -> TestData<T> {
    TestData {
        vk: crate::VkWithConfig::new(
            crate::vk::Plonky2Config::Keccak,
            include_bytes!("resources/degree_19/uncompressed/keccak/vk.bin").to_vec(),
        ),
        proof: crate::Proof::new(
            false,
            include_bytes!("resources/degree_19/uncompressed/keccak/proof.bin").to_vec(),
        ),
        pubs: include_bytes!("resources/degree_19/uncompressed/keccak/pubs.bin").to_vec(),
    }
}
