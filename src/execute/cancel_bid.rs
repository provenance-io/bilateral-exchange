use crate::error::ContractError;
use crate::state::{get_bid_storage, get_bid_storage_read};
use cosmwasm_std::{attr, BankMsg, DepsMut, Env, MessageInfo, Response};
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
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return Err(ContractError::CancelWithFunds {});
    }

    let bid_storage = get_bid_storage_read(deps.storage);
    let stored_bid_order = bid_storage.load(id.as_bytes());
    match stored_bid_order {
        Ok(stored_bid_order) => {
            if !info.sender.eq(&stored_bid_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut bid_storage = get_bid_storage(deps.storage);
            bid_storage.remove(id.as_bytes());

            // 'send quote back to owner' message
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: stored_bid_order.owner.to_string(),
                    amount: stored_bid_order.quote,
                })
                .add_attributes(vec![attr("action", "cancel_bid")]))
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::execute;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::ExecuteMsg;
    use crate::state::get_bid_storage_read;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{attr, coins, Addr, BankMsg, Coin, CosmosMsg, Timestamp, Uint128};
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
            id: "bid_id".into(),
            base: vec![Coin {
                denom: "base_1".into(),
                amount: Uint128::new(200),
            }],
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),);

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
        let bid_storage = get_bid_storage_read(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_err());
    }
}
