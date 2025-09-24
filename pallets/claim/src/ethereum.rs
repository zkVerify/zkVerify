use super::*;
use alloc::vec::Vec;
use alloc::{format, string::String};
use codec::DecodeWithMemTracking;
use frame_support::pallet_prelude::*;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};

const ETH_PREFIX: &[u8] = b"\x19Ethereum Signed Message:\n";

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Default,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    Ord,
    PartialOrd,
)]
pub struct EthereumAddress(pub [u8; 20]);

impl Serialize for EthereumAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
        serializer.serialize_str(&format!("0x{}", hex))
    }
}

impl<'de> Deserialize<'de> for EthereumAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 40 {
            Err(serde::de::Error::custom(
                "Bad length of Ethereum address (should be 42 including '0x')",
            ))?;
        }
        let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
        let mut r = Self::default();
        r.0.copy_from_slice(&raw);
        Ok(r)
    }
}

impl AsRef<[u8]> for EthereumAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// A 65-byte Ethereum ECDSA Signature
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, TypeInfo, MaxEncodedLen, Eq)]
pub struct EthereumSignature(pub [u8; 65]);

impl core::default::Default for EthereumSignature {
    fn default() -> Self {
        EthereumSignature([0u8; 65])
    }
}

impl PartialEq for EthereumSignature {
    fn eq(&self, other: &Self) -> bool {
        &self.0[..] == &other.0[..]
    }
}

impl core::fmt::Debug for EthereumSignature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EthereumSignature({:?})", &self.0[..])
    }
}

impl Serialize for EthereumSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
        serializer.serialize_str(&format!("0x{}", hex))
    }
}

impl<'de> Deserialize<'de> for EthereumSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 130 {
            Err(serde::de::Error::custom(
                "Bad length of Ethereum signature (should be 132 including '0x')",
            ))?;
        }
        let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
        let mut r = Self::default();
        r.0.copy_from_slice(&raw);
        Ok(r)
    }
}

// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
pub(crate) fn ethereum_signable_message(msg: &[u8]) -> Vec<u8> {
    let mut l = msg.len();
    let mut rev = Vec::new();
    while l > 0 {
        rev.push(b'0' + (l % 10) as u8);
        l /= 10;
    }
    let mut v = ETH_PREFIX.to_vec();
    v.extend(rev.into_iter().rev());
    v.extend_from_slice(msg);
    v
}

// Attempts to recover the Ethereum address from a message signature signed by using
// the Ethereum RPC's `personal_sign` and `eth_sign`.
pub(crate) fn eth_recover(s: &EthereumSignature, msg: &[u8]) -> Option<EthereumAddress> {
    let msg = keccak_256(&ethereum_signable_message(msg));
    let mut res = EthereumAddress::default();
    res.0
        .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
    Some(res)
}
