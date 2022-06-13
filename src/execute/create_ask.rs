use crate::error::ContractError;
use crate::state::{get_ask_storage, AskOrder};
use cosmwasm_std::{attr, to_binary, Coin, DepsMut, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

// create ask entrypoint
pub fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    id: String,
    quote: Vec<Coin>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if info.funds.is_empty() {
        return Err(ContractError::MissingAskBase);
    }
    if quote.is_empty() {
        return Err(ContractError::MissingField {
            field: "quote".into(),
        });
    }

    let mut ask_storage = get_ask_storage(deps.storage);

    let ask_order = AskOrder {
        base: info.funds,
        id,
        owner: info.sender,
        quote,
    };

    ask_storage.save(ask_order.id.as_bytes(), &ask_order)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "create_ask")])
        .set_data(to_binary(&ask_order)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::ExecuteMsg;
    use crate::state::get_ask_storage_read;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn create_ask_with_valid_data() {
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
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
        };

        let asker_info = mock_info("asker", &coins(2, "base_1"));

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            asker_info.clone(),
            create_ask_msg.clone(),
        );

        // verify handle create ask response
        match create_ask_response {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 1);
                assert_eq!(response.attributes[0], attr("action", "create_ask"));
            }
            Err(error) => {
                panic!("failed to create ask: {:?}", error)
            }
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read(&deps.storage);
        if let ExecuteMsg::CreateAsk { id, quote } = create_ask_msg {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrder {
                            base: asker_info.funds,
                            id,
                            owner: asker_info.sender,
                            quote,
                        }
                    )
                }
                _ => {
                    panic!("ask order was not found in storage")
                }
            }
        } else {
            panic!("ask_message is not a CreateAsk type. this is bad.")
        }
    }

    #[test]
    fn create_ask_with_invalid_data() {
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

        // create ask invalid data
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "".into(),
            quote: vec![],
        };

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        // verify handle create ask response returns ContractError::MissingField { id }
        match create_ask_response {
            Ok(_) => panic!("expected error, but handle_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing id
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "".into(),
            quote: coins(100, "quote_1"),
        };

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "base_1")),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::MissingField { id }
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing quote
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: vec![],
        };

        // execute create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "base_1")),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::MissingField { quote }
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "quote")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing base
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
        };

        // execute create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        // verify execute create ask response returns ContractError::AskMissingBase
        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingAskBase {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }
}
