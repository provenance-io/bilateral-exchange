use crate::storage::ask_order::get_ask_order_by_id;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use provwasm_std::ProvenanceQuery;

pub fn query_ask(deps: Deps<ProvenanceQuery>, id: String) -> StdResult<Binary> {
    to_binary(&get_ask_order_by_id(deps.storage, id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query;
    use crate::storage::ask_order::{insert_ask_order, AskCollateral, AskOrder};
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::ask_base::COIN_ASK_TYPE;
    use crate::types::msg::QueryMsg;
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
            id: "ask_id".into(),
            ask_type: COIN_ASK_TYPE.to_string(),
            owner: Addr::unchecked("asker"),
            collateral: AskCollateral::Coin {
                base: coins(200, "base_1"),
                quote: coins(100, "quote_1"),
            },
        };

        if let Err(error) = insert_ask_order(deps.as_mut().storage, &ask_order) {
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
