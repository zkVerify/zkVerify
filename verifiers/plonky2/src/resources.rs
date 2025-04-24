use crate::vk::Plonky2Config;
use frame_support::traits::ConstU32;
use std::fs::read;

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
fn get_parameterized_test_data<T: crate::Config>(
    degree: usize,
    config: Plonky2Config,
    // compress: bool,
) -> TestData<T> {
    let h = match config {
        Plonky2Config::Poseidon => "poseidon",
        Plonky2Config::Keccak => "keccak",
    };
    // let c = match compress {
    //     true => "compressed",
    //     _ => "uncompressed",
    // };
    let base_path = format!("src/resources/degree_{degree}/uncompressed/{h}"); // TODO: FIX TO SUPPORT COMPRESSED PROOFS
    let vk_path = format!("{base_path}/vk.bin");
    let proof_path = format!("{base_path}/proof.bin");
    let pubs_path = format!("{base_path}/pubs.bin");

    TestData {
        vk: crate::VkWithConfig::new(
            config,
            read(&vk_path).expect(format!("File: {vk_path} not found!").as_str()),
        ),
        proof: crate::Proof::new(
            // false, // TODO: FIX TO SUPPORT COMPRESSED PROOFS
            read(&proof_path).expect(format!("File: {proof_path} not found!").as_str()),
        ),
        pubs: read(&pubs_path).expect(format!("File: {pubs_path} not found!").as_str()),
    }
}
