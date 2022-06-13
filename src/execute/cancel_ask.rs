use crate::error::ContractError;
use crate::state::{get_ask_storage, get_ask_storage_read};
use cosmwasm_std::{attr, BankMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

// cancel ask entrypoint
pub fn cancel_ask(
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

    let ask_storage = get_ask_storage_read(deps.storage);
    let stored_ask_order = ask_storage.load(id.as_bytes());
    match stored_ask_order {
        Err(_) => Err(ContractError::Unauthorized {}),
        Ok(stored_ask_order) => {
            if !info.sender.eq(&stored_ask_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut ask_storage = get_ask_storage(deps.storage);
            ask_storage.remove(id.as_bytes());

            // 'send base back to owner' message
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: stored_ask_order.owner.to_string(),
                    amount: stored_ask_order.base,
                })
                .add_attributes(vec![attr("action", "cancel_ask")]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::ExecuteMsg;
    use crate::state::AskOrder;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, Coin, CosmosMsg, Timestamp, Uint128};
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

        // create ask data
        let asker_info = mock_info("asker", &coins(200, "base_1"));

        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
        };

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_ok());

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
        let ask_storage = get_ask_storage_read(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_err());
    }

    #[test]
    fn cancel_with_invalid_data() {
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
                ContractError::Unauthorized {} => {}
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
                ContractError::Unauthorized {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }

        // cancel ask order with sender not equal to owner returns ContractError::Unauthorized
        if let Err(error) = get_ask_storage(&mut deps.storage).save(
            "ask_id".to_string().as_bytes(),
            &AskOrder {
                base: coins(200, "base_1"),
                id: "ask_id".into(),
                owner: Addr::unchecked(""),
                quote: coins(100, "quote_1"),
            },
        ) {
            panic!("unexpected error: {:?}", error)
        };
        let cancel_ask_msg = ExecuteMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = execute(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

        match cancel_response {
            Err(error) => match error {
                ContractError::Unauthorized {} => {}
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
                ContractError::CancelWithFunds {} => {}
                _ => {
                    panic!("unexpected error: {:?}", error)
                }
            },
            Ok(_) => panic!("expected error, but cancel_response ok"),
        }
    }
}
