use frame_support::weights::Weight;
use hp_verifiers::WeightInfo;

pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T> WeightInfo<T> for SubstrateWeight<T> {
    fn store_verifying_key() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn verify_proof(_proof: &[u8], _public_inputs: &[u8]) -> Weight {
        Weight::from_parts(10_000, 0)
    }
}

pub struct MockWeightInfo;

impl<T> WeightInfo<T> for MockWeightInfo {
    fn store_verifying_key() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn verify_proof(_proof: &[u8], _public_inputs: &[u8]) -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
