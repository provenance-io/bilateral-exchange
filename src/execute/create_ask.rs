use crate::storage::state::{get_ask_storage, get_ask_storage_read, AskOrder};
use crate::types::ask_base::{AskBase, CoinAskBase, MarkerAskBase};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{attr, to_binary, DepsMut, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

pub fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    ask_base: AskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // If loading an ask by the target id returns an Ok response, then the requested id already
    // exists in storage and should not be overwritten
    if get_ask_storage_read(deps.storage)
        .load(ask_base.get_storage_key())
        .is_ok()
    {
        return ContractError::ExistingAskId {
            id: ask_base.get_id().to_string(),
        }
        .to_err();
    }

    match ask_base {
        AskBase::Coin(coin_ask) => create_coin_ask(deps, info, coin_ask),
        AskBase::Marker(marker_ask) => create_marker_ask(deps, info, marker_ask),
    }
}

// create ask entrypoint
pub fn create_coin_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    coin_ask: CoinAskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if coin_ask.id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if info.funds.is_empty() {
        return Err(ContractError::MissingAskBase);
    }
    if coin_ask.quote.is_empty() {
        return Err(ContractError::MissingField {
            field: "quote".into(),
        });
    }

    let mut ask_storage = get_ask_storage(deps.storage);

    let ask_order = AskOrder {
        base: info.funds,
        id: coin_ask.id,
        owner: info.sender,
        quote: coin_ask.quote,
    };

    ask_storage.save(ask_order.id.as_bytes(), &ask_order)?;

    Response::new()
        .add_attributes(vec![attr("action", "create_ask")])
        .set_data(to_binary(&ask_order)?)
        .to_ok()
}

pub fn create_marker_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    marker_ask: MarkerAskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    let querier = ProvenanceQuerier::new(&deps.querier);
    let marker = querier.get_marker_by_denom(&marker_ask.denom)?;
    marker.
    Response::new().to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::storage::state::get_ask_storage_read;
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr};
    use provwasm_mocks::mock_dependencies;
    use schemars::_serde_json::{from_str, to_string};

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
            base: AskBase::new_coin("ask_id", coins(100, "quote_1")),
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
        if let ExecuteMsg::CreateAsk {
            base: AskBase::Coin(CoinAskBase { id, quote }),
        } = create_ask_msg
        {
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
            base: AskBase::new_coin("", vec![]),
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
            base: AskBase::new_coin("", coins(100, "quote_1")),
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
            base: AskBase::new_coin("id", vec![]),
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
            base: AskBase::new_coin("id", coins(100, "quote_1")),
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
                ContractError::MissingAskBase => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }
}
