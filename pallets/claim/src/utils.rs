use super::*;
use frame_benchmarking::account;
use sp_runtime::Saturating;

pub(crate) fn get_beneficiaries_map<T: Config>(
    n: u32,
) -> (BTreeMap<T::AccountId, BalanceOf<T>>, BalanceOf<T>) {
    let base_amount = BalanceOf::<T>::from(T::Currency::minimum_balance());
    let mut total_amount = BalanceOf::<T>::zero();
    let beneficiaries_map = (1..=n)
        .into_iter()
        .map(|i| {
            let amount = base_amount.saturating_add(i.into());
            total_amount = total_amount.saturating_add(amount);
            (account("", i, i), amount)
        })
        .collect::<BTreeMap<_, _>>();
    (beneficiaries_map, total_amount)
}
