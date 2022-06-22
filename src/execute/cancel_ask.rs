use crate::storage::ask_order_storage::{delete_ask_order_by_id, get_ask_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::core::error::ContractError;
use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{release_marker_from_contract, replace_scope_owner};
use cosmwasm_std::{to_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{write_scope, ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

// cancel ask entrypoint
pub fn cancel_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if id.is_empty() {
        return ContractError::ValidationError {
            messages: vec!["an id must be provided when cancelling an ask".to_string()],
        }
        .to_err();
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds should not be provided when cancelling an ask".to_string(),
        }
        .to_err();
    }
    let ask_order = get_ask_order_by_id(deps.storage, &id)?;
    // Only the owner of the ask and the admin can cancel an ask
    if info.sender != ask_order.owner && get_contract_info(deps.storage)?.admin != ask_order.owner {
        return ContractError::Unauthorized.to_err();
    }
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];
    match &ask_order.collateral {
        AskCollateral::CoinTrade(collateral) => {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: ask_order.owner.to_string(),
                amount: collateral.base.to_owned(),
            }));
        }
        AskCollateral::MarkerTrade(collateral) => {
            messages.append(&mut release_marker_from_contract(
                &collateral.denom,
                &env.contract.address,
                &collateral.removed_permissions,
            )?);
        }
        AskCollateral::MarkerShareSale(collateral) => {
            messages.append(&mut release_marker_from_contract(
                &collateral.denom,
                &env.contract.address,
                &collateral.removed_permissions,
            )?);
        }
        AskCollateral::ScopeTrade(collateral) => {
            let mut scope =
                ProvenanceQuerier::new(&deps.querier).get_scope(&collateral.scope_address)?;
            scope = replace_scope_owner(scope, ask_order.owner.to_owned());
            messages.push(write_scope(scope, vec![env.contract.address])?);
        }
    }
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    Response::new()
        .add_messages(messages)
        .add_attribute("action", "cancel_ask")
        .set_data(to_binary(&ask_order)?)
        .to_ok()
}

#[cfg(test)]
#[cfg(feature = "enable-test-utils")]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::execute::create_ask::create_ask;
    use crate::storage::ask_order_storage::insert_ask_order;
    use crate::test::mock_instantiate::default_instantiate;
    use crate::test::mock_marker::{MockMarker, DEFAULT_MARKER_DENOM};
    use crate::types::core::msg::ExecuteMsg;
    use crate::types::request::ask_types::ask::Ask;
    use crate::types::request::ask_types::ask_order::AskOrder;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{attr, coins, from_binary, Addr, CosmosMsg};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{MarkerMsgParams, ProvenanceMsgParams};

    #[test]
    fn cancel_coin_ask_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        default_instantiate(&mut deps.storage);

        // create ask data
        let asker_info = mock_info("asker", &coins(200, "base_1"));

        let create_ask_msg = ExecuteMsg::CreateAsk {
            ask: Ask::new_coin_trade("ask_id", &coins(100, "quote_1")),
            descriptor: None,
        };

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        assert!(get_ask_order_by_id(deps.as_ref().storage, "ask_id").is_ok());

        // cancel ask order
        let asker_info = mock_info("asker", &[]);

        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };
        let cancel_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        );

        match cancel_ask_response {
            Ok(cancel_ask_response) => {
                assert_eq!(cancel_ask_response.attributes.len(), 1);
                assert_eq!(
                    cancel_ask_response.attributes[0],
                    attr("action", "cancel_ask")
                );
                assert_eq!(cancel_ask_response.messages.len(), 1);
                assert_eq!(
                    cancel_ask_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: asker_info.sender.to_string(),
                        amount: coins(200, "base_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify ask order removed from storage
        assert!(get_ask_order_by_id(deps.as_ref().storage, "ask_id").is_err());
    }

    #[test]
    fn cancel_coin_ask_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        default_instantiate(&mut deps.storage);

        let asker_info = mock_info("asker", &[]);

        // cancel ask order with missing id returns ContractError::Unauthorized
        let cancel_ask_msg = ExecuteMsg::CancelAsk { id: "".to_string() };
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        )
        .expect_err("expected an error to occur when a blank id is provided");

        match cancel_response {
            ContractError::ValidationError { messages } => {
                assert_eq!(1, messages.len());
                assert_eq!(
                    "an id must be provided when cancelling an ask",
                    messages.first().unwrap(),
                );
            }
            e => panic!("unexpected error with invalid id provided: {:?}", e),
        }

        // cancel non-existent ask order returns ContractError::Unauthorized
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "unknown_id".to_string(),
        };

        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            cancel_ask_msg,
        )
        .expect_err("expected an error to occur for a missing ask");

        match cancel_response {
            ContractError::StorageError { message } => {
                assert_eq!(
                    "failed to find AskOrder by id [unknown_id]: NotFound { kind: \"bilateral_exchange::types::request::ask_types::ask_order::AskOrder\" }",
                    message,
                );
            }
            e => panic!("unexpected error when cancelling missing ask: {:?}", e),
        }

        // cancel ask order with sender not equal to owner returns ContractError::Unauthorized
        if let Err(error) = insert_ask_order(
            &mut deps.storage,
            &AskOrder::new_unchecked(
                "ask_id",
                Addr::unchecked(""),
                AskCollateral::coin_trade(&coins(200, "base_1"), &coins(100, "quote_1")),
                None,
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        };
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel ask order with sent_funds returns ContractError::CancelWithFunds
        let asker_info = mock_info("asker", &coins(1, "sent_coin"));
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg)
            .expect_err("expected an error to occur when sending funds");

        match cancel_response {
            ContractError::InvalidFundsProvided { message } => {
                assert_eq!(
                    "funds should not be provided when cancelling an ask",
                    message,
                );
            }
            e => panic!(
                "unexpected error occurred when sending funds while cancelling an ask: {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_cancel_marker_trade_ask_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        default_instantiate(&mut deps.storage);
        let ask_id = "ask_id".to_string();
        deps.querier
            .with_markers(vec![MockMarker::new_owned_marker("asker")]);
        create_ask(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            Ask::new_marker_trade(&ask_id, DEFAULT_MARKER_DENOM, &coins(150, "nhash")),
            None,
        )
        .expect("marker trade ask should be created without issue");
        let ask_order = get_ask_order_by_id(&mut deps.storage, &ask_id)
            .expect("an ask order should be available in storage");
        let response = cancel_ask(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            ask_id.to_owned(),
        )
        .expect("cancel ask should succeed");
        assert_eq!(
            2,
            response.messages.len(),
            "two message should be added to the response to properly rewrite the marker to its original owner permissions",
        );
        response.messages.iter().for_each(|msg| match &msg.msg {
            CosmosMsg::Custom(ProvenanceMsg {
                                  params: ProvenanceMsgParams::Marker(MarkerMsgParams::GrantMarkerAccess {
                                                                          denom,
                                                                          address,
                                                                          ..
                                                                      }),
                                  ..
                              }) => {
                assert_eq!(
                    DEFAULT_MARKER_DENOM,
                    denom,
                    "the correct marker denom should be referenced in the grant access request",
                );
                assert_eq!(
                    "asker",
                    address.as_str(),
                    "the asker account should be granted its marker access again",
                );
            },
            CosmosMsg::Custom(ProvenanceMsg {
                                  params: ProvenanceMsgParams::Marker(MarkerMsgParams::RevokeMarkerAccess {
                                                                          denom,
                                                                          address,
                                                                      }),
                                  ..
                              }) => {
                assert_eq!(
                    DEFAULT_MARKER_DENOM,
                    denom,
                    "the correct marker denom should be referenced in the revoke access request",
                );
                assert_eq!(
                    MOCK_CONTRACT_ADDR,
                    address.as_str(),
                    "the contract address should be used to revoke its marker access",
                );
            },
            msg => panic!("unexpected message produced when cancelling a marker ask: {:?}", msg),
        });
        assert_eq!(
            1,
            response.attributes.len(),
            "the response should have a single attribute",
        );
        let attribute = response.attributes.first().unwrap();
        assert_eq!("action", attribute.key);
        assert_eq!("cancel_ask", attribute.value);
        let response_data_ask_order = from_binary::<AskOrder>(
            &response.data.expect("response data should be set"),
        )
        .expect("the response data should be able to be converted from binary to an AskOrder");
        assert_eq!(
            ask_order, response_data_ask_order,
            "the response data's AskOrder should equate to the cancelled ask order",
        );
        get_ask_order_by_id(&deps.storage, &ask_id)
            .expect_err("the ask should no longer be available in storage after a cancellation");
    }

    #[test]
    fn test_cancel_marker_trade_ask_with_invalid_data() {}
}
