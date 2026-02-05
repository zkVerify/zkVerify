use crate::xcm_config::{RootLocation, TokenLocation};
use rstest::{fixture, rstest};
use xcm::{
    v5::{AssetId, Assets, Fungibility, Junction, WeightLimit, Xcm},
    IntoVersion, VersionedAssetId, VersionedLocation, VersionedXcm,
};
use xcm_runtime_apis::{
    conversions::runtime_decl_for_location_to_account_api::LocationToAccountApiV1,
    fees::Error as XcmPaymentApiError,
};

use super::*;

#[fixture]
fn xcm_program() -> VersionedXcm<()> {
    let asset = AssetId(TokenLocation::get()).into_asset(Fungibility::Fungible(currency::VFY));
    let xcm = Xcm::builder()
        .withdraw_asset(Assets::from(asset.clone()))
        .clear_origin()
        .buy_execution(asset, WeightLimit::Unlimited)
        .trap(1u64)
        .build();
    VersionedXcm::V5(xcm)
}

mod query_acceptable_payment_assets {
    use super::*;
    use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2;

    #[rstest]
    #[case::v3(3)]
    #[case::v4(4)]
    #[case::v5(5)]
    fn works_with_supported_version(#[case] version: xcm::Version) {
        test().execute_with(|| {
            let got = Runtime::query_acceptable_payment_assets(version).unwrap();
            let expected = vec![
                VersionedAssetId::V5(xcm::latest::AssetId(TokenLocation::get()))
                    .into_version(version)
                    .unwrap(),
            ];
            assert_eq!(got, expected);
        })
    }

    #[test]
    fn returns_empty_vector_with_unsupported_version() {
        test().execute_with(|| {
            assert_eq!(Runtime::query_acceptable_payment_assets(2).unwrap(), vec![])
        })
    }
}

mod query_weight_to_asset_fee {
    use super::*;
    use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2;

    #[rstest]
    #[case::v3(3)]
    #[case::v4(4)]
    #[case::v5(5)]
    fn returns_nonzero_for_nonzero_weight(#[case] version: xcm::Version) {
        test().execute_with(|| {
            let weight = Weight::from_parts(1_000_000, 1_000);
            let asset = VersionedAssetId::V5(xcm::latest::AssetId(TokenLocation::get()))
                .into_version(version)
                .unwrap();
            assert_ne!(
                Runtime::query_weight_to_asset_fee(weight, asset).unwrap(),
                0
            )
        })
    }

    #[rstest]
    #[case::v3(3)]
    #[case::v4(4)]
    #[case::v5(5)]
    fn returns_zero_for_zero_weight(#[case] version: xcm::Version) {
        test().execute_with(|| {
            let weight = Weight::zero();
            let asset = VersionedAssetId::V5(xcm::latest::AssetId(TokenLocation::get()))
                .into_version(version)
                .unwrap();
            assert_eq!(
                Runtime::query_weight_to_asset_fee(weight, asset).unwrap(),
                0
            )
        })
    }

    #[test]
    fn returns_error_for_unsupported_asset() {
        test().execute_with(|| {
            let weight = Weight::from_parts(1_000_000, 1_000);
            let location = RootLocation::get()
                .pushed_with_interior(Junction::Parachain(1))
                .unwrap();
            let unsuitable_asset = VersionedAssetId::V5(AssetId(location));
            assert_eq!(
                Runtime::query_weight_to_asset_fee(weight, unsuitable_asset).unwrap_err(),
                XcmPaymentApiError::AssetNotFound
            )
        })
    }
}

mod query_xcm_weight {
    use super::*;
    use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2;

    #[rstest]
    fn returns_nonzero_weight_for_nonempty_program(xcm_program: VersionedXcm<()>) {
        test().execute_with(|| {
            let weight = Runtime::query_xcm_weight(xcm_program).unwrap();
            assert_ne!(weight, Weight::zero())
        })
    }

    #[test]
    fn returns_zero_weight_for_empty_program() {
        test().execute_with(|| {
            let xcm_program = VersionedXcm::V5(Xcm::new());
            let weight = Runtime::query_xcm_weight(xcm_program).unwrap();
            assert_eq!(weight, Weight::zero())
        })
    }
}

mod query_delivery_fees {
    use super::*;
    use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV2;

    #[rstest]
    fn is_routed(xcm_program: VersionedXcm<()>) {
        test().execute_with(|| {
            let asset_id = VersionedAssetId::V5(AssetId(TokenLocation::get()));
            assert_eq!(
                Runtime::query_delivery_fees(
                    VersionedLocation::V5(RootLocation::get()),
                    xcm_program,
                    asset_id,
                )
                .unwrap_err(),
                XcmPaymentApiError::Unroutable
            );
        })
    }
}

mod convert_location {
    use super::*;

    #[fixture]
    fn alice_account_id_32() -> [u8; 32] {
        hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
    }

    #[rstest]
    fn leaves_parachain_account_id_32_unmodified(alice_account_id_32: [u8; 32]) {
        test().execute_with(|| {
            let location = RootLocation::get()
                .pushed_with_interior(Junction::AccountId32 {
                    network: None,
                    id: alice_account_id_32,
                })
                .unwrap()
                .into_versioned();

            assert_eq!(
                Runtime::convert_location(location).unwrap(),
                alice_account_id_32.into()
            );
        })
    }
}
