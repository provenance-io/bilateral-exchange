use crate::storage::contract_info::{get_contract_info, ContractInfo, CONTRACT_VERSION};
use crate::types::core::error::ContractError;
use crate::types::migrate::migration_options::MigrationOptions;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{attr, to_binary, DepsMut, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};
use semver::Version;

pub fn migrate_contract(
    deps: DepsMut<ProvenanceQuery>,
    options: Option<MigrationOptions>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    let mut contract_info = get_contract_info(deps.storage)?;
    check_valid_migration_versioning(&contract_info)?;
    contract_info.contract_version = CONTRACT_VERSION.to_string();
    let mut additional_attributes = vec![];
    if let Some(options) = options {
        if let Some(new_admin_address) = options.new_admin_address {
            contract_info.admin = deps.api.addr_validate(&new_admin_address)?;
            additional_attributes.push(attr("new_admin_address", &new_admin_address));
        }
    }
    Response::new()
        .add_attribute("action", "migrate_contract")
        .add_attribute("new_version", CONTRACT_VERSION)
        .add_attributes(additional_attributes)
        .set_data(to_binary(&contract_info)?)
        .to_ok()
}

pub fn check_valid_migration_versioning(contract_info: &ContractInfo) -> Result<(), ContractError> {
    let existing_contract_version = contract_info.contract_version.parse::<Version>()?;
    let new_contract_version = CONTRACT_VERSION.parse::<Version>()?;
    if existing_contract_version > new_contract_version {
        return ContractError::invalid_migration(format!(
            "current contract version [{}] is greater than the migration target version [{}]",
            &contract_info.contract_version, CONTRACT_VERSION,
        ))
        .to_err();
    }
    ().to_ok()
}
