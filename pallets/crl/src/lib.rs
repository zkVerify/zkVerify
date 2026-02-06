// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! A pallet for storing and managing Certificate Revocation Lists (CRLs) from multiple CAs.
//!
//! This pallet allows a manager to register Certificate Authorities (CAs) and update their
//! CRLs independently. Each CA has its own root certificate for CRL signature verification.
//!
//! The CRL is parsed and validated using the `tee-verifier` crate's `parse_crl` function,
//! which verifies the CRL signature against a certificate chain.

mod weight;

extern crate alloc;

use alloc::vec::Vec;

use frame_support::pallet_prelude::*;
use frame_support::{ensure, traits::UnixTime, weights::Weight};
pub use pallet::*;
pub use tee_verifier::{Crl, RevokedCertId};
pub use weight::WeightInfo;

/// Maximum size in bytes of the CRL PEM data.
pub const MAX_CRL_PEM_LENGTH: u32 = 65536;

/// Maximum size in bytes of the certificate chain PEM data.
pub const MAX_CERT_CHAIN_PEM_LENGTH: u32 = 16384;

/// Maximum size in bytes of the root certificate (DER encoded).
pub const MAX_ROOT_CERT_LENGTH: u32 = 2048;

/// Maximum number of revoked certificates that can be stored per CA.
pub const MAX_REVOKED_CERTS_PER_CA: u32 = 10000;

/// Error returned when a CA is not found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaNotFoundError;

/// Trait for accessing the CRL from other pallets.
pub trait CrlProvider {
    /// Returns the CRL for a specific CA by name.
    /// Returns an error if the CA is not found.
    fn get_crl(ca_name: &str) -> Result<Crl, CaNotFoundError>;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    use tee_verifier::parse_crl;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Manager origin allowed to register CAs and update CRLs.
        type ManagerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Maximum length of a CA name in bytes.
        #[pallet::constant]
        type MaxCaNameLength: Get<u32>;

        /// Provider for UnixTime
        type UnixTime: UnixTime;
    }

    /// Type alias for bounded CA name.
    pub type CaName<T> = BoundedVec<u8, <T as Config>::MaxCaNameLength>;

    /// Information about a registered Certificate Authority.
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct CaInfo {
        /// The root certificate (DER encoded) for CRL signature verification.
        pub root_cert: BoundedVec<u8, ConstU32<MAX_ROOT_CERT_LENGTH>>,
        /// The number of revoked certificates currently stored for this CA.
        pub revoked_count: u32,
        /// Timestamp (in secs) representing the issuing date for this CA's CRL. Incremented each time the CRL is updated.
        pub crl_version: u64,
    }

    /// Unique identifiers for revoked certificates.
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct RevokedInfo {
        issuer: BoundedVec<u8, ConstU32<256>>,
        serial: BoundedVec<u8, ConstU32<64>>,
    }

    /*
    impl From<RevokedInfo> for RevokedCertId {
        fn from(info: RevokedInfo) -> Self {
            RevokedCertId {
                issuer: info.issuer.into(),
                serial_number: info.serial.into(),
            }
        }
    }
    */

    /// Storage for registered CAs and their metadata.
    #[pallet::storage]
    pub type CertificateAuthorities<T: Config> =
        StorageMap<_, Blake2_128Concat, CaName<T>, CaInfo, OptionQuery>;

    /// Storage for revoked certificate issuers, keyed by (CA name, index).
    #[pallet::storage]
    pub type Revoked<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        CaName<T>,
        BoundedVec<RevokedInfo, ConstU32<MAX_REVOKED_CERTS_PER_CA>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new CA has been registered.
        CaRegistered {
            /// The name of the registered CA.
            name: CaName<T>,
        },
        /// A CA has been unregistered.
        CaUnregistered {
            /// The name of the unregistered CA.
            name: CaName<T>,
        },
        /// A CA's CRL has been updated.
        CrlUpdated {
            /// The name of the CA whose CRL was updated.
            ca_name: CaName<T>,
            /// The new version of the CRL.
            version: u64,
            /// The number of revoked certificates in the new CRL.
            revoked_count: u32,
        },
    }

    /// Errors for the CRL pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// The CA name exceeds the maximum allowed length.
        CaNameTooLong,
        /// The CA name is empty.
        CaNameEmpty,
        /// The root cert for the CA is empty.
        RootCertEmpty,
        /// The CA is already registered.
        CaAlreadyRegistered,
        /// The CA is not registered.
        CaNotFound,
        /// Maximum number of CAs reached.
        MaxCasReached,
        /// The root certificate exceeds the maximum allowed length.
        RootCertTooLarge,
        /// The CRL PEM data exceeds the maximum allowed length.
        CrlPemTooLarge,
        /// The certificate chain PEM data exceeds the maximum allowed length.
        CertChainPemTooLarge,
        /// Failed to parse or verify the CRL.
        CrlValidationError,
        /// Too many revoked certificates in the CRL.
        TooManyRevokedCerts,
        /// Issuer data exceeds maximum length.
        IssuerTooLarge,
        /// Serial number data exceeds maximum length.
        SerialNumberTooLarge,
        /// Crl exceeds maximum length.
        CrlTooLarge,
        /// The updated Crl is older than the one already registered
        NotNewerCrl,
    }

    impl<T: Config> Pallet<T> {
        /// Clear all revoked certificates for a CA.
        fn clear_revoked_certs_for_ca(ca_name: &CaName<T>) {
            Revoked::<T>::remove(ca_name);
        }

        /// Store a list of revoked certificates for a CA.
        fn store_revoked_certs_for_ca(ca_name: &CaName<T>, crl: &Crl) -> DispatchResult {
            let count = crl.len() as u32;
            if count > MAX_REVOKED_CERTS_PER_CA {
                return Err(Error::<T>::TooManyRevokedCerts.into());
            }

            let mut store: BoundedVec<RevokedInfo, ConstU32<MAX_REVOKED_CERTS_PER_CA>> =
                Default::default();
            for r in crl {
                store
                    .try_push(RevokedInfo {
                        issuer: r
                            .issuer
                            .clone()
                            .try_into()
                            .map_err(|_| Error::<T>::IssuerTooLarge)?,
                        serial: r
                            .serial_number
                            .clone()
                            .try_into()
                            .map_err(|_| Error::<T>::SerialNumberTooLarge)?,
                    })
                    .map_err(|_| Error::<T>::CrlTooLarge)?;
            }

            Revoked::<T>::insert(ca_name, store);
            Ok(())
        }

        /// Get the CRL for a specific CA.
        fn get_crl_for_ca(ca_name: &CaName<T>) -> Result<Crl, CaNotFoundError> {
            let crl = Revoked::<T>::get(ca_name).ok_or(CaNotFoundError)?;
            Ok(crl
                .into_iter()
                .map(|c| RevokedCertId {
                    issuer: c.issuer.into(),
                    serial_number: c.serial.into(),
                })
                .collect())
        }
    }

    impl<T: Config> CrlProvider for Pallet<T> {
        fn get_crl(ca_name: &str) -> Result<Crl, CaNotFoundError> {
            let bounded_name: CaName<T> = ca_name
                .as_bytes()
                .to_vec()
                .try_into()
                .map_err(|_| CaNotFoundError)?;
            Self::get_crl_for_ca(&bounded_name)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new Certificate Authority.
        ///
        /// # Arguments
        /// * `origin` - Must be the ManagerOrigin.
        /// * `name` - Unique name for the CA.
        /// * `root_cert` - The root certificate (DER encoded) for CRL signature verification.
        ///
        /// # Errors
        /// * `CaNameEmpty` - The CA name is empty.
        /// * `CaNameTooLong` - The CA name exceeds MaxCaNameLength.
        /// * `CaAlreadyRegistered` - A CA with this name already exists.
        /// * `MaxCasReached` - Maximum number of CAs reached.
        /// * `RootCertTooLarge` - The root certificate exceeds MAX_ROOT_CERT_LENGTH.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_ca())]
        pub fn register_ca(
            origin: OriginFor<T>,
            name: Vec<u8>,
            root_cert: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::ManagerOrigin::ensure_origin(origin)?;

            // Validate name
            if name.is_empty() {
                return Err(Error::<T>::CaNameEmpty.into());
            }
            if root_cert.is_empty() {
                return Err(Error::<T>::RootCertEmpty.into());
            }
            let bounded_name: CaName<T> = name.try_into().map_err(|_| Error::<T>::CaNameTooLong)?;

            // Check if CA already exists
            if CertificateAuthorities::<T>::contains_key(&bounded_name) {
                return Err(Error::<T>::CaAlreadyRegistered.into());
            }

            // Validate root cert
            let bounded_root_cert: BoundedVec<u8, ConstU32<MAX_ROOT_CERT_LENGTH>> = root_cert
                .try_into()
                .map_err(|_| Error::<T>::RootCertTooLarge)?;

            // Store CA info
            let ca_info = CaInfo {
                root_cert: bounded_root_cert,
                revoked_count: 0,
                crl_version: 0,
            };
            CertificateAuthorities::<T>::insert(&bounded_name, ca_info);

            log::info!("Registered CA: {:?}", bounded_name);
            Self::deposit_event(Event::CaRegistered { name: bounded_name });

            Ok(Pays::No.into())
        }

        /// Unregister a Certificate Authority and remove all its CRL data.
        ///
        /// # Arguments
        /// * `origin` - Must be the ManagerOrigin.
        /// * `name` - Name of the CA to unregister.
        ///
        /// # Errors
        /// * `CaNotFound` - No CA with this name exists.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unregister_ca())]
        pub fn unregister_ca(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            T::ManagerOrigin::ensure_origin(origin)?;

            let bounded_name: CaName<T> = name.try_into().map_err(|_| Error::<T>::CaNameTooLong)?;

            // Clear revoked certificates
            Self::clear_revoked_certs_for_ca(&bounded_name);

            // Remove CA info
            CertificateAuthorities::<T>::remove(&bounded_name);

            log::info!("Unregistered CA: {:?}", bounded_name);
            Self::deposit_event(Event::CaUnregistered { name: bounded_name });

            Ok(Pays::No.into())
        }

        /// Update the Certificate Revocation List for a specific CA.
        ///
        /// # Arguments
        /// * `origin` - Must be the ManagerOrigin.
        /// * `ca_name` - Name of the CA whose CRL to update.
        /// * `crl_pem` - PEM-encoded CRL data.
        /// * `cert_chain_pem` - PEM-encoded certificate chain for CRL signature verification.
        ///
        /// # Errors
        /// * `CaNotFound` - No CA with this name exists.
        /// * `CrlPemTooLarge` - The CRL PEM data exceeds MAX_CRL_PEM_LENGTH.
        /// * `CertChainPemTooLarge` - The certificate chain exceeds MAX_CERT_CHAIN_PEM_LENGTH.
        /// * `CrlValidationError` - Failed to parse or verify the CRL.
        /// * `TooManyRevokedCerts` - The CRL contains more than MAX_REVOKED_CERTS_PER_CA entries.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_crl(crl_pem.len() as u32))]
        pub fn update_crl(
            origin: OriginFor<T>,
            ca_name: Vec<u8>,
            crl_pem: Vec<u8>,
            cert_chain_pem: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            let bounded_name: CaName<T> =
                ca_name.try_into().map_err(|_| Error::<T>::CaNameTooLong)?;

            // Get CA info
            let mut ca_info =
                CertificateAuthorities::<T>::get(&bounded_name).ok_or(Error::<T>::CaNotFound)?;

            // Validate input sizes
            if crl_pem.len() > MAX_CRL_PEM_LENGTH as usize {
                return Err(Error::<T>::CrlPemTooLarge.into());
            }
            if cert_chain_pem.len() > MAX_CERT_CHAIN_PEM_LENGTH as usize {
                return Err(Error::<T>::CertChainPemTooLarge.into());
            }

            let (updated, parsed_crl) = parse_crl(
                &crl_pem,
                &cert_chain_pem,
                Some(ca_info.root_cert.as_slice()),
                <T as Config>::UnixTime::now().as_secs().try_into().unwrap(),
            )
            .map_err(|e| {
                log::error!("Failed to parse CRL for CA {:?}: {:?}", bounded_name, e);
                Error::<T>::CrlValidationError
            })?;

            if updated <= ca_info.crl_version {
                return Err(Error::<T>::NotNewerCrl.into());
            }

            let revoked_count = parsed_crl.len() as u32;

            // Store new CRL
            Self::store_revoked_certs_for_ca(&bounded_name, &parsed_crl)?;

            // Update CA info
            ca_info.revoked_count = revoked_count;
            ca_info.crl_version = updated;
            let version = ca_info.crl_version;
            CertificateAuthorities::<T>::insert(&bounded_name, ca_info);

            log::info!(
                "CRL updated for CA {:?} to version {} with {} revoked certificates",
                bounded_name,
                version,
                revoked_count
            );

            Self::deposit_event(Event::CrlUpdated {
                ca_name: bounded_name,
                version,
                revoked_count,
            });

            Ok(Pays::No.into())
        }
    }
}
