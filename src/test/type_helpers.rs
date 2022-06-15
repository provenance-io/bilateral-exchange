use cosmwasm_std::{Decimal, Uint128};

pub fn decimal(value: u128) -> Decimal {
    Decimal::new(Uint128::new(value))
}
