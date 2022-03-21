use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Timestamp,
};
use provwasm_std::{
    bind_name, write_scope, NameBinding, Party, PartyType, ProvenanceMsg, ProvenanceQuerier,
    ProvenanceQuery, Scope,
};

use crate::contract_info::{get_contract_info, set_contract_info, ContractInfo};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    get_ask_storage_read_v2, get_ask_storage_v2, get_bid_storage_read_v2, get_bid_storage_v2,
    AskOrderV2, BaseType, BidOrderV2,
};

// smart contract initialization entrypoint
#[entry_point]
pub fn instantiate(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
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
    let contract_info = ContractInfo::new(info.sender, msg.bind_name, msg.contract_name);
    set_contract_info(deps.storage, &contract_info)?;

    // create name binding provenance message
    let bind_name_msg = bind_name(
        contract_info.bind_name,
        env.contract.address,
        NameBinding::Restricted,
    )?;

    // build response
    Ok(Response::new()
        .add_messages(vec![bind_name_msg])
        .add_attributes(vec![
            attr(
                "contract_info",
                format!("{:?}", get_contract_info(deps.storage)?),
            ),
            attr("action", "init"),
        ]))
}

// smart contract execute entrypoint
#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address,
        } => create_ask(deps, env, info, id, quote, scope_address),
        ExecuteMsg::CreateBid {
            id,
            base,
            effective_time,
        } => create_bid(deps, info, id, base, effective_time),
        ExecuteMsg::CancelAsk { id } => cancel_ask(deps, env, info, id),
        ExecuteMsg::CancelBid { id } => cancel_bid(deps, env, info, id),
        ExecuteMsg::ExecuteMatch { ask_id, bid_id } => {
            execute_match(deps, env, info, ask_id, bid_id)
        }
    }
}

// create ask entrypoint
fn create_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    id: String,
    quote: Vec<Coin>,
    scope_address: Option<String>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if quote.is_empty() {
        return Err(ContractError::MissingField {
            field: "quote".into(),
        });
    }
    let (messages, base) = if let Some(address) = scope_address {
        // can't provide funds when putting in an ask for a scope
        if !info.funds.is_empty() {
            return Err(ContractError::ScopeAskBaseWithFunds);
        }
        let mut scope = ProvenanceQuerier::new(&deps.querier).get_scope(&address)?;
        // due to restrictions on permissioning, the value owner address must be the contract's address prior to invoking this execute path
        if scope.value_owner_address != env.contract.address {
            return Err(ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id,
                explanation: "the contract must be the scope's value owner".to_string(),
            });
        }
        // Remove the owner from the scope and replace it with the contract
        scope = replace_scope_owner(
            scope,
            env.contract.address.clone(),
            false,
            Some(info.sender.clone()),
        )?;
        let scope_write_msg = write_scope(scope, vec![env.contract.address])?;
        (vec![scope_write_msg], BaseType::scope(&address))
    } else {
        if info.funds.is_empty() {
            return Err(ContractError::MissingAskBase);
        }
        (vec![], BaseType::coins(info.funds))
    };

    let mut ask_storage = get_ask_storage_v2(deps.storage);

    let ask_order = AskOrderV2 {
        base,
        id,
        owner: info.sender,
        quote,
    };

    ask_storage.save(ask_order.id.as_bytes(), &ask_order)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "create_ask")])
        .add_messages(messages)
        .set_data(to_binary(&ask_order)?))
}

// create bid entrypoint
fn create_bid(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    id: String,
    base: BaseType,
    effective_time: Option<Timestamp>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if let BaseType::Coin { coins } = &base {
        if coins.is_empty() {
            return Err(ContractError::MissingField {
                field: "base".into(),
            });
        }
    }

    // todo: check for scope existence? Unless there is a reason to allow bidding on a scope
    // that may not exist yet...

    if id.is_empty() {
        return Err(ContractError::MissingField { field: "id".into() });
    }
    if info.funds.is_empty() {
        return Err(ContractError::MissingBidQuote);
    }

    let mut bid_storage = get_bid_storage_v2(deps.storage);

    let bid_order = BidOrderV2 {
        base,
        effective_time,
        id,
        owner: info.sender,
        quote: info.funds,
    };

    bid_storage.save(bid_order.id.as_bytes(), &bid_order)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "create_bid")])
        .set_data(to_binary(&bid_order)?))
}

// cancel ask entrypoint
fn cancel_ask(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
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

    let ask_storage = get_ask_storage_read_v2(deps.storage);
    let stored_ask_order = ask_storage.load(id.as_bytes());
    match stored_ask_order {
        Err(_) => Err(ContractError::Unauthorized {}),
        Ok(stored_ask_order) => {
            if !info.sender.eq(&stored_ask_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut ask_storage = get_ask_storage_v2(deps.storage);
            ask_storage.remove(id.as_bytes());

            let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];

            match stored_ask_order.base {
                BaseType::Coin { coins } => {
                    messages.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
                        to_address: stored_ask_order.owner.to_string(),
                        amount: coins,
                    }));
                }
                BaseType::Scope { scope_address } => {
                    // fetch scope
                    let scope = ProvenanceQuerier::new(&deps.querier).get_scope(scope_address)?;

                    // Set the original asker's address back to being the owner and value owner address
                    messages.push(write_scope(
                        replace_scope_owner(scope, stored_ask_order.owner, true, None)?,
                        vec![env.contract.address],
                    )?);
                }
            };

            // 'send base back to owner' message
            Ok(Response::new()
                .add_messages(messages)
                .add_attributes(vec![attr("action", "cancel_ask")]))
        }
    }
}

// cancel bid entrypoint
fn cancel_bid(
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

    let bid_storage = get_bid_storage_read_v2(deps.storage);
    let stored_bid_order = bid_storage.load(id.as_bytes());
    match stored_bid_order {
        Ok(stored_bid_order) => {
            if !info.sender.eq(&stored_bid_order.owner) {
                return Err(ContractError::Unauthorized {});
            }

            // remove the ask order from storage
            let mut bid_storage = get_bid_storage_v2(deps.storage);
            bid_storage.remove(id.as_bytes());

            // 'send quote back to owner' message
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: stored_bid_order.owner.to_string(),
                    amount: stored_bid_order.quote,
                })
                .add_attributes(vec![attr("action", "cancel_bid")]))
        }
        Err(_) => Err(ContractError::Unauthorized {}),
    }
}

// match and execute an ask and bid order
fn execute_match(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    ask_id: String,
    bid_id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // only the admin may execute matches
    if info.sender != get_contract_info(deps.storage)?.admin {
        return Err(ContractError::Unauthorized {});
    }

    // return error if id is empty
    if ask_id.is_empty() | bid_id.is_empty() {
        return Err(ContractError::Unauthorized {});
    }

    // return error if funds sent
    if !info.funds.is_empty() {
        return Err(ContractError::ExecuteWithFunds {});
    }

    let ask_storage_read = get_ask_storage_read_v2(deps.storage);
    let ask_order_result = ask_storage_read.load(ask_id.as_bytes());
    if ask_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let bid_storage_read = get_bid_storage_read_v2(deps.storage);
    let bid_order_result = bid_storage_read.load(bid_id.as_bytes());
    if bid_order_result.is_err() {
        return Err(ContractError::AskBidMismatch {});
    }

    let ask_order = ask_order_result.unwrap();
    let bid_order = bid_order_result.unwrap();

    if !is_executable(&ask_order, &bid_order) {
        return Err(ContractError::AskBidMismatch {});
    }

    // 'send quote to asker' and 'send base to bidder' messages
    let response = Response::new().add_message(BankMsg::Send {
        to_address: ask_order.owner.to_string(),
        amount: ask_order.quote,
    });
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];

    match bid_order.base {
        BaseType::Coin { coins } => messages.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: coins,
        })),
        BaseType::Scope { scope_address } => {
            // fetch scope
            let scope = ProvenanceQuerier::new(&deps.querier).get_scope(scope_address)?;

            messages.push(write_scope(
                // TODO: Use:
                // replace_scope_owner(scope, bid_order.owner, true, None)?,
                Scope {
                    owners: vec![Party {
                        address: bid_order.owner,
                        role: PartyType::Owner,
                    }],
                    ..scope
                },
                vec![env.contract.address],
            )?)
        }
    };

    // finally remove the orders from storage
    get_ask_storage_v2(deps.storage).remove(ask_id.as_bytes());
    get_bid_storage_v2(deps.storage).remove(bid_id.as_bytes());

    Ok(response
        .add_messages(messages)
        .add_attributes(vec![attr("action", "execute")]))
}

fn is_executable(ask_order: &AskOrderV2, bid_order: &BidOrderV2) -> bool {
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));

    let ask_base = ask_order.base.to_owned().sorted();
    let bid_base = bid_order.base.to_owned().sorted();

    let mut ask_quote = ask_order.quote.to_owned();
    ask_quote.sort_by(coin_sorter);
    let mut bid_quote = bid_order.quote.to_owned();
    bid_quote.sort_by(coin_sorter);

    ask_base == bid_base && ask_quote == bid_quote
}

/// Switches the scope's current owner value to the given owner value.
/// If replace_value_owner is specified, the new_owner value will also be used as the value owner.
/// If validate_expected_owner is given an address, the owner's current address will be verified against it.
fn replace_scope_owner(
    mut scope: Scope,
    new_owner: Addr,
    replace_value_owner: bool,
    validate_expected_owner: Option<Addr>,
) -> Result<Scope, ContractError> {
    let owners = scope
        .owners
        .iter()
        .filter(|owner| owner.role == PartyType::Owner)
        .collect::<Vec<&Party>>();
    // if more than one owner is specified, removing all of them can potentially cause data loss
    if owners.len() != 1 {
        return Err(ContractError::InvalidScopeOwner {
            scope_address: scope.scope_id,
            explanation: format!(
                "the scope should only include a single owner, but found: {}",
                owners.len(),
            ),
        });
    }
    // The owner of the scope must be the sender to the contract - otherwise, scopes could be released on the behalf of others
    if let Some(expected_owner) = validate_expected_owner {
        let owner = owners.first().unwrap();
        if owner.address != expected_owner {
            return Err(ContractError::InvalidScopeOwner {
                scope_address: scope.scope_id,
                explanation: format!(
                    "the scope owner was expected to be [{}], not [{}]",
                    expected_owner, owner.address,
                ),
            });
        }
    }
    // Empty out all owners from the scope now that it's verified safe to do
    scope.owners = scope
        .owners
        .into_iter()
        .filter(|owner| owner.role != PartyType::Owner)
        .collect();
    // Append the target value as the new sole owner
    scope.owners.push(Party {
        address: new_owner.clone(),
        role: PartyType::Owner,
    });
    // Upon request, also swap over the value owner.  The contract cannot transfer value ownership to itself
    // during contract execution, so this functionality should only be used when the contract is already the
    // value owner of a scope and wants to swap it to a different owner
    if replace_value_owner {
        scope.value_owner_address = new_owner;
    }
    Ok(scope)
}

// smart contract query entrypoint
#[entry_point]
pub fn query(deps: Deps<ProvenanceQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAsk { id } => {
            let ask_storage_read = get_ask_storage_read_v2(deps.storage);
            return to_binary(&ask_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetBid { id } => {
            let bid_storage_read = get_bid_storage_read_v2(deps.storage);
            return to_binary(&bid_storage_read.load(id.as_bytes())?);
        }
        QueryMsg::GetContractInfo {} => to_binary(&get_contract_info(deps.storage)?),
    }
}

// unit tests
#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, Addr, BankMsg};
    use cosmwasm_std::{CosmosMsg, Uint128};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{NameMsgParams, ProvenanceMsg, ProvenanceMsgParams, ProvenanceRoute};

    use crate::contract_info::{ContractInfo, CONTRACT_TYPE, CONTRACT_VERSION};
    use crate::state::{get_bid_storage_read_v2, BaseType};

    use super::*;
    use crate::msg::ExecuteMsg;

    #[test]
    fn test_is_executable() {
        assert!(is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(is_executable(
            &AskOrderV2 {
                base: BaseType::coins(vec![coin(100, "base_1"), coin(200, "base_2")]),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coins(vec![coin(200, "base_2"), coin(100, "base_1")]),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_2"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrderV2 {
                base: BaseType::coin(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrderV2 {
                base: BaseType::coin(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_2"),
            }
        ));
    }

    #[test]
    fn instantiate_with_valid_data() {
        // create valid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_admin", &[]);
        let init_msg = InstantiateMsg {
            bind_name: "contract_bind_name".to_string(),
            contract_name: "contract_name".to_string(),
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info, init_msg.clone());

        // verify initialize response
        match init_response {
            Ok(init_response) => {
                assert_eq!(init_response.messages.len(), 1);
                assert_eq!(
                    init_response.messages[0].msg,
                    CosmosMsg::Custom(ProvenanceMsg {
                        route: ProvenanceRoute::Name,
                        params: ProvenanceMsgParams::Name(NameMsgParams::BindName {
                            name: init_msg.bind_name,
                            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                            restrict: true
                        }),
                        version: "2.0.0".to_string(),
                    })
                );
                let expected_contract_info = ContractInfo {
                    admin: Addr::unchecked("contract_admin"),
                    bind_name: "contract_bind_name".to_string(),
                    contract_name: "contract_name".to_string(),
                    contract_type: CONTRACT_TYPE.into(),
                    contract_version: CONTRACT_VERSION.into(),
                };

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
    fn instantiate_with_invalid_data() {
        // create invalid init data
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_owner", &[]);
        let init_msg = InstantiateMsg {
            bind_name: "".to_string(),
            contract_name: "contract_name".to_string(),
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info.to_owned(), init_msg);

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

        let init_msg = InstantiateMsg {
            bind_name: "bind_name".to_string(),
            contract_name: "".to_string(),
        };

        // initialize
        let init_response = instantiate(deps.as_mut(), mock_env(), info, init_msg);

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
    fn create_ask_for_coin_with_valid_data() {
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
            scope_address: None,
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
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address: None,
        } = create_ask_msg
        {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrderV2 {
                            base: BaseType::coins(asker_info.funds),
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
    fn test_create_ask_for_scope_with_valid_data() {
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

        let scope_address = "scope1qraczfp249d3rmysdurne8cxrwmqamu8tk".to_string();

        // create ask data
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some(scope_address.clone()),
        };

        let asker_info = mock_info("asker", &[]);

        deps.querier.with_scope(Scope {
            scope_id: scope_address.clone(),
            specification_id: "scopespec1qs0lctxj49wprm9xwxt5wk0paswqzkdaax".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("asker"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

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
                assert_eq!(response.messages.len(), 1);
                // TODO: Uncomment this set of checks once the values are exposed in provwasm for it to compile
                // match response.messages.first().unwrap().msg {
                //     CosmosMsg::Custom(ProvenanceMsg {
                //         params:
                //             ProvenanceMsgParams::Metadata(
                //                 provwasm_std::msg::MetadataMsgParams::WriteScope { scope, signers },
                //             ),
                //         ..
                //     }) => {
                //         assert_eq!(
                //             1,
                //             scope.owners.len(),
                //             "expected the scope to only include one owner after the owner was changed to the contract",
                //         );
                //         let scope_owner = scope.owners.first().unwrap();
                //         assert_eq!(
                //             MOCK_CONTRACT_ADDR,
                //             scope_owner.address.as_str(),
                //             "expected the contract address to be set as the scope owner",
                //         );
                //         assert_eq!(
                //             PartyType::Owner,
                //             scope_owner.role,
                //             "expected the contract's role to be that of owner",
                //         );
                //         assert_eq!(
                //             MOCK_CONTRACT_ADDR,
                //             scope.value_owner_address.as_str(),
                //             "expected the contract to remain the value owner on the scope",
                //         );
                //         assert_eq!(
                //             1,
                //             signers.len(),
                //             "expected only a single signer to be used on the write scope request",
                //         );
                //         assert_eq!(
                //             MOCK_CONTRACT_ADDR,
                //             signers.first().unwrap().as_str(),
                //             "expected the signer for the write scope request to be the contract",
                //         );
                //     }
                //     msg => panic!("unexpected message emitted by create ask: {:?}", msg),
                // };
            }
            Err(error) => {
                panic!("failed to create ask: {:?}", error)
            }
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateAsk {
            id,
            quote,
            scope_address,
        } = create_ask_msg
        {
            match ask_storage.load("ask_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        AskOrderV2 {
                            base: BaseType::scope(scope_address.unwrap()),
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
            scope_address: None,
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
            scope_address: None,
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
            scope_address: None,
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
            scope_address: None,
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
        };

        // create ask with scope and funds provided
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope-address".to_string()),
        };

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &coins(150, "fakecoin")),
            create_ask_msg,
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::ScopeAskBaseWithFunds => {}
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with incorrect value owner address
        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope_address".to_string()),
        };

        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![],
            data_access: vec![],
            value_owner_address: Addr::unchecked("not_contract_address"),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg.clone(),
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the contract must be the scope's value owner", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with multiple owners specified - re-using previous ask msg
        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![
                Party {
                    address: Addr::unchecked("asker"),
                    role: PartyType::Owner,
                },
                Party {
                    address: Addr::unchecked("other-guy"),
                    role: PartyType::Owner,
                },
            ],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg.clone(),
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the scope should only include a single owner, but found: 2", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };

        // create ask with scope provided with incorrect single owner specified - re-using previous ask msg
        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("not-asker"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        let create_ask_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("asker", &[]),
            create_ask_msg,
        );

        match create_ask_response {
            Ok(_) => panic!("expected error, but execute_create_ask_response ok"),
            Err(error) => match error {
                ContractError::InvalidScopeOwner {
                    scope_address,
                    explanation,
                } => {
                    assert_eq!(
                        "scope_address", scope_address,
                        "the proper scope address should be found",
                    );
                    assert_eq!(
                        "the scope owner was expected to be [asker], not [not-asker]", explanation,
                        "the proper explanation must be used in the InvalidScopeOwner error",
                    );
                }
                error => panic!("unexpected error: {:?}", error),
            },
        };
    }

    #[test]
    fn create_bid_with_valid_data() {
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

        // create bid data
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        let bidder_info = mock_info("bidder", &coins(2, "mark_2"));

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            bidder_info.clone(),
            create_bid_msg.clone(),
        );

        // verify execute create bid response
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
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        if let ExecuteMsg::CreateBid {
            id,
            base,
            effective_time,
        } = create_bid_msg
        {
            match bid_storage.load("bid_id".to_string().as_bytes()) {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrderV2 {
                            base,
                            effective_time,
                            id,
                            owner: bidder_info.sender,
                            quote: bidder_info.funds,
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
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // create bid missing id
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "quote_1")),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::MissingField { id }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "id")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing base
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "id".into(),
            base: BaseType::coins(vec![]),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &coins(100, "quote_1")),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::MissingField { base }
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingField { field } => {
                    assert_eq!(field, "base")
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // create bid missing quote
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "id".into(),
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &[]),
            create_bid_msg,
        );

        // verify execute create bid response returns ContractError::BidMissingQuote
        match create_bid_response {
            Ok(_) => panic!("expected error, but create_bid_response ok"),
            Err(error) => match error {
                ContractError::MissingBidQuote {} => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn cancel_coin_with_valid_data() {
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
            scope_address: None,
        };

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
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
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_err());

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "quote_1"));
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::coins(vec![Coin {
                denom: "base_1".into(),
                amount: Uint128::new(200),
            }]),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),);

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = ExecuteMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = execute(
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
                    cancel_bid_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bidder_info.sender.to_string(),
                        amount: coins(100, "quote_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_err());
    }

    #[test]
    fn cancel_scope_with_valid_data() {
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

        // create ask data - omit funds because a scope is being provided
        let asker_info = mock_info("asker", &[]);

        let create_ask_msg = ExecuteMsg::CreateAsk {
            id: "ask_id".into(),
            quote: coins(100, "quote_1"),
            scope_address: Some("scope_address".to_string()),
        };

        deps.querier.with_scope(Scope {
            scope_id: "scope_address".to_string(),
            specification_id: "spec_address".to_string(),
            owners: vec![Party {
                address: Addr::unchecked("asker"),
                role: PartyType::Owner,
            }],
            data_access: vec![],
            value_owner_address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        });

        // execute create ask
        if let Err(error) = execute(deps.as_mut(), mock_env(), asker_info, create_ask_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify ask order stored
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
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
                // TODO: Uncomment this set of checks once the values are exposed in provwasm for it to compile
                // match cancel_ask_response.messages.first().unwrap().msg {
                //     CosmosMsg::Custom(ProvenanceMsg {
                //         params:
                //             ProvenanceMsgParams::Metadata(
                //                 provwasm_std::msg::MetadataMsgParams::WriteScope { scope, signers },
                //             ),
                //         ..
                //     }) => {
                //         assert_eq!(
                //             1,
                //             scope.owners.len(),
                //             "expected the scope to only include one owner after the owner is swapped back to the original value",
                //         );
                //         let scope_owner = scope.owners.first().unwrap();
                //         assert_eq!(
                //             "asker",
                //             scope_owner.address.as_str(),
                //             "expected the asker address to be set as the scope owner",
                //         );
                //         assert_eq!(
                //             PartyType::Owner,
                //             scope_owner.role,
                //             "expected the asker's role to be that of owner",
                //         );
                //         assert_eq!(
                //             "asker",
                //             scope.value_owner_address.as_str(),
                //             "expected the asker to be set as the value owner after a cancellation",
                //         );
                //         assert_eq!(
                //             1,
                //             signers.len(),
                //             "expected only a single signer to be used on the write scope request",
                //         );
                //         assert_eq!(
                //             MOCK_CONTRACT_ADDR,
                //             signers.first().unwrap().as_str(),
                //             "expected the signer for the write scope request to be the contract",
                //         );
                //     }
                //     msg => panic!("unexpected message emitted by cancel ask: {:?}", msg),
                // };
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify ask order removed from storage
        let ask_storage = get_ask_storage_read_v2(&deps.storage);
        assert!(ask_storage.load("ask_id".to_string().as_bytes()).is_err());

        // create bid data
        let bidder_info = mock_info("bidder", &coins(100, "quote_1"));
        let create_bid_msg = ExecuteMsg::CreateBid {
            id: "bid_id".into(),
            base: BaseType::scope("scope_address"),
            effective_time: Some(Timestamp::default()),
        };

        // execute create bid
        if let Err(error) = execute(deps.as_mut(), mock_env(), bidder_info, create_bid_msg) {
            panic!("unexpected error: {:?}", error)
        }

        // verify bid order stored
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_ok(),);

        // cancel bid order
        let bidder_info = mock_info("bidder", &[]);

        let cancel_bid_msg = ExecuteMsg::CancelBid {
            id: "bid_id".to_string(),
        };

        let cancel_bid_response = execute(
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
                    cancel_bid_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bidder_info.sender.to_string(),
                        amount: coins(100, "quote_1"),
                    })
                );
            }
            Err(error) => panic!("unexpected error: {:?}", error),
        }

        // verify bid order removed from storage
        let bid_storage = get_bid_storage_read_v2(&deps.storage);
        assert!(bid_storage.load("bid_id".to_string().as_bytes()).is_err());
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
        if let Err(error) = get_ask_storage_v2(&mut deps.storage).save(
            "ask_id".to_string().as_bytes(),
            &AskOrderV2 {
                base: BaseType::coin(200, "base_1"),
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

    #[test]
    fn execute_with_valid_data() {
        // setup
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

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coins(vec![coin(100, "base_1"), coin(200, "base_2")]),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(200, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coins(vec![coin(200, "base_2"), coin(100, "base_1")]),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".to_string(),
            owner: Addr::unchecked("bidder"),
            quote: coins(200, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on matched ask order and bid order
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: ask_order.id,
            bid_id: bid_order.id.clone(),
        };

        let execute_response = execute(
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
                    execute_response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: ask_order.owner.to_string(),
                        amount: ask_order.quote,
                    })
                );
                handle_expected_coin(&bid_order.base, |coins| {
                    assert_eq!(
                        execute_response.messages[1].msg,
                        CosmosMsg::Bank(BankMsg::Send {
                            to_address: bid_order.owner.to_string(),
                            amount: coins.to_vec(),
                        })
                    );
                });
            }
        }
    }

    #[test]
    fn execute_with_invalid_data() {
        // setup
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

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coin(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute by non-admin ContractError::Unauthorized
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            execute_msg,
        );

        match execute_response {
            Err(ContractError::Unauthorized {}) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }

        // execute on mismatched ask order and bid order returns ContractError::AskBidMismatch
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
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
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "no_ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
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
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "no_bid_id".into(),
        };

        let execute_response = execute(
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
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: "ask_id".into(),
            bid_id: "bid_id".into(),
        };

        let execute_response = execute(
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
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrderV2 {
            base: BaseType::coin(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage_v2(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrderV2 {
            base: BaseType::coin(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage_v2(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // query for contract_info
        let query_contract_info_response =
            query(deps.as_ref(), mock_env(), QueryMsg::GetContractInfo {});

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

    fn handle_expected_coin<A: FnOnce(&Vec<Coin>) -> ()>(base_type: &BaseType, action: A) {
        match base_type {
            BaseType::Coin { coins } => action(coins),
            _ => panic!("Unexpected base type of scope"),
        }
    }
}
