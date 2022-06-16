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
    if marker
        .coins
        .iter()
        .filter(|coin| coin.denom == marker.denom)
        .count()
        != 1
    {
        return ContractError::InvalidMarker {
            message: format!(
                "expected marker [{}] to have a single coin entry for denom [{}], but it did not.  Holdings: {:?}",
                marker.address.as_str(),
                marker.denom,
                marker.coins,
            ),
        }
            .to_err();
    }
    marker
        .coins
        .iter()
        .find(|coin| coin.denom == marker.denom)
        .unwrap()
        .to_owned()
        .to_ok()
}

pub fn get_marker_quote(
    marker: &Marker,
    quote_per_share: &[Coin],
) -> Result<Vec<Coin>, ContractError> {
    let marker_share_count = get_single_marker_coin_holding(&marker)?.amount.u128();
    quote_per_share
        .iter()
        .map(|c| coin(c.amount.u128() * marker_share_count, &c.denom))
        .to_owned()
        .collect::<Vec<Coin>>()
        .to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::mock_marker::MockMarker;
    use cosmwasm_std::coins;

    #[test]
    fn test_get_marker_quote() {
        let marker = MockMarker {
            denom: "testdenom".to_string(),
            coins: coins(100, "testdenom"),
            ..MockMarker::default()
        }
        .to_marker();
        let quote = get_marker_quote(&marker, &coins(1, "nhash"))
            .expect("expected the conversion to succeed");
        assert_eq!(
            coins(100, "nhash"),
            quote,
            "expected 1 nhash per share mapping on 100 testdenom to equate to 100 nhash",
        );
    }
}
