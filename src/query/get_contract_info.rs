use crate::storage::contract_info::get_contract_info;
use crate::types::core::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

pub fn query_contract_info(deps: Deps<ProvenanceQuery>) -> Result<Binary, ContractError> {
    to_binary(&get_contract_info(deps.storage)?)?.to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::core::msg::QueryMsg;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::Addr;
    use provwasm_mocks::mock_dependencies;

    #[test]
    pub fn query_with_valid_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // query for contract_info
        let query_contract_info_response =
            query(deps.as_ref(), mock_env(), QueryMsg::GetContractInfo {});

        match query_contract_info_response {
            Ok(contract_info) => {
                assert_eq!(
                    contract_info,
                    to_binary(&get_contract_info(&deps.storage).unwrap()).unwrap()
                )
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }
    }
}
