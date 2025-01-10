use halo2_proofs::{
    halo2curves::bn256::Bn256, poly::{commitment::Params, kzg::commitment::ParamsKZG}
};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

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
