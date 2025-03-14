use super::*;
use bincode::{deserialize, serialize, serialized_size};
use frame_benchmarking::account;
use sp_runtime::Saturating;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const BENEFICIARIES_FILE: &str = "./src/resources/test_beneficiaries";

pub(crate) fn get_beneficiaries_map<T: Config>(
    n: u32,
) -> (BTreeMap<T::AccountId, BalanceOf<T>>, BalanceOf<T>) {
    let base_amount = BalanceOf::<T>::from(T::Currency::minimum_balance());
    let mut total_amount = BalanceOf::<T>::zero();
    let beneficiaries = (1..=n)
        .into_iter()
        .map(|i| {
            let amount = base_amount.saturating_add(i.into());
            total_amount = total_amount.saturating_add(amount);
            let account = account::<T::AccountId>("", i, i);
            (account, amount)
        })
        .collect::<BTreeMap<_, _>>();

    (beneficiaries, total_amount)
}

#[allow(dead_code)]
pub(crate) fn serialize_beneficiaries<T: Config>(
    beneficiaries: &BTreeMap<T::AccountId, BalanceOf<T>>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(Path::new(path))?;
    beneficiaries.iter().try_for_each(|(account, amount)| {
        let raw_account = serialize(&account)?;
        file.write(&raw_account)?;

        let raw_amount = serialize(&amount)?;
        file.write(&raw_amount)?;

        Ok(())
    })
}

pub(crate) fn deserialize_beneficiaries<T: Config>(
    n: u32,
    path: &str,
) -> Result<(BTreeMap<T::AccountId, BalanceOf<T>>, BalanceOf<T>), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut total_amount = BalanceOf::<T>::zero();
    let account_size = serialized_size(&account::<T::AccountId>("", 0, 0))?;
    let amount_size = serialized_size(&total_amount)?;
    Ok((
        (0..n)
            .map(|_| {
                let mut raw_account = vec![0u8; account_size as usize];
                file.read(&mut raw_account)?;
                let account = deserialize::<T::AccountId>(&raw_account)?;

                let mut raw_amount = vec![0u8; amount_size as usize];
                file.read(&mut raw_amount)?;
                let amount = deserialize::<BalanceOf<T>>(&raw_amount)?;
                total_amount += amount;

                Ok((account, amount))
            })
            .collect::<Result<BTreeMap<_, _>, Box<dyn std::error::Error>>>()?,
        total_amount,
    ))
}

#[test]
fn test_serialize_deserialize_beneficiaries() {
    let path = "./temp_test";
    let max_beneficiaries = 100;

    // Test complete serialize/deserialize
    let expected = get_beneficiaries_map::<crate::mock::Test>(max_beneficiaries);
    serialize_beneficiaries::<crate::mock::Test>(&expected.0, path).unwrap();

    let actual = deserialize_beneficiaries::<crate::mock::Test>(max_beneficiaries, path).unwrap();
    assert_eq!(actual, expected);

    // Test partial serialize/deserialize
    for n in 1..=max_beneficiaries - 1 {
        let (actual, _) = deserialize_beneficiaries::<crate::mock::Test>(n, path).unwrap();

        assert_eq!(
            actual.iter().collect::<Vec<_>>(),
            expected.0.iter().take(n as usize).collect::<Vec<_>>()
        );
    }

    std::fs::remove_file(path).unwrap()
}
