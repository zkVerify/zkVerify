// Mock implementation for Assets that satisfies token_gateway requirements
// without the complexity of a full pallet implementation.

use codec::MaxEncodedLen;
use core::fmt::Debug;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::{
            fungibles::{self, Dust},
            DepositConsequence, Fortitude, Precision, Preservation, Provenance,
            WithdrawConsequence,
        },
        Currency,
    },
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Zero},
    DispatchError,
};
use sp_std::{marker::PhantomData, prelude::*};

/// Configuration trait for the mock Assets module
pub trait Config: frame_system::Config {
    /// The currency mechanism
    type Currency: Currency<Self::AccountId>;

    /// The asset ID type
    type AssetId: Member + Parameter + MaxEncodedLen + Copy + TypeInfo + Debug;

    /// The balance type
    type Balance: Member
        + Parameter
        + MaxEncodedLen
        + Copy
        + From<u128>
        + Into<u128>
        + AtLeast32BitUnsigned
        + Zero
        + Default
        + TypeInfo
        + Debug
        + frame_support::traits::tokens::Balance;
}

// Mock implementation of Assets pallet
pub struct Pallet<T>(PhantomData<T>);

// MOCK: Implementation of fungibles::Inspect for Assets (required by Mutate)
impl<T: Config> fungibles::Inspect<T::AccountId> for Pallet<T> {
    type AssetId = T::AssetId;
    type Balance = T::Balance;

    fn total_issuance(_id: Self::AssetId) -> Self::Balance {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn minimum_balance(_id: Self::AssetId) -> Self::Balance {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn balance(_id: Self::AssetId, _who: &T::AccountId) -> Self::Balance {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn total_balance(_id: Self::AssetId, _who: &T::AccountId) -> Self::Balance {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn reducible_balance(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _keep_alive: Preservation,
        _force: Fortitude,
    ) -> Self::Balance {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn can_deposit(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
        _mint: Provenance,
    ) -> DepositConsequence {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn can_withdraw(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn asset_exists(_id: Self::AssetId) -> bool {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }
}

// MOCK: Implementation of fungibles::Unbalanced for Assets (required by Mutate)
impl<T: Config> fungibles::Unbalanced<T::AccountId> for Pallet<T> {
    fn set_total_issuance(_id: Self::AssetId, _amount: Self::Balance) {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn decrease_balance(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
        _precision: Precision,
        _preservation: Preservation,
        _force: Fortitude,
    ) -> Result<Self::Balance, DispatchError> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn increase_balance(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
        _precision: Precision,
    ) -> Result<Self::Balance, DispatchError> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn write_balance(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _balance: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn handle_dust(_dust: Dust<T::AccountId, Self>) {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }
}

// MOCK: Implementation of fungibles::Mutate for Assets
impl<T: Config> fungibles::Mutate<T::AccountId> for Pallet<T> {
    fn mint_into(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn burn_from(
        _id: Self::AssetId,
        _who: &T::AccountId,
        _amount: Self::Balance,
        _keep_alive: Preservation,
        _precision: Precision,
        _force: Fortitude,
    ) -> Result<Self::Balance, DispatchError> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }
}

// MOCK: Implementation of fungibles::metadata::Inspect for Assets (required by metadata::Mutate)
impl<T: Config> fungibles::metadata::Inspect<T::AccountId> for Pallet<T> {
    fn name(_id: Self::AssetId) -> Vec<u8> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn symbol(_id: Self::AssetId) -> Vec<u8> {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }

    fn decimals(_id: Self::AssetId) -> u8 {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }
}

// MOCK: Implementation of fungibles::metadata::Mutate for Assets
impl<T: Config> fungibles::metadata::Mutate<T::AccountId> for Pallet<T> {
    fn set(
        _id: Self::AssetId,
        _owner: &T::AccountId,
        _name: Vec<u8>,
        _symbol: Vec<u8>,
        _decimals: u8,
    ) -> DispatchResult {
        unimplemented!("Mock Assets pallet is not meant to be used")
    }
}
