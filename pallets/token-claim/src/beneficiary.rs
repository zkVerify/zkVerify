use crate::{Config, EthereumAddress, EthereumSignature};
use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use core::cmp::Ordering;
use frame_support::{defensive, pallet_prelude::TypeInfo};
use serde::{self, Deserialize, Serialize};
use sp_runtime::traits::Verify;

pub(crate) const MSG_PREFIX: &[u8] = b"<Bytes>";
pub(crate) const MSG_SUFFIX: &[u8] = b"</Bytes>";
pub(crate) const ETH_MSG_SEPARATOR: &[u8] = b"\n";

#[derive(Clone, PartialEq, Encode, Decode, TypeInfo, Serialize, Deserialize, Eq, MaxEncodedLen)]
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

pub(crate) enum ClaimSignature<T: Config> {
    Substrate(T::Signature),
    Ethereum((EthereumSignature, T::AccountId)),
}

impl<T: Config> ClaimSignature<T> {
    pub(crate) fn verify(&self, claim_message: &[u8], beneficiary: &Beneficiary<T>) -> bool {
        use alloy_primitives::{Address, PrimitiveSignature};
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
                let dest_account = T::AccountIdBytesToSign::to_bytes_literal(&dest);
                let msg_with_dest =
                    [claim_message, ETH_MSG_SEPARATOR, dest_account.as_slice()].concat();

                // Check signature is successful and signer from signature is the same as beneficiary
                if let Ok(sig) = PrimitiveSignature::from_raw_array(&eth_sig.0) {
                    sig.recover_address_from_msg(msg_with_dest.as_slice())
                        .map_or_else(
                            |_| false,
                            |extracted_signer| {
                                extracted_signer == Address::from(eth_addr.as_fixed_bytes())
                            },
                        )
                } else {
                    false
                }
            }
            _ => {
                defensive!();
                false
            } // Other combinations not allowed
        }
    }
}
