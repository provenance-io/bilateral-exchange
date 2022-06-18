use crate::storage::ask_order_storage::{get_ask_order_by_id, insert_ask_order};
use crate::types::ask::{Ask, CoinTradeAsk, MarkerShareSaleAsk, MarkerTradeAsk, ScopeTradeAsk};
use crate::types::ask_collateral::AskCollateral;
use crate::types::ask_order::AskOrder;
use crate::types::error::ContractError;
use crate::types::request_descriptor::RequestDescriptor;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{check_scope_owners, get_single_marker_coin_holding};
use crate::validation::marker_exchange_validation::validate_marker_for_ask;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{
    revoke_marker_access, AccessGrant, MarkerAccess, ProvenanceMsg, ProvenanceQuerier,
    ProvenanceQuery,
};

pub fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    env: Env,
    ask: Ask,
    descriptor: Option<RequestDescriptor>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // If loading an ask by the target id returns an Ok response, then the requested id already
    // exists in storage and should not be overwritten
    if get_ask_order_by_id(deps.storage, ask.get_id()).is_ok() {
        return ContractError::ExistingId {
            id_type: "ask".to_string(),
            id: ask.get_id().to_string(),
        }
        .to_err();
    }
    let ask_creation_data = match &ask {
        Ask::CoinTrade(coin_ask) => create_coin_ask_collateral(&info, &coin_ask),
        Ask::MarkerTrade(marker_ask) => {
            create_marker_ask_collateral(&deps, &info, &env, &marker_ask)
        }
        Ask::MarkerShareSale(marker_share_sale) => {
            create_marker_share_sale_ask_collateral(&deps, &info, &env, &marker_share_sale)
        }
        Ask::ScopeTrade(scope_trade) => {
            create_scope_trade_ask_collateral(&deps, &info, &env, &scope_trade)
        }
    }?;
    let ask_order = AskOrder::new(
        ask.get_id(),
        info.sender,
        ask_creation_data.collateral,
        descriptor,
    )?;
    insert_ask_order(deps.storage, &ask_order)?;
    Response::new()
        .add_messages(ask_creation_data.messages)
        .add_attribute("action", "create_ask")
        .set_data(to_binary(&ask_order)?)
        .to_ok()
}

struct AskCreationData {
    pub collateral: AskCollateral,
    pub messages: Vec<CosmosMsg<ProvenanceMsg>>,
}

// create ask entrypoint
fn create_coin_ask_collateral(
    info: &MessageInfo,
    coin_trade: &CoinTradeAsk,
) -> Result<AskCreationData, ContractError> {
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "coin trade ask requests should include funds".to_string(),
        }
        .to_err();
    }
    if coin_trade.id.is_empty() {
        return ContractError::MissingField { field: "id".into() }.to_err();
    }
    if coin_trade.quote.is_empty() {
        return ContractError::MissingField {
            field: "quote".into(),
        }
        .to_err();
    }

    AskCreationData {
        collateral: AskCollateral::coin_trade(&info.funds, &coin_trade.quote),
        messages: vec![],
    }
    .to_ok()
}

fn create_marker_ask_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    env: &Env,
    marker_trade: &MarkerTradeAsk,
) -> Result<AskCreationData, ContractError> {
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: format!("marker trade ask requests should not include funds"),
        }
        .to_err();
    }
    let marker = ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_trade.denom)?;
    validate_marker_for_ask(
        &marker,
        &info.sender,
        &env.contract.address,
        &[MarkerAccess::Admin],
    )?;
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
    AskCreationData {
        collateral: AskCollateral::marker_trade(
            marker.address.clone(),
            &marker.denom,
            get_single_marker_coin_holding(&marker)?.amount.u128(),
            &marker_trade.quote_per_share,
            &marker
                .permissions
                .into_iter()
                .filter(|perm| perm.address != env.contract.address)
                .collect::<Vec<AccessGrant>>(),
        ),
        messages,
    }
    .to_ok()
}

fn create_marker_share_sale_ask_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    env: &Env,
    marker_share_sale: &MarkerShareSaleAsk,
) -> Result<AskCreationData, ContractError> {
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: format!("marker share sale ask requests should not include funds"),
        }
        .to_err();
    }
    let marker =
        ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_share_sale.denom)?;
    validate_marker_for_ask(
        &marker,
        &info.sender,
        &env.contract.address,
        &[MarkerAccess::Admin, MarkerAccess::Withdraw],
    )?;
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
    AskCreationData {
        collateral: AskCollateral::marker_share_sale(
            marker.address.clone(),
            &marker.denom,
            get_single_marker_coin_holding(&marker)?.amount.u128(),
            &marker_share_sale.quote_per_share,
            &marker
                .permissions
                .into_iter()
                .filter(|perm| perm.address != env.contract.address)
                .collect::<Vec<AccessGrant>>(),
            marker_share_sale.share_sale_type.to_owned(),
        ),
        messages,
    }
    .to_ok()
}

fn create_scope_trade_ask_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    env: &Env,
    scope_trade: &ScopeTradeAsk,
) -> Result<AskCreationData, ContractError> {
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: format!("scope trade ask requests should not include funds"),
        }
        .to_err();
    }
    check_scope_owners(
        &ProvenanceQuerier::new(&deps.querier).get_scope(&scope_trade.scope_address)?,
        Some(&env.contract.address),
        Some(&env.contract.address),
    )?;
    AskCreationData {
        collateral: AskCollateral::scope_trade(&scope_trade.scope_address, &scope_trade.quote),
        messages: vec![],
    }
    .to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::ask_order_storage::get_ask_order_by_id;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::msg::ExecuteMsg;
    use crate::types::request_type::RequestType;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{attr, coins, Addr};
    use provwasm_mocks::mock_dependencies;

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
            ask: Ask::new_coin_trade("ask_id", &coins(100, "quote_1")),
            descriptor: None,
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
            ask: Ask::CoinTrade(CoinTradeAsk { id, quote }),
            descriptor: None,
        } = create_ask_msg
        {
            match get_ask_order_by_id(deps.as_ref().storage, "ask_id") {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrder {
                            id,
                            ask_type: RequestType::CoinTrade,
                            owner: asker_info.sender,
                            collateral: AskCollateral::coin_trade(&asker_info.funds, &quote),
                            descriptor: None,
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
            ask: Ask::new_coin_trade("", &[]),
            descriptor: None,
        };

        // handle create ask
        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        )
        .expect_err("an error should occur when an invalid funds are provided");

        // verify handle create ask response returns ContractError::MissingField { id }
        match create_ask_response {
            ContractError::InvalidFundsProvided { message } => {
                assert_eq!("coin trade ask requests should include funds", message,)
            }
            e => panic!(
                "unexpected error when including no funds in an ask request: {:?}",
                e
            ),
        };

        // create ask missing id
        let create_ask_msg = ExecuteMsg::CreateAsk {
            ask: Ask::new_coin_trade("", &coins(100, "quote_1")),
            descriptor: None,
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
            ask: Ask::new_coin_trade("id", &[]),
            descriptor: None,
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
            ask: Ask::new_coin_trade("id", &coins(100, "quote_1")),
            descriptor: None,
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
