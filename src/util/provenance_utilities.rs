use crate::types::core::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{coin, Addr, Coin, CosmosMsg};
use provwasm_std::{
    grant_marker_access, revoke_marker_access, AccessGrant, Marker, MarkerAccess, Party, PartyType,
    ProvenanceMsg, Scope,
};

pub fn marker_has_permissions(
    marker: &Marker,
    address: &Addr,
    expected_permissions: &[MarkerAccess],
) -> bool {
    marker.permissions.iter().any(|permission| {
        &permission.address == address
            && expected_permissions.iter().all(|expected_permission| {
                permission
                    .permissions
                    .iter()
                    .any(|held_permission| held_permission == expected_permission)
            })
    })
}

pub fn marker_has_admin(marker: &Marker, admin_address: &Addr) -> bool {
    marker_has_permissions(marker, admin_address, &[MarkerAccess::Admin])
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

pub fn derive_marker_quote(
    marker: &Marker,
    quote_per_share: &[Coin],
) -> Result<Vec<Coin>, ContractError> {
    calculate_marker_quote(
        get_single_marker_coin_holding(marker)?.amount.u128(),
        quote_per_share,
    )
    .to_ok()
}

pub fn calculate_marker_quote(marker_share_count: u128, quote_per_share: &[Coin]) -> Vec<Coin> {
    quote_per_share
        .iter()
        .map(|c| coin(c.amount.u128() * marker_share_count, &c.denom))
        .to_owned()
        .collect::<Vec<Coin>>()
}

/// Verifies that the scope is properly owned.  At minimum, checks that the scope has only a singular owner.
/// If expected_owner is provided, the single owner with party type Owner must match this address.
/// If expected_value_owner is provided, the value_owner_address value must match this.
pub fn check_scope_owners(
    scope: &Scope,
    expected_owner: Option<&Addr>,
    expected_value_owner: Option<&Addr>,
) -> Result<(), ContractError> {
    let owners = scope
        .owners
        .iter()
        .filter(|owner| owner.role == PartyType::Owner)
        .collect::<Vec<&Party>>();
    // if more than one owner is specified, removing all of them can potentially cause data loss
    if owners.len() != 1 {
        return ContractError::InvalidScopeOwner {
            scope_address: scope.scope_id.clone(),
            explanation: format!(
                "the scope should only include a single owner, but found: {}",
                owners.len(),
            ),
        }
        .to_err();
    }
    if let Some(expected) = expected_owner {
        let owner = owners.first().unwrap();
        if &owner.address != expected {
            return ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id.clone(),
                explanation: format!(
                    "the scope owner was expected to be [{}], not [{}]",
                    expected, owner.address,
                ),
            }
            .to_err();
        }
    }
    if let Some(expected) = expected_value_owner {
        if &scope.value_owner_address != expected {
            return ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id.clone(),
                explanation: format!(
                    "the scope's value owner was expected to be [{}], not [{}]",
                    expected, scope.value_owner_address,
                ),
            }
            .to_err();
        }
    }
    ().to_ok()
}

/// Switches the scope's current owner value to the given owner value.
pub fn replace_scope_owner(mut scope: Scope, new_owner: Addr) -> Scope {
    // Empty out all owners from the scope now that it's verified safe to do
    scope.owners = scope
        .owners
        .into_iter()
        .filter(|owner| owner.role != PartyType::Owner)
        .collect();
    // Append the target value as the new sole owner
    scope.owners.push(Party {
        address: new_owner.clone(),
        role: PartyType::Owner,
    });
    // Swap over the value owner, ensuring that the target owner not only is listed as an owner,
    // but has full access control over the scope
    scope.value_owner_address = new_owner;
    scope
}

pub fn release_marker_from_contract<S: Into<String>>(
    marker_denom: S,
    contract_address: &Addr,
    permissions_to_grant: &[AccessGrant],
) -> Result<Vec<CosmosMsg<ProvenanceMsg>>, ContractError> {
    let marker_denom = marker_denom.into();
    let mut messages = vec![];
    // Restore all permissions that the marker had before it was transferred to the
    // contract.
    for permission in permissions_to_grant {
        messages.push(grant_marker_access(
            &marker_denom,
            permission.address.to_owned(),
            permission.permissions.to_owned(),
        )?);
    }
    // Remove the contract's ownership of the marker now that it is no longer available for
    // sale / trade.  This message HAS TO COME LAST because the contract will lose its permission
    // to restore the originally-revoked permissions otherwise.
    messages.push(revoke_marker_access(
        &marker_denom,
        contract_address.to_owned(),
    )?);
    messages.to_ok()
}

#[cfg(test)]
#[cfg(feature = "enable-test-utils")]
mod tests {
    use super::*;
    use crate::test::mock_marker::MockMarker;
    use cosmwasm_std::coins;

    #[test]
    fn test_derive_marker_quote() {
        let marker = MockMarker {
            denom: "testdenom".to_string(),
            coins: coins(100, "testdenom"),
            ..MockMarker::default()
        }
        .to_marker();
        let quote = derive_marker_quote(&marker, &coins(1, "nhash"))
            .expect("expected the conversion to succeed");
        assert_eq!(
            coins(100, "nhash"),
            quote,
            "expected 1 nhash per share mapping on 100 testdenom to equate to 100 nhash",
        );
    }
}
