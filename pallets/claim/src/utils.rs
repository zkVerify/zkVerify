use super::*;
use crate::beneficiary::Beneficiary;
use frame_benchmarking::account;
use sp_runtime::Saturating;

pub(crate) fn get_beneficiaries_map<T: Config>(
    n: u32,
) -> (BTreeMap<Beneficiary<T>, BalanceOf<T>>, BalanceOf<T>) {
    let base_amount = BalanceOf::<T>::from(T::Currency::minimum_balance());
    let mut total_amount = BalanceOf::<T>::zero();
    let beneficiaries_map = (1..=n)
        .into_iter()
        .map(|i| {
            let amount = base_amount.saturating_add(i.into());
            total_amount = total_amount.saturating_add(amount);
            (Beneficiary::<T>::Substrate(account("", i, i)), amount)
        })
        .collect::<BTreeMap<_, _>>();
    (beneficiaries_map, total_amount)
}

pub(crate) mod secp_utils {
    use crate::ethereum::*;
    use sp_io::hashing::keccak_256;

    pub(crate) fn secret_from_seed(seed: &[u8]) -> libsecp256k1::SecretKey {
        libsecp256k1::SecretKey::parse(&keccak_256(seed)).unwrap()
    }

    pub(crate) fn public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
        libsecp256k1::PublicKey::from_secret_key(secret)
    }

    pub(crate) fn eth(secret: &libsecp256k1::SecretKey) -> EthereumAddress {
        let mut res = EthereumAddress::default();
        res.0
            .copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
        res
    }

    pub(crate) fn sig(secret: &libsecp256k1::SecretKey, msg: &[u8]) -> EthereumSignature {
        let msg = keccak_256(ethereum_signable_message(msg).as_slice());
        let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        EthereumSignature(r)
    }
}

// mod test {
//     use super::secp_utils::*;
//     use crate::ethereum::{to_ascii_hex, EthereumAddress, EthereumSignature};
//     use codec::Encode;

//     fn parse_secret(secret_bytes: &[u8]) -> libsecp256k1::SecretKey {
//         libsecp256k1::SecretKey::parse_slice(secret_bytes).unwrap()
//     }
// 
//     #[test]
//     fn consistency_check() {
//         // Check we derive same address
//         let eth_address = EthereumAddress(hex_literal::hex!(
//             ""
//         ));

//         let secret_bytes =
//             hex_literal::hex!("");
//         let secret = parse_secret(&secret_bytes[..]);
//         let derived_address = eth(&secret);

//         assert_eq!(derived_address, eth_address);

//         // Check signature and verification works the same
//         // The hardcoded signature was generated via Etherscan "Verified Signature" tool linked to a Metamask wallet
//         let message = b"TestMessage42";
//         let derived_signature = sig(&secret, &message[..]);

//         assert_eq!(
//             eth_address,
//             crate::ethereum::eth_recover(&derived_signature, &message[..]).unwrap()
//         );
//         assert_eq!(
//             derived_address,
//             crate::ethereum::eth_recover(
//                 &EthereumSignature(hex_literal::hex!("")),
//                 &message[..]
//             ).unwrap()
//         );
//     }
// }
