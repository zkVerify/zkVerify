use crate::{Config, EthereumAddress, EthereumSignature};
use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::cmp::Ordering;
use frame_support::{defensive, pallet_prelude::TypeInfo};
use serde::{self, Deserialize, Serialize};
use sp_runtime::traits::Verify;

pub const MSG_PREFIX: &[u8] = b"<Bytes>";
pub const MSG_SUFFIX: &[u8] = b"</Bytes>";

#[derive(
    Clone,
    PartialEq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    Serialize,
    Deserialize,
    Eq,
    MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
/// A beneficiary with a Substrate-based address or an Ethereum address
pub enum Beneficiary<T: Config> {
    /// A beneficiary with a Substrate address
    Substrate(T::AccountId),
    /// A beneficiary with an Ethereum address
    Ethereum(EthereumAddress),
}

impl<T: Config> PartialOrd for Beneficiary<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Config> Ord for Beneficiary<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Beneficiary::*;

        match (self, other) {
            (Substrate(_), Ethereum(_)) => Ordering::Less,
            (Ethereum(_), Substrate(_)) => Ordering::Greater,
            (Substrate(sa), Substrate(sb)) => sa.cmp(sb),
            (Ethereum(ea), Ethereum(eb)) => ea.cmp(eb),
        }
    }
}

impl<T: Config> core::fmt::Debug for Beneficiary<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Beneficiary::*;

        match self {
            Substrate(s) => s.fmt(f),
            Ethereum(e) => e.fmt(f),
        }
    }
}

use sp_core::{
    crypto::{Ss58AddressFormat, Ss58Codec},
    Get,
};

/// Means of converting an account id to bytes string literal
pub trait AccountIdToBytesLiteral<T: frame_system::Config> {
    /// The id type of an account
    type AccountId;

    /// Convert to bytes string literal
    fn to_bytes_literal(account: &Self::AccountId) -> Vec<u8>;
}

/// Convert an AccountId32 to the Ss58 bytes string representation
pub struct AccountId32ToSs58BytesToSign;

impl<T: frame_system::Config> AccountIdToBytesLiteral<T> for AccountId32ToSs58BytesToSign {
    type AccountId = sp_runtime::AccountId32;
    fn to_bytes_literal(account: &Self::AccountId) -> Vec<u8> {
        let version = Ss58AddressFormat::custom(T::SS58Prefix::get());
        account.to_ss58check_with_version(version).into_bytes()
    }
}

pub enum ClaimSignature<T: Config> {
    Substrate(T::Signature),
    Ethereum((EthereumSignature, T::AccountId)),
}

pub fn eip191_hash_message(message: &[u8]) -> [u8; 32] {
    use crate::alloc::string::ToString;

    let mut eth_message = b"\x19Ethereum Signed Message:\n".to_vec();
    eth_message.extend_from_slice(message.len().to_string().as_bytes());
    eth_message.extend_from_slice(message);
    sp_core::keccak_256(&eth_message)
}

pub fn eth_recover(
    msg_to_sign: &[u8; 32],
    eth_sig: &EthereumSignature,
) -> Result<EthereumAddress, ()> {
    use frame_support::crypto::ecdsa::ECDSAExt;

    let pk_raw = sp_io::crypto::secp256k1_ecdsa_recover(&eth_sig.0, msg_to_sign).map_err(|_| ())?;
    let pk = sp_core::ecdsa::Public::from_full(pk_raw.as_slice()).map_err(|_| ())?;
    Ok(EthereumAddress::from(pk.to_eth_address()?))
}

impl<T: Config> ClaimSignature<T> {
    pub fn verify(&self, claim_message: &[u8], beneficiary: &Beneficiary<T>) -> bool {
        match (beneficiary, self) {
            // Beneficiary with Substrate address
            (Beneficiary::<T>::Substrate(sub_addr), ClaimSignature::<T>::Substrate(sub_sig)) => {
                // When signing with substrate tools/wallets (e.g. PolkadotJS Sign&Verify, Talisman, etc.)
                // the message is wrapped in this way "<Bytes>MSG</Bytes>"", w.r.t. to signing with
                // the keyring directly. Thus we need to take into consideration both cases here
                let prefixed_message = [MSG_PREFIX, claim_message, MSG_SUFFIX].concat();
                sub_sig.verify(prefixed_message.as_slice(), sub_addr)
                    || sub_sig.verify(claim_message, sub_addr)
            }

            // Beneficiary with Ethereum address
            (
                Beneficiary::<T>::Ethereum(eth_addr),
                ClaimSignature::<T>::Ethereum((eth_sig, dest)),
            ) => {
                // We need to append to the message the destination Substrate account, otherwise anyone could
                // intercept the transaction and change the destination
                let dest_account = T::AccountIdBytesToSign::to_bytes_literal(dest);
                let msg_with_dest = [
                    claim_message,
                    T::EthMsgSeparator::get(),
                    dest_account.as_slice(),
                ]
                .concat();

                // Check signature is successful and signer from signature is the same as beneficiary
                let msg_to_sign = eip191_hash_message(msg_with_dest.as_slice());
                eth_recover(&msg_to_sign, eth_sig)
                    .map_or_else(|_| false, |derived_address| &derived_address == eth_addr)
            }
            _ => {
                defensive!();
                false
            } // Other combinations not allowed
        }
    }
}
