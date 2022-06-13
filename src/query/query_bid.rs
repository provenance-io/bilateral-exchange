use crate::state::get_bid_storage_read;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use provwasm_std::ProvenanceQuery;

pub fn query_bid(deps: Deps<ProvenanceQuery>, id: String) -> StdResult<Binary> {
    to_binary(&get_bid_storage_read(deps.storage).load(id.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::QueryMsg;
    use crate::state::{get_bid_storage, BidOrder};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{coins, Addr, Timestamp};
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

        // store valid bid order
        let bid_order = BidOrder {
            base: coins(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // query for bid order
        let query_bid_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetBid {
                id: bid_order.id.clone(),
            },
        );

        assert_eq!(query_bid_response, to_binary(&bid_order));
    }
}
