use cosmwasm_std::{HumanAddr, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

const NAMESPACE_CONTRACT_INFO: &str = "contract_info";
const CONTRACT_TYPE: &str = "figure:smart-contracts.bilateral-exchange";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONTRACT_INFO: Item<ContractInfo> = Item::new(NAMESPACE_CONTRACT_INFO);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    admin: HumanAddr,
    bind_name: String,
    contract_type: String,
    contract_name: String,
    contract_version: String,
}

impl ContractInfo {
    pub fn new<T: Into<String>>(admin: HumanAddr, name: T, bind_name: T) -> ContractInfo {
        ContractInfo {
            admin,
            bind_name: bind_name.into(),
            contract_type: CONTRACT_TYPE.into(),
            contract_name: name.into(),
            contract_version: CONTRACT_VERSION.into(),
        }
    }
}

pub fn set_contract_info<T: Into<String>>(
    store: &mut dyn Storage,
    admin: HumanAddr,
    bind_name: T,
    contract_name: T,
) -> Result<(), ContractError> {
    let info = ContractInfo {
        admin,
        bind_name: bind_name.into(),
        contract_name: contract_name.into(),
        contract_type: CONTRACT_TYPE.into(),
        contract_version: CONTRACT_VERSION.into(),
    };

    let result = CONTRACT_INFO.save(store, &info);
    result.map_err(ContractError::Std)
}

pub fn get_contract_info(store: &dyn Storage) -> StdResult<ContractInfo> {
    CONTRACT_INFO.load(store)
}

#[cfg(test)]
mod tests {
    use provwasm_mocks::mock_dependencies;

    use crate::contract_info::{
        get_contract_info, set_contract_info, CONTRACT_TYPE, CONTRACT_VERSION,
    };
    use cosmwasm_std::HumanAddr;

    #[test]
    pub fn set_contract_info_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        let result = set_contract_info(
            &mut deps.storage,
            HumanAddr::from("contract_admin"),
            "contract_bind_name",
            "contract_name",
        );
        match result {
            Ok(()) => {}
            result => panic!("unexpected error: {:?}", result),
        }

        let contract_info = get_contract_info(&deps.storage);
        match contract_info {
            Ok(contract_info) => {
                assert_eq!(contract_info.admin, HumanAddr::from("contract_admin"));
                assert_eq!(contract_info.bind_name, "contract_bind_name");
                assert_eq!(contract_info.contract_name, "contract_name");
                assert_eq!(contract_info.contract_type, CONTRACT_TYPE);
                assert_eq!(contract_info.contract_version, CONTRACT_VERSION);
            }
            result => panic!("unexpected error: {:?}", result),
        }
    }
}
