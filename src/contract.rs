use cosmwasm_std::{
    attr, to_binary, BankMsg, Binary, Coin, Context, Deps, DepsMut, Env, HandleResponse,
    InitResponse, MessageInfo, StdResult,
};
use provwasm_std::{bind_name, ProvenanceMsg};

use crate::contract_info::{get_contract_info, set_contract_info};
use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{
    get_ask_storage, get_ask_storage_read, get_bid_storage, get_bid_storage_read, AskOrder,
    BidOrder,
};

// smart contract initialization entrypoint
pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse<ProvenanceMsg>, ContractError> {
    if msg.bind_name.is_empty() {
        return Err(ContractError::MissingField {
            field: "bind_name".into(),
        });
    }
    if msg.contract_name.is_empty() {
        return Err(ContractError::MissingField {
            field: "contract_name".into(),
        });
    }

    // set contract info
    set_contract_info(
        deps.storage,
        info.sender,
        msg.bind_name.clone(),
        msg.contract_name.clone(),
    )?;

    // create name binding provenance message
    let bind_name_msg = bind_name(msg.bind_name, env.contract.address);

    // build response
    Ok(InitResponse {
        messages: vec![bind_name_msg],
        attributes: vec![
            attr(
                "contract_info",
                format!("{:?}", get_contract_info(deps.storage)?),
            ),
            attr("action", "init"),
        ],
    })
}

// smart contract execute entrypoint
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    match msg {
        HandleMsg::CreateAsk { id, price } => create_ask(deps, info, id, price),
        HandleMsg::CreateBid { id, asset } => create_bid(deps, info, id, asset),
        HandleMsg::CancelAsk { id } => cancel_ask(deps, env, info, id),
        HandleMsg::CancelBid { id } => cancel_bid(deps, env, info, id),
        HandleMsg::Execute { ask_id, bid_id } => execute(deps, env, info, ask_id, bid_id),
    }
}

// create ask entrypoint
fn create_ask(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    price: Vec<Coin>,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if info.sent_funds.is_empty() {
        return Err(ContractError::MissingAskAsset);
    }
    if price.is_empty() {
        return Err(ContractError::MissingField {
            field: "price".into(),
        });
    }

    let mut ask_storage = get_ask_storage(deps.storage);

    let ask_order = AskOrder {
        asset: info.sent_funds,
        id,
        owner: info.sender,
        price,
    };

    ask_storage.save(&ask_order.id.as_bytes(), &ask_order)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![attr("action", "create_ask")],
        data: Some(to_binary(&ask_order)?),
    })
}

// create bid entrypoint
fn create_bid(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    asset: Vec<Coin>,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    if asset.is_empty() {
        return Err(ContractError::MissingField {
            field: "asset".into(),
        });
    }
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if info.sent_funds.is_empty() {
        return Err(ContractError::MissingBidPrice);
    }

    let mut bid_storage = get_bid_storage(deps.storage);

    let bid_order = BidOrder {
        price: info.sent_funds,
        id,
        owner: info.sender,
        asset,
    };

    bid_storage.save(&bid_order.id.as_bytes(), &bid_order)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![attr("action", "create_bid")],
        data: Some(to_binary(&bid_order)?),
    })
}

// cancel ask entrypoint
fn cancel_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.sent_funds.is_empty() {
        return Err(ContractError::CancelWithFunds {});
    }

    let ask_storage = get_ask_storage_read(deps.storage);
    let stored_ask_order = ask_storage.load(id.as_bytes());
    match stored_ask_order {
        Ok(stored_ask_order) => {
            if !info.sender.eq(&stored_ask_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            let mut response = Context::new();

            // add 'send asset back to owner' message
            response.add_message(BankMsg::Send {
                from_address: env.contract.address,
                to_address: stored_ask_order.owner,
                amount: stored_ask_order.asset,
            });

            response.add_attribute("action", "cancel_ask");

            // finally remove the ask order from storage
            let mut ask_storage = get_ask_storage(deps.storage);
            ask_storage.remove(id.as_bytes());

            Ok(response.into())
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

// cancel ask entrypoint
fn cancel_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.sent_funds.is_empty() {
        return Err(ContractError::CancelWithFunds {});
    }

    let bid_storage = get_bid_storage_read(deps.storage);
    let stored_bid_order = bid_storage.load(id.as_bytes());
    match stored_bid_order {
        Ok(stored_bid_order) => {
            if !info.sender.eq(&stored_bid_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            let mut response = Context::new();

            // add 'send asset back to owner' message
            response.add_message(BankMsg::Send {
                from_address: env.contract.address,
                to_address: stored_bid_order.owner,
                amount: stored_bid_order.price,
            });

            response.add_attribute("action", "cancel_bid");

            // finally remove the ask order from storage
            let mut bid_storage = get_bid_storage(deps.storage);
            bid_storage.remove(id.as_bytes());

            Ok(response.into())
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ask_id: String,
    bid_id: String,
) -> Result<HandleResponse<ProvenanceMsg>, ContractError> {
    // return error if id is empty
    if ask_id.is_empty() | bid_id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.sent_funds.is_empty() {
        return Err(ContractError::ExecuteWithFunds {});
    }

    let ask_storage_read = get_ask_storage_read(deps.storage);
    let ask_order_result = ask_storage_read.load(ask_id.as_bytes());
    if ask_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let bid_storage_read = get_bid_storage_read(deps.storage);
    let bid_order_result = bid_storage_read.load(bid_id.as_bytes());
    if bid_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let ask_order = ask_order_result.unwrap();
    let bid_order = bid_order_result.unwrap();

    if !is_executable(&ask_order, &bid_order) {
        return Err(ContractError::AskBidMismatch {});
    }

    let mut response = Context::new();

    // add 'send price to asker' message
    response.add_message(BankMsg::Send {
        from_address: env.contract.address.clone(),
        to_address: ask_order.owner,
        amount: ask_order.price,
    });

    // add 'send asset to bidder' message
    response.add_message(BankMsg::Send {
        from_address: env.contract.address,
        to_address: bid_order.owner,
        amount: bid_order.asset,
    });

    response.add_attribute("action", "execute");

    // finally remove the orders from storage
    get_ask_storage(deps.storage).remove(ask_id.as_bytes());
    get_bid_storage(deps.storage).remove(bid_id.as_bytes());

    Ok(response.into())
}

fn is_executable(ask_order: &AskOrder, bid_order: &BidOrder) -> bool {
    // sort the asset and price vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));

    let mut ask_asset = ask_order.asset.to_owned();
    ask_asset.sort_by(coin_sorter);
    let mut bid_asset = bid_order.asset.to_owned();
    bid_asset.sort_by(coin_sorter);

    let mut ask_asset = ask_order.asset.to_owned();
    ask_asset.sort_by(coin_sorter);
    let mut bid_asset = bid_order.asset.to_owned();
    bid_asset.sort_by(coin_sorter);

    ask_asset == bid_asset && ask_order.price == bid_order.price
}

// smart contract query entrypoint
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAsk { id } => {
            let ask_storage_read = get_ask_storage_read(deps.storage);
            return to_binary(&ask_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetBid { id } => {
            let bid_storage_read = get_bid_storage_read(deps.storage);
            return to_binary(&bid_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetContractInfo => to_binary(&get_contract_info(deps.storage)?),
    }
}

// unit tests
#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, BankMsg, HumanAddr};
    use cosmwasm_std::{CosmosMsg, Uint128};
    use provwasm_std::{NameMsgParams, ProvenanceMsg, ProvenanceMsgParams, ProvenanceRoute};

    use crate::contract_info::ContractInfo;
    use crate::state::get_bid_storage_read;

    use super::*;

    #[test]
    fn test_is_executable() {
        assert_eq!(
            is_executable(
                &AskOrder {
                    asset: coins(100, "asset_1"),
                    id: "ask_id".to_string(),
                    owner: HumanAddr("asker".into()),
                    price: coins(100, "price_1"),
                },
                &BidOrder {
                    asset: coins(100, "asset_1"),
                    id: "bid_id".to_string(),
                    owner: HumanAddr("bidder".into()),
                    price: coins(100, "price_1"),
                }
            ),
            true
        );
        assert_eq!(
            is_executable(
                &AskOrder {
                    asset: vec![coin(100, "asset_1"), coin(200, "asset_2")],
                    id: "ask_id".to_string(),
                    owner: HumanAddr("asker".into()),
                    price: coins(100, "price_1"),
                },
                &BidOrder {
                    asset: vec![coin(200, "asset_2"), coin(100, "asset_1")],
                    id: "bid_id".to_string(),
                    owner: HumanAddr("bidder".into()),
                    price: coins(100, "price_1"),
                }
            ),
            true
        );
        assert_eq!(
            is_executable(
                &AskOrder {
                    asset: coins(100, "asset_1"),
                    id: "ask_id".to_string(),
                    owner: HumanAddr("asker".into()),
                    price: coins(100, "price_1"),
                },
                &BidOrder {
                    asset: coins(100, "asset_2"),
                    id: "bid_id".to_string(),
                    owner: HumanAddr("bidder".into()),
                    price: coins(100, "price_1"),
                }
            ),
            false
        );
        assert_eq!(
            is_executable(
                &AskOrder {
                    asset: coins(100, "asset_1"),
                    id: "ask_id".to_string(),
                    owner: HumanAddr("asker".into()),
                    price: coins(100, "price_1"),
                },
                &BidOrder {
                    asset: coins(100, "asset_1"),
                    id: "bid_id".to_string(),
                    owner: HumanAddr("bidder".into()),
                    price: coins(100, "price_2"),
                }
            ),
            false
        );
    }

    #[test]
    fn init_with_valid_data() {
        // create valid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_admin", &[]);
        let init_msg = InitMsg {
            bind_name: "contract_bind_name".to_string(),
            contract_name: "contract_name".to_string(),
        };

        // initialize
        let init_response = init(deps.as_mut(), mock_env(), info, init_msg.clone());

        // verify initialize response
        match init_response {
            Ok(init_response) => {
                assert_eq!(init_response.messages.len(), 1);
                assert_eq!(
                    init_response.messages[0],
                    CosmosMsg::Custom(ProvenanceMsg {
                        route: ProvenanceRoute::Name,
                        params: ProvenanceMsgParams::Name(NameMsgParams::BindName {
                            name: init_msg.bind_name,
                            address: MOCK_CONTRACT_ADDR.into(),
                            restrict: true
                        }),
                        version: "2.0.0".to_string(),
                    })
                );
                let expected_contract_info = ContractInfo::new(
                    HumanAddr("contract_admin".into()),
                    "contract_name",
                    "contract_bind_name",
                );
                assert_eq!(init_response.attributes.len(), 2);
                assert_eq!(
                    init_response.attributes[0],
                    attr("contract_info", format!("{:?}", expected_contract_info))
                );
                assert_eq!(init_response.attributes[1], attr("action", "init"));
            }
            error => panic!("failed to initialize: {:?}", error),
        }
    }

    #[test]
    fn init_with_invalid_data() {
        // create invalid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_owner", &[]);
        let init_msg = InitMsg {
            bind_name: "".to_string(),
            contract_name: "contract_name".to_string(),
        };

        // initialize
        let init_response = init(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

        // verify initialize response
        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "bind_name")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        let init_msg = InitMsg {
            bind_name: "bind_name".to_string(),
            contract_name: "".to_string(),
        };

        // initialize
        let init_response = init(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

        // verify initialize response
        match init_response {
            Ok(_) => panic!("expected error, but init_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "contract_name")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn create_ask_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask data
        let create_ask_msg = HandleMsg::CreateAsk {
            id: "ask_id".into(),
            price: coins(100, "price_1"),
        };

        let asker_info = mock_info("asker", &coins(2, "asset_1"));

        // handle create ask
        let create_ask_response = handle(
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
        if let HandleMsg::CreateAsk { id, price } = create_ask_msg {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrder {
                            asset: asker_info.sent_funds,
                            id,
                            owner: asker_info.sender,
                            price,
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
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask invalid data
        let create_ask_msg = HandleMsg::CreateAsk {
            id: "".into(),
            price: vec![],
        };

        // handle create ask
        let create_ask_response = handle(
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
        let create_ask_msg = HandleMsg::CreateAsk {
            id: "".into(),
            price: coins(100, "price_1"),
        };

        // handle create ask
        let create_ask_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "asset_1")),
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

        // create ask missing price
        let create_ask_msg = HandleMsg::CreateAsk {
            id: "id".into(),
            price: vec![],
        };

        // handle create ask
        let create_ask_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(100, "asset_1")),
            create_ask_msg,
        );

        // verify handle create ask response returns ContractError::MissingField { price }
        match create_ask_response {
            Ok(_) => panic!("expected error, but handle_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "price")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create ask missing asset
        let create_ask_msg = HandleMsg::CreateAsk {
            id: "id".into(),
            price: coins(100, "price_1"),
        };

        // handle create ask
        let create_ask_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        // verify handle create ask response returns ContractError::AskMissingAsset
        match create_ask_response {
            Ok(_) => panic!("expected error, but handle_create_ask_response ok"),
            Err(error) => match error {
                ContractError::MissingAskAsset {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn create_bid_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid data
        let create_bid_msg = HandleMsg::CreateBid {
            id: "bid_id".into(),
            asset: coins(100, "asset_1"),
        };

        let bidder_info = mock_info("bidder", &coins(2, "mark_2"));

        // handle create bid
        let create_bid_response = handle(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            create_bid_msg.clone(),
        );

        // verify handle create bid response
        match create_bid_response {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 1);
                assert_eq!(response.attributes[0], attr("action", "create_bid"));
            }
            Err(error) => {
                panic!("failed to create bid: {:?}", error)
            }
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read(&deps.storage);
        if let HandleMsg::CreateBid { id, asset } = create_bid_msg {
            match bid_storage.load("bid_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrder {
                            asset,
                            id,
                            owner: bidder_info.sender,
                            price: bidder_info.sent_funds,
                        }
                    )
                }
                _ => {
                    panic!("bid order was not found in storage")
                }
            }
        } else {
            panic!("bid_message is not a CreateBid type. this is bad.")
        }
    }

    #[test]
    fn create_bid_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid missing id
        let create_bid_msg = HandleMsg::CreateBid {
            id: "".into(),
            asset: coins(100, "asset_1"),
        };

        // handle create bid
        let create_bid_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "price_1")),
            create_bid_msg,
        );

        // verify handle create bid response returns ContractError::MissingField { id }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing asset
        let create_bid_msg = HandleMsg::CreateBid {
            id: "id".into(),
            asset: vec![],
        };

        // handle create bid
        let create_bid_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "price_1")),
            create_bid_msg,
        );

        // verify handle create bid response returns ContractError::MissingField { asset }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "asset")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing price
        let create_bid_msg = HandleMsg::CreateBid {
            id: "id".into(),
            asset: coins(100, "asset_1"),
        };

        // handle create bid
        let create_bid_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &[]),
            create_bid_msg,
        );

        // verify handle create bid response returns ContractError::BidMissingPrice
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingBidPrice {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn cancel_with_valid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create ask data
        let asker_info = mock_info("asker", &coins(200, "asset_1"));

        let create_ask_msg = HandleMsg::CreateAsk {
            id: "ask_id".into(),
            price: coins(100, "price_1"),
        };

        // handle create ask
        if let Err(error) = handle(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read(&deps.storage);
        assert_eq!(
            ask_storage.load("ask_id".to_string().as_bytes()).is_ok(),
            true
        );

        // cancel ask order
        let asker_info = mock_info("asker", &[]);

        let cancel_ask_msg = HandleMsg::CancelAsk {
            id: "ask_id".to_string(),
        };
        let cancel_ask_response = handle(
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
                    cancel_ask_response.messages[0],
                    CosmosMsg::Bank(BankMsg::Send {
                        from_address: MOCK_CONTRACT_ADDR.into(),
                        to_address: asker_info.sender,
                        amount: coins(200, "asset_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify ask order removed from storage
        let ask_storage = get_ask_storage_read(&deps.storage);
        assert_eq!(
            ask_storage.load("ask_id".to_string().as_bytes()).is_err(),
            true
        );

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "price_1"));
        let create_bid_msg = HandleMsg::CreateBid {
            id: "bid_id".into(),
            asset: vec![Coin {
                denom: "asset_1".into(),
                amount: Uint128(200),
            }],
        };

        // handle create bid
        if let Err(error) = handle(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read(&deps.storage);
        assert_eq!(
            bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),
            true
        );

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = HandleMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = handle(
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
                    cancel_bid_response.messages[0],
                    CosmosMsg::Bank(BankMsg::Send {
                        from_address: MOCK_CONTRACT_ADDR.into(),
                        to_address: bidder_info.sender,
                        amount: coins(100, "price_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        let bid_storage = get_bid_storage_read(&deps.storage);
        assert_eq!(
            bid_storage.load("bid_id".to_string().as_bytes()).is_err(),
            true
        );
    }

    #[test]
    fn cancel_with_invalid_data() {
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        let asker_info = mock_info("asker", &[]);

        // cancel ask order with missing id returns ContractError::Unauthorized
        let cancel_ask_msg = HandleMsg::CancelAsk { id: "".to_string() };
        let cancel_response = handle(
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
        let cancel_ask_msg = HandleMsg::CancelAsk {
            id: "unknown_id".to_string(),
        };

        let cancel_response = handle(
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
                asset: coins(200, "asset_1"),
                id: "ask_id".into(),
                owner: "".into(),
                price: coins(100, "price_1"),
            },
        ) {
            panic!("unexpected error: {:?}", error)
        };
        let cancel_ask_msg = HandleMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = handle(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

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
        let cancel_ask_msg = HandleMsg::CancelAsk {
            id: "ask_id".to_string(),
        };

        let cancel_response = handle(deps.as_mut(), mock_env(), asker_info, cancel_ask_msg);

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

    #[test]
    fn execute_with_valid_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrder {
            asset: vec![coin(100, "asset_1"), coin(200, "asset_2")],
            id: "ask_id".into(),
            owner: HumanAddr("asker".into()),
            price: coins(200, "price_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(&ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrder {
            asset: vec![coin(200, "asset_2"), coin(100, "asset_1")],
            id: "bid_id".to_string(),
            owner: HumanAddr("bidder".into()),
            price: coins(200, "price_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
        if let Err(error) = bid_storage.save(&bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on matched ask order and bid order
        let execute_msg = HandleMsg::Execute {
            ask_id: ask_order.id,
            bid_id: bid_order.id,
        };

        let execute_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        // validate execute response
        match execute_response {
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(execute_response) => {
                assert_eq!(execute_response.attributes.len(), 1);
                assert_eq!(execute_response.attributes[0], attr("action", "execute"));
                assert_eq!(execute_response.messages.len(), 2);
                assert_eq!(
                    execute_response.messages[0],
                    CosmosMsg::Bank(BankMsg::Send {
                        from_address: MOCK_CONTRACT_ADDR.into(),
                        to_address: ask_order.owner,
                        amount: ask_order.price,
                    })
                );
                assert_eq!(
                    execute_response.messages[1],
                    CosmosMsg::Bank(BankMsg::Send {
                        from_address: MOCK_CONTRACT_ADDR.into(),
                        to_address: bid_order.owner,
                        amount: bid_order.asset,
                    })
                );
            }
        }
    }

    #[test]
    fn execute_with_invalid_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrder {
            asset: coins(200, "asset_1"),
            id: "ask_id".into(),
            owner: HumanAddr("asker".into()),
            price: coins(100, "price_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(&ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrder {
            asset: coins(100, "asset_1"),
            id: "bid_id".into(),
            owner: HumanAddr("bidder".into()),
            price: coins(100, "price_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
        if let Err(error) = bid_storage.save(&bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on mismatched ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = HandleMsg::Execute {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = HandleMsg::Execute {
            ask_id: "no_ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = HandleMsg::Execute {
            ask_id: "ask_id".into(),
            bid_id: "no_bid_id".into(),
        };

        let execute_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::AskBidMismatch {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute with sent_funds returns ContractError::ExecuteWithFunds
        let execute_msg = HandleMsg::Execute {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = handle(
            deps.as_mut(),
            mock_env(),
            mock_info("contract_admin", &coins(100, "funds")),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::ExecuteWithFunds {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }
    }

    #[test]
    pub fn query_with_valid_data() {
        // setup
        let mut deps = mock_dependencies(&[]);
        if let Err(error) = set_contract_info(
            &mut deps.storage,
            HumanAddr("contract_admin".into()),
            "contract_bind_name",
            "contract_name",
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrder {
            asset: coins(200, "asset_1"),
            id: "ask_id".into(),
            owner: HumanAddr("asker".into()),
            price: coins(100, "price_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(&ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrder {
            asset: coins(100, "asset_1"),
            id: "bid_id".into(),
            owner: HumanAddr("bidder".into()),
            price: coins(100, "price_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
        if let Err(error) = bid_storage.save(&bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // query for contract_info
        let query_contract_info_response =
            query(deps.as_ref(), mock_env(), QueryMsg::GetContractInfo);

        match query_contract_info_response {
            Ok(contract_info) => {
                assert_eq!(
                    contract_info,
                    to_binary(&get_contract_info(&deps.storage).unwrap()).unwrap()
                )
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // query for ask order
        let query_ask_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAsk {
                id: ask_order.id.clone(),
            },
        );

        assert_eq!(query_ask_response, to_binary(&ask_order));

        // query for bid order
        let query_bid_response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetBid {
                id: bid_order.id.clone(),
            },
        );

        assert_eq!(query_bid_response, to_binary(&bid_order));
    }
}
