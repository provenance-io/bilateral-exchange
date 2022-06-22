use crate::types::core::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{
    get_single_marker_coin_holding, marker_has_admin, marker_has_permissions,
};
use cosmwasm_std::Addr;
use provwasm_std::{Marker, MarkerAccess, MarkerStatus};

pub fn validate_marker_for_ask(
    marker: &Marker,
    original_owner_address: &Addr,
    contract_address: &Addr,
    expected_contract_permissions: &[MarkerAccess],
) -> Result<(), ContractError> {
    if !marker_has_admin(marker, original_owner_address) {
        return ContractError::InvalidMarker {
            message: format!(
                "expected sender [{}] to have admin privileges on marker [{}]",
                original_owner_address.as_str(),
                marker.address.as_str(),
            ),
        }
        .to_err();
    }
    if !marker_has_permissions(marker, contract_address, expected_contract_permissions) {
        return ContractError::InvalidMarker {
            message: format!(
                "expected this contract [{}] to have privileges {:?} on marker [{}]",
                contract_address.as_str(),
                expected_contract_permissions,
                marker.address.as_str(),
            ),
        }
        .to_err();
    }
    if marker.status != MarkerStatus::Active {
        return ContractError::InvalidMarker {
            message: format!(
                "expected marker [{}] to be active, but was in status [{:?}]",
                marker.address.as_str(),
                marker.status,
            ),
        }
        .to_err();
    }
    let marker_coin = get_single_marker_coin_holding(marker)?;
    if marker_coin.amount.u128() == 0 {
        return ContractError::InvalidMarker {
            message: format!(
                "expected marker [{}] to hold at least one of its supply of denom [{}], but it had [{}]",
                marker.address.as_str(),
                marker.denom,
                marker_coin.amount.u128(),
            )
        }.to_err();
    }
    ().to_ok()
}
