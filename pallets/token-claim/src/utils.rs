use super::*;
use crate::{beneficiary::Beneficiary, EthereumAddress, EthereumSignature};
use frame_benchmarking::account;
use sp_runtime::Saturating;

pub(crate) fn get_beneficiaries_map<T: Config>(
    n: u32,
) -> (BTreeMap<Beneficiary<T>, BalanceOf<T>>, BalanceOf<T>) {
    use crate::alloc::string::ToString;
    use secp_utils::{eth, secret_from_seed};

    let base_amount = BalanceOf::<T>::from(T::Currency::minimum_balance());
    let mut total_amount = BalanceOf::<T>::zero();
    let beneficiaries_map = (1..=n)
        .into_iter()
        .map(|i| {
            let amount = base_amount.saturating_add(i.into());
            total_amount = total_amount.saturating_add(amount);
            // Mix Substrate and Ethereum beneficiaries
            if i % 2 == 0 {
                (Beneficiary::<T>::Substrate(account("", i, i)), amount)
            } else {
                (
                    Beneficiary::<T>::Ethereum(eth(&secret_from_seed(&i.to_string().into_bytes()))),
                    amount,
                )
            }
        })
        .collect::<BTreeMap<_, _>>();
    (beneficiaries_map, total_amount)
}

pub(crate) mod secp_utils {
    use super::*;
    use libsecp256k1::{sign, Message, PublicKey, SecretKey};
    use sp_io::hashing::keccak_256;

    #[allow(dead_code)]
    pub(crate) fn parse_secret(secret_bytes: &[u8]) -> SecretKey {
        SecretKey::parse_slice(secret_bytes).unwrap()
    }

    pub(crate) fn secret_from_seed(seed: &[u8]) -> SecretKey {
        SecretKey::parse(&keccak_256(seed)).unwrap()
    }

    pub(crate) fn public(secret: &SecretKey) -> PublicKey {
        PublicKey::from_secret_key(secret)
    }

    pub(crate) fn eth(secret: &SecretKey) -> EthereumAddress {
        let mut res = EthereumAddress::default();
        res.0
            .copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
        res
    }

    pub(crate) fn sig(secret: &SecretKey, msg: &[u8]) -> EthereumSignature {
        let msg = alloy_primitives::eip191_hash_message(msg);
        let (sig, recovery_id) = sign(&Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        EthereumSignature::from_raw(r)
    }
}

mod test {

    #[test]
    fn consistency_check() {
        use super::secp_utils::*;
        use alloc::str::FromStr;
        use alloy_primitives::{address, Address, PrimitiveSignature};

        // Check consistency with EOA wallets (e.g. Metamask, Talisman, ...)

        // Check we derive same address
        let eth_address = address!("0xCFb405552868d9906DeDCAbe2F387a37E35e9610");
        let eth_sig = PrimitiveSignature::from_str("0x731dd59f3e8685917f883c9b70645a157704d877784a61593abb8635c063bfb02df081d2a99316b4710aab27b878ce496a882342312ba857b84823164c667be31c").unwrap();

        // Useless key
        let secret_bytes =
            hex_literal::hex!("7b2d076abcc1215ef9c5a37da07f50c92de1048b2e1e7a27b74c0ce154f9cbae");
        let secret = parse_secret(&secret_bytes[..]);
        let derived_address = Address::from(&eth(&secret).0);

        assert_eq!(derived_address, eth_address);

        // Check signature and verification works the same
        // The hardcoded signature was generated via Etherscan "Verified Signature" tool linked to a Metamask wallet
        let message = b"TestMessage42";
        let derived_signature =
            PrimitiveSignature::from_raw_array(&sig(&secret, &message[..]).0).unwrap();

        assert_eq!(
            eth_address,
            derived_signature
                .recover_address_from_msg(&message[..])
                .unwrap()
        );
        assert_eq!(
            derived_address,
            eth_sig.recover_address_from_msg(&message[..]).unwrap()
        );
    }
}
