use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{coin, Addr, Coin};
use provwasm_std::{Marker, MarkerAccess};

pub fn marker_has_admin(marker: &Marker, expected_admin: &Addr) -> bool {
    marker.permissions.iter().any(|permission| {
        &permission.address == expected_admin
            && permission
                .permissions
                .iter()
                .any(|grant| grant == &MarkerAccess::Admin)
    })
}

pub fn get_single_marker_coin_holding(marker: &Marker) -> Result<Coin, ContractError> {
    if marker.coins.len() != 1 {
        return ContractError::InvalidMarker {
            message: format!(
                "expected marker [{}] to have only a single coin entry, but had: {:?}",
                marker.address.as_str(),
                marker.coins,
            ),
        }
        .to_err();
    }
    let marker_coin = marker.coins.first().unwrap();
    if marker_coin.denom != marker.denom {
        return ContractError::InvalidMarker {
            message: format!(
                "expected marker [{}] to hold a single coin of type [{}] but it had coin: [{:?}]",
                marker.address.as_str(),
                marker.denom,
                marker_coin,
            ),
        }
        .to_err();
    }
    marker_coin.to_owned().to_ok()
}

pub fn get_marker_quote(marker: &Marker, quote_per_share: &[Coin]) -> Vec<Coin> {
    let marker_share_count = marker.total_supply.atomics().u128();
    quote_per_share
        .iter()
        .map(|c| coin(c.amount.u128() * marker_share_count, &c.denom))
        .to_owned()
        .collect()
}
