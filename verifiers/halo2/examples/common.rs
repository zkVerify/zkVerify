use frame_support::traits::IsType;
use halo2_proofs::{
    halo2curves::bn256::{self, Bn256}, plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, Circuit}, poly::{commitment::Params, kzg::{commitment::{KZGCommitmentScheme, ParamsKZG}, multiopen::{ProverSHPLONK, VerifierSHPLONK}, strategy::SingleStrategy}}, transcript::{Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer}
};
use hp_verifiers::Verifier;
use pallet_halo2_verifier::Halo2;
use rand_chacha::ChaCha20Rng;
use rand::{thread_rng, SeedableRng};


pub fn test_verifier<ConcreteCircuit: Circuit<bn256::Fr>>(
    k: u32,
    circuit: &ConcreteCircuit,
    pi: Option<Vec<bn256::Fr>>,
    expected: bool,
) {
    let params = gen_srs(k);

    let vk = keygen_vk(&params, circuit).unwrap();
    let pk = keygen_pk(&params, vk.clone(), circuit).unwrap();
    let vk: pallet_halo2_verifier::Vk = vk.into_ref().try_into().unwrap();

    let proof = {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        let instance = if let Some(ref pi) = pi {
            vec![&pi[..]]
        } else {
            vec![]
        };

        create_proof::<KZGCommitmentScheme<bn256::Bn256>, ProverSHPLONK<bn256::Bn256>, _, _, _, _>(
            &params,
            &pk,
            std::slice::from_ref(circuit),
            &[&instance[..]],
            thread_rng(),
            &mut transcript,
        )
        .expect("proof generation should not fail");

        transcript.finalize()
    };

    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

    {
        let instance = if let Some(ref pi) = pi {
            vec![&pi[..]]
        } else {
            vec![]
        };
        let strategy = SingleStrategy::new(&params);
        verify_proof::<KZGCommitmentScheme<bn256::Bn256>, VerifierSHPLONK<bn256::Bn256>, _, _, _>(
            &params,
            pk.get_vk(),
            strategy,
            &[&instance[..]],
            &mut transcript,
        )
        .unwrap_or_default();
    }

    let pubs = pi.map(|x| x.into_iter().map(|x| x.into()).collect::<Vec<_>>());
    let params = params.try_into().unwrap();

    assert_eq!(
        Halo2::verify_proof(&(vk, params), &proof, &pubs).is_ok(),
        expected
    );
}


pub fn gen_srs(k: u32) -> ParamsKZG<Bn256> {
    let dir = "./params".to_string();
    let path = format!("{dir}/kzg_bn254_{k}.srs");
    match std::fs::read(path.as_str()) {
        Ok(mut b) => {
            println!("read params from {path}");
            ParamsKZG::<Bn256>::read(&mut b.as_slice()).unwrap()
        }
        Err(_) => {
            println!("creating params for {k}");
            std::fs::create_dir_all(dir).unwrap();
            let params = ParamsKZG::<Bn256>::setup(k, ChaCha20Rng::from_seed(Default::default()));
            let mut bytes = vec![];
            params.write(&mut bytes).unwrap();
            std::fs::write(path.as_str(), bytes).unwrap();
            params
        }
    }
}
