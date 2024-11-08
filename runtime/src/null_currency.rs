use frame_support::traits::tokens::fungible::{Dust, Inspect, Mutate, Unbalanced};
use frame_support::traits::tokens::{
    DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence,
};
use sp_runtime::DispatchError;

pub struct NullCurrency;

impl<AccountId: Eq> Inspect<AccountId> for NullCurrency {
    type Balance = u128;

    fn total_issuance() -> Self::Balance {
        0
    }

    fn minimum_balance() -> Self::Balance {
        0
    }

    fn total_balance(_who: &AccountId) -> Self::Balance {
        0
    }

    fn balance(_who: &AccountId) -> Self::Balance {
        0
    }

    fn reducible_balance(
        _who: &AccountId,
        _preservation: Preservation,
        _force: Fortitude,
    ) -> Self::Balance {
        0
    }

    fn can_deposit(
        _who: &AccountId,
        _amount: Self::Balance,
        _provenance: Provenance,
    ) -> DepositConsequence {
        DepositConsequence::Success
    }

    fn can_withdraw(
        _who: &AccountId,
        _amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        WithdrawConsequence::Success
    }
}

impl<AccountId: Eq> Unbalanced<AccountId> for NullCurrency {
    fn handle_dust(_dust: Dust<AccountId, Self>) {}

    fn write_balance(
        _who: &AccountId,
        _amount: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError> {
        Ok(None)
    }

    fn set_total_issuance(_amount: Self::Balance) {}
}

impl<AccountId: Eq> Mutate<AccountId> for NullCurrency {
    fn done_mint_into(_who: &AccountId, _amount: Self::Balance) {}
    fn done_burn_from(_who: &AccountId, _amount: Self::Balance) {}
    fn done_shelve(_who: &AccountId, _amount: Self::Balance) {}
    fn done_restore(_who: &AccountId, _amount: Self::Balance) {}
    fn done_transfer(_source: &AccountId, _dest: &AccountId, _amount: Self::Balance) {}
}
