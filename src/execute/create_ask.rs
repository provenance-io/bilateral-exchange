use crate::storage::ask_order::{get_ask_order_by_id, insert_ask_order, AskCollateral, AskOrder};
use crate::types::ask_base::{AskBase, CoinAskBase, MarkerAskBase};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::validation::marker_validation::validate_marker_for_ask;
use cosmwasm_std::{attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{revoke_marker_access, ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

pub fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    env: Env,
    ask_base: AskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // If loading an ask by the target id returns an Ok response, then the requested id already
    // exists in storage and should not be overwritten
    if get_ask_order_by_id(deps.storage, ask_base.get_id()).is_ok() {
        return ContractError::ExistingAskId {
            id: ask_base.get_id().to_string(),
        }
        .to_err();
    }

    match ask_base {
        AskBase::Coin(coin_ask) => create_coin_ask(deps, info, coin_ask),
        AskBase::Marker(marker_ask) => create_marker_ask(deps, info, env, marker_ask),
    }
}

// create ask entrypoint
pub fn create_coin_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    coin_ask: CoinAskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: format!("coin ask requests should include funds"),
        }
        .to_err();
    }
    if coin_ask.id.is_empty() {
        return ContractError::MissingField { field: "id".into() }.to_err();
    }
    if coin_ask.quote.is_empty() {
        return ContractError::MissingField {
            field: "quote".into(),
        }
        .to_err();
    }

    let ask_order = AskOrder::new(
        coin_ask.id,
        info.sender,
        AskCollateral::Coin {
            base: info.funds,
            quote: coin_ask.quote,
        },
    )?;

    insert_ask_order(deps.storage, &ask_order)?;

    Response::new()
        .add_attributes(vec![attr("action", "create_ask")])
        .set_data(to_binary(&ask_order)?)
        .to_ok()
}

pub fn create_marker_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    env: Env,
    marker_ask: MarkerAskBase,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: format!("marker ask requests should not include funds"),
        }
        .to_err();
    }
    let marker = ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_ask.denom)?;
    validate_marker_for_ask(&marker, &info.sender, &env.contract.address)?;
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];
    for permission in marker
        .permissions
        .iter()
        .filter(|perm| perm.address != env.contract.address)
    {
        messages.push(revoke_marker_access(
            &marker.denom,
            permission.clone().address,
        )?);
    }
    let ask_order = AskOrder::new(
        marker_ask.id,
        info.sender,
        AskCollateral::Marker {
            address: marker.address,
            denom: marker.denom,
            removed_permissions: marker
                .permissions
                .into_iter()
                .filter(|perm| perm.address != env.contract.address)
                .collect(),
        },
    )?;
    insert_ask_order(deps.storage, &ask_order)?;
    Response::new()
        .add_messages(messages)
        .add_attribute("action", "create_ask")
        .set_data(to_binary(&ask_order)?)
        .to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::ask_order::get_ask_order_by_id;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::ask_base::COIN_ASK_TYPE;
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr};
    use provwasm_mocks::mock_dependencies;
    use schemars::_serde_json::{from_str, to_string};

    #[test]
    fn create_coin_ask_with_valid_data() {
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
        if let ExecuteMsg::CreateAsk {
            base: AskBase::Coin(CoinAskBase { id, quote }),
        } = create_ask_msg
        {
            match get_ask_order_by_id(deps.as_ref().storage, "ask_id") {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrder {
                            id,
                            ask_type: COIN_ASK_TYPE.to_string(),
                            owner: asker_info.sender,
                            collateral: AskCollateral::Coin {
                                base: asker_info.funds,
                                quote,
                            },
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
    fn create_coin_ask_with_invalid_data() {
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
                ContractError::InvalidFundsProvided { .. } => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }
}
