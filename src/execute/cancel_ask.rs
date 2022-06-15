use crate::storage::ask_order::{delete_ask_order_by_id, get_ask_order_by_id, AskCollateral};
use crate::storage::contract_info::get_contract_info;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{attr, to_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{grant_marker_access, revoke_marker_access, ProvenanceMsg, ProvenanceQuery};

// cancel ask entrypoint
pub fn cancel_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if id.is_empty() {
        return ContractError::InvalidFields {
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
        AskCollateral::Coin { base, .. } => {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: ask_order.owner.to_string(),
                amount: base.to_owned(),
            }));
        }
        AskCollateral::Marker {
            denom,
            removed_permissions,
            ..
        } => {
            // Restore all permissions that the marker had before it was transferred to the
            // contract.
            for permission in removed_permissions {
                messages.push(grant_marker_access(
                    denom,
                    permission.address.to_owned(),
                    permission.permissions.to_owned(),
                )?);
            }
            // Remove the contract's ownership of the marker now that it is no longer available for
            // sale.
            messages.push(revoke_marker_access(denom, env.contract.address)?);
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
    use crate::storage::ask_order::{insert_ask_order, AskOrder};
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::ask_base::AskBase;
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, CosmosMsg};
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
            ask: AskBase::new_coin("ask_id", coins(100, "quote_1")),
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
        );

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
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
        );

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel ask order with sender not equal to owner returns ContractError::Unauthorized
        if let Err(error) = insert_ask_order(
            &mut deps.storage,
            &AskOrder::new_unchecked(
                "ask_id",
                Addr::unchecked(""),
                AskCollateral::coin(coins(200, "base_1"), coins(100, "quote_1")),
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

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

        match cancel_response {
            Err(error) => match error {
                ContractError::CancelWithFunds => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }
    }
}
