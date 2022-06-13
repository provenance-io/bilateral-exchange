use crate::state::get_ask_storage_read;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use provwasm_std::ProvenanceQuery;

pub fn query_ask(deps: Deps<ProvenanceQuery>, id: String) -> StdResult<Binary> {
    to_binary(&get_ask_storage_read(deps.storage).load(id.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::QueryMsg;
    use crate::state::{get_ask_storage, AskOrder};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{coins, Addr};
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

        // store valid ask order
        let ask_order = AskOrder {
            base: coins(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // query for ask order
        let query_ask_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAsk {
                id: ask_order.id.clone(),
            },
        );

        assert_eq!(query_ask_response, to_binary(&ask_order));
    }
}
