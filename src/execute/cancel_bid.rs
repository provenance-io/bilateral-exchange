use crate::storage::bid_order_storage::{delete_bid_order_by_id, get_bid_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::core::error::ContractError;
use crate::types::request::bid_types::bid_collateral::BidCollateral;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{to_binary, BankMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

// cancel bid entrypoint
pub fn cancel_bid(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if id.is_empty() {
        return ContractError::ValidationError {
            messages: vec!["an id must be provided when cancelling a bid".to_string()],
        }
        .to_err();
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds should not be provided when cancelling a bid".to_string(),
        }
        .to_err();
    }
    let bid_order = get_bid_order_by_id(deps.storage, &id)?;
    // Only the owner of the bid and the admin can cancel a bid
    if info.sender != bid_order.owner && get_contract_info(deps.storage)?.admin != bid_order.owner {
        return ContractError::Unauthorized.to_err();
    }
    let coin_to_send = match &bid_order.collateral {
        BidCollateral::CoinTrade(collateral) => collateral.quote.to_owned(),
        BidCollateral::MarkerTrade(collateral) => collateral.quote.to_owned(),
        BidCollateral::MarkerShareSale(collateral) => collateral.quote.to_owned(),
        BidCollateral::ScopeTrade(collateral) => collateral.quote.to_owned(),
    };
    // Remove the bid order from storage now that it is no longer needed
    delete_bid_order_by_id(deps.storage, &id)?;
    Response::new()
        .add_message(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: coin_to_send,
        })
        .add_attribute("action", "cancel_bid")
        .set_data(to_binary(&bid_order)?)
        .to_ok()
}

#[cfg(test)]
mod tests {
    use crate::contract::execute;
    use crate::storage::bid_order_storage::get_bid_order_by_id;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::core::msg::ExecuteMsg;
    use crate::types::request::bid_types::bid::Bid;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{attr, coins, Addr, BankMsg, CosmosMsg};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn cancel_with_valid_data() {
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

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "quote_1"));
        let create_bid_msg = ExecuteMsg::CreateBid {
            bid: Bid::new_coin("bid_id", &coins(200, "base_1")),
            descriptor: None,
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        assert!(get_bid_order_by_id(deps.as_ref().storage, "bid_id").is_ok());

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = ExecuteMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            cancel_bid_msg,
        );

        match cancel_bid_response {
            Ok(cancel_bid_response) => {
                assert_eq!(cancel_bid_response.attributes.len(), 1);
                assert_eq!(
                    cancel_bid_response.attributes[0],
                    attr("action", "cancel_bid")
                );
                assert_eq!(cancel_bid_response.messages.len(), 1);
                assert_eq!(
                    cancel_bid_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bidder_info.sender.to_string(),
                        amount: coins(100, "quote_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        assert!(
            get_bid_order_by_id(deps.as_ref().storage, "bid_id").is_err(),
            "the bid should be removed from storage after a successful cancellation",
        );
    }
}
