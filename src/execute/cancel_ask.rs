use crate::storage::ask_order_storage::{delete_ask_order_by_id, get_ask_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::ask_collateral::AskCollateral;
use crate::types::error::ContractError;
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
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::ask_order_storage::insert_ask_order;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::ask::Ask;
    use crate::types::ask_order::AskOrder;
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{attr, coins, Addr, CosmosMsg};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn cancel_coin_ask_with_valid_data() {
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
                    "failed to find AskOrder by id [unknown_id]: NotFound { kind: \"bilateral_exchange::types::ask_order::AskOrder\" }",
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
}
