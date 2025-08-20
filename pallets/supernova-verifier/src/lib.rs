pub fn verify_proof(
    vk_b64: &[u8],
    proof_b64: &[u8],
    num_steps: u32,
) -> Result<bool, ()> {
    use nova_snark::{
        nova::{PublicParams, RecursiveSNARK},
        provider::{PallasEngine, VestaEngine},
        traits::Engine,
    };
    use ark_std::vec::Vec;

    // engine type aliases
    type E1 = PallasEngine;
    type E2 = VestaEngine;
    type F1 = <E1 as Engine>::Scalar;

    // Deserialize PublicParams from base64 (vk_b64)
    let vk_str = core::str::from_utf8(vk_b64).map_err(|_| ())?;
    let pp: PublicParams<E1, E2, crate::verifier::FibStep> =
        crate::verifier::decode_b64(vk_str).map_err(|_| ())?;

    // Deserialize RecursiveSNARK from base64 (proof_b64)
    let proof_str = core::str::from_utf8(proof_b64).map_err(|_| ())?;
    let rs: RecursiveSNARK<E1, E2, crate::verifier::FibStep> =
        crate::verifier::decode_b64(proof_str).map_err(|_| ())?;

    // z0 = [1,1] (same as in CLI)
    let z0 = Vec::<F1>::from([F1::ONE, F1::ONE]);

    // verify
    match rs.verify(&pp, num_steps as usize, &z0) {
        Ok(_z_final) => Ok(true),
        Err(_) => Ok(false),
    }
}
