use crate::storage::ask_order_storage::{get_ask_order_by_id, insert_ask_order};
use crate::types::core::error::ContractError;
use crate::types::request::ask_types::ask::{
    Ask, CoinTradeAsk, MarkerShareSaleAsk, MarkerTradeAsk, ScopeTradeAsk,
};
use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::types::request::ask_types::ask_order::AskOrder;
use crate::types::request::request_descriptor::RequestDescriptor;
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
    env: Env,
    info: MessageInfo,
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
        Ask::CoinTrade(coin_ask) => create_coin_trade_ask_collateral(&info, coin_ask),
        Ask::MarkerTrade(marker_ask) => {
            create_marker_trade_ask_collateral(&deps, &info, &env, marker_ask)
        }
        Ask::MarkerShareSale(marker_share_sale) => {
            create_marker_share_sale_ask_collateral(&deps, &info, &env, marker_share_sale)
        }
        Ask::ScopeTrade(scope_trade) => {
            create_scope_trade_ask_collateral(&deps, &info, &env, scope_trade)
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
fn create_coin_trade_ask_collateral(
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

fn create_marker_trade_ask_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    env: &Env,
    marker_trade: &MarkerTradeAsk,
) -> Result<AskCreationData, ContractError> {
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "marker trade ask requests should not include funds".to_string(),
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
            message: "marker share sale ask requests should not include funds".to_string(),
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
            message: "scope trade ask requests should not include funds".to_string(),
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
#[cfg(feature = "enable-test-utils")]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::ask_order_storage::get_ask_order_by_id;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::test::mock_instantiate::default_instantiate;
    use crate::test::mock_marker::{
        MockMarker, DEFAULT_MARKER_ADDRESS, DEFAULT_MARKER_DENOM, DEFAULT_MARKER_HOLDINGS,
    };
    use crate::types::core::msg::ExecuteMsg;
    use crate::types::request::request_type::RequestType;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr, Storage};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{MarkerMsgParams, ProvenanceMsgParams};

    #[test]
    fn create_coin_trade_ask_with_valid_data() {
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
        )
        .expect("coin trade ask should properly respond");

        assert!(
            create_ask_response.messages.is_empty(),
            "coin trades should not generate any messages, but got messages: {:?}",
            create_ask_response.messages.to_owned(),
        );

        let ask_order = assert_valid_response(&deps.storage, &create_ask_response);
        assert_eq!("ask_id", ask_order.id);
        assert_eq!("asker", ask_order.owner.as_str());
        assert_eq!(RequestType::CoinTrade, ask_order.ask_type);
        assert_eq!(None, ask_order.descriptor);
        let collateral = match &ask_order.collateral {
            AskCollateral::CoinTrade(collateral) => collateral,
            _ => panic!("unexpected collateral found for coin trade ask order"),
        };
        assert_eq!(coins(2, "base_1"), collateral.base);
        assert_eq!(coins(100, "quote_1"), collateral.quote);
    }

    #[test]
    fn create_coin_trade_ask_with_invalid_data() {
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

    #[test]
    fn create_marker_trade_ask_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        default_instantiate(&mut deps.storage);
        deps.querier
            .with_markers(vec![MockMarker::new_owned_marker("asker")]);
        let descriptor = RequestDescriptor::basic("a decent ask");
        let response = create_ask(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            Ask::new_marker_trade("ask_id", DEFAULT_MARKER_DENOM, &[coin(150, "nhash")]),
            Some(descriptor.to_owned()),
        )
        .expect("expected the ask to be accepted");
        assert_eq!(
            1,
            response.messages.len(),
            "expected a single message to be emitted for the marker trade",
        );
        match &response.messages.first().unwrap().msg {
            CosmosMsg::Custom(ProvenanceMsg {
                params:
                    ProvenanceMsgParams::Marker(MarkerMsgParams::RevokeMarkerAccess { denom, address }),
                ..
            }) => {
                assert_eq!(
                    DEFAULT_MARKER_DENOM, denom,
                    "the default marker denom should be referenced in the revocation",
                );
                assert_eq!(
                    "asker",
                    address.as_str(),
                    "the asker address should be revoked its access from the marker on a successful ask",
                );
            }
            msg => panic!("unexpected message in marker trade: {:?}", msg),
        }
        let ask_order = assert_valid_response(&deps.storage, &response);
        assert_eq!(
            "ask_id", ask_order.id,
            "the proper ask id should be set in the ask order",
        );
        assert_eq!(
            RequestType::MarkerTrade,
            ask_order.ask_type,
            "the proper request type should bet set in the ask order",
        );
        assert_eq!(
            "asker",
            ask_order.owner.as_str(),
            "the proper owner address should be set in the ask order",
        );
        assert_eq!(
            descriptor,
            ask_order
                .descriptor
                .expect("the descriptor should be set in the ask order"),
            "the proper descriptor should be set in the ask order",
        );
        let marker_trade_collateral = match &ask_order.collateral {
            AskCollateral::MarkerTrade(collateral) => collateral,
            collateral => panic!("incorrect collateral type in ask order: {:?}", collateral),
        };
        assert_eq!(
            DEFAULT_MARKER_ADDRESS,
            marker_trade_collateral.address.as_str(),
            "the correct marker address should be set in the marker trade collateral",
        );
        assert_eq!(
            DEFAULT_MARKER_DENOM, marker_trade_collateral.denom,
            "the correct marker denom should be set in the marker trade collateral",
        );
        assert_eq!(
            DEFAULT_MARKER_HOLDINGS,
            marker_trade_collateral.share_count.u128(),
            "the correct marker share count should be set in the marker trade collateral",
        );
        assert_eq!(
            coins(150, "nhash"),
            marker_trade_collateral.quote_per_share,
            "the correct quote per share should be set in the marker trade collateral",
        );
    }

    #[test]
    fn test_create_marker_trade_ask_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        default_instantiate(&mut deps.storage);
        let error = create_ask(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "nhash")),
            Ask::new_marker_trade("ask_id", DEFAULT_MARKER_DENOM, &[]),
            None,
        )
        .expect_err("a marker trade with funds should be rejected");
        assert!(
            matches!(error, ContractError::InvalidFundsProvided { .. }),
            "an invalid funds error should be returned when funds are added to a marker trade ask",
        );
        let error = create_ask(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            Ask::new_marker_trade("ask_id", DEFAULT_MARKER_DENOM, &coins(100, "nhash")),
            None,
        )
        .expect_err(
            "a marker trade that references a marker that does not exist should be rejected",
        );
        assert!(
            matches!(error, ContractError::Std(_)),
            "a missing marker should cause a standard cosmwasm error",
        );
    }

    fn assert_valid_response(
        storage: &dyn Storage,
        response: &Response<ProvenanceMsg>,
    ) -> AskOrder {
        assert_eq!(
            1,
            response.attributes.len(),
            "expected only a single response attribute"
        );
        let attribute = response.attributes.first().unwrap();
        assert_eq!(
            "action", attribute.key,
            "the response attribute should have the proper key",
        );
        assert_eq!(
            "create_ask", attribute.value,
            "the response attribute should have the proper value",
        );
        let ask_order: AskOrder = if let Some(ask_order_binary) = &response.data {
            from_binary(&ask_order_binary)
                .expect("expected ask order to deserialize properly from response")
        } else {
            panic!("expected data to be properly set after a successful response")
        };
        let storage_ask_order = get_ask_order_by_id(storage, &ask_order.id)
            .expect("expected the ask order to be found by its id in storage");
        assert_eq!(
            ask_order, storage_ask_order,
            "the ask order found in storage should equate to the ask order in the output data",
        );
        ask_order
    }
}
