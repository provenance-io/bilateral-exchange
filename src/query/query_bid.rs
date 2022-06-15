use crate::storage::bid_order::get_bid_order_by_id;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

pub fn query_bid(deps: Deps<ProvenanceQuery>, id: String) -> Result<Binary, ContractError> {
    to_binary(&get_bid_order_by_id(deps.storage, id)?)?.to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::query;
    use crate::storage::bid_order::{insert_bid_order, BidCollateral, BidOrder};
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::msg::QueryMsg;
    use crate::types::request_descriptor::RequestDescriptor;
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
        let bid_order = BidOrder::new_unchecked(
            "bid_id",
            Addr::unchecked("bidder"),
            BidCollateral::coin(&coins(100, "base_1"), &coins(100, "quote_1")),
            Some(RequestDescriptor {
                description: Some("description words".to_string()),
                effective_time: Some(Timestamp::default()),
            }),
        );

        if let Err(error) = insert_bid_order(deps.as_mut().storage, &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // query for bid order
        let query_bid_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetBid {
                id: bid_order.id.clone(),
            },
        )
        .expect("expected the query to execute successfully");

        assert_eq!(
            query_bid_response,
            to_binary(&bid_order).expect("expected binary serialization to succeed for bid order"),
        );
    }
}
