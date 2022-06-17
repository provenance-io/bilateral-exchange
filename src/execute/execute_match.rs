use crate::storage::ask_order_storage::{delete_ask_order_by_id, get_ask_order_by_id};
use crate::storage::bid_order_storage::{delete_bid_order_by_id, get_bid_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::ask_collateral::{
    AskCollateral, CoinTradeAskCollateral, MarkerShareSaleAskCollateral, MarkerTradeAskCollateral,
    ScopeTradeAskCollateral,
};
use crate::types::ask_order::AskOrder;
use crate::types::bid_collateral::{
    CoinTradeBidCollateral, MarkerShareSaleBidCollateral, MarkerTradeBidCollateral,
    ScopeTradeBidCollateral,
};
use crate::types::bid_order::BidOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::validation::execute_match_validation::validate_match;
use cosmwasm_std::{BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{grant_marker_access, revoke_marker_access, ProvenanceMsg, ProvenanceQuery};

// match and execute an ask and bid order
pub fn execute_match(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    ask_id: String,
    bid_id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // only the admin may execute matches
    if info.sender != get_contract_info(deps.storage)?.admin {
        return Err(ContractError::Unauthorized);
    }
    // return error if funds sent
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds should not be provided during match execution".to_string(),
        }
        .to_err();
    }
    let mut invalid_fields: Vec<String> = vec![];
    if ask_id.is_empty() {
        invalid_fields.push("ask id must not be empty".to_string());
    }
    if bid_id.is_empty() {
        invalid_fields.push("bid id must not be empty".to_string());
    }
    // return error if either ids are badly formed
    if !invalid_fields.is_empty() {
        return ContractError::ValidationError {
            messages: invalid_fields,
        }
        .to_err();
    }

    let ask_order = get_ask_order_by_id(deps.storage, ask_id)?;
    let bid_order = get_bid_order_by_id(deps.storage, bid_id)?;

    validate_match(&deps, &ask_order, &bid_order)?;

    let execute_result = match &ask_order.collateral {
        AskCollateral::CoinTrade(collateral) => execute_coin_trade(
            deps,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_coin_trade()?,
        )?,
        AskCollateral::MarkerTrade(collateral) => execute_marker_trade(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_marker_trade()?,
        )?,
        AskCollateral::MarkerShareSale(collateral) => execute_marker_share_sale(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_marker_share_sale()?,
        )?,
        AskCollateral::ScopeTrade(collateral) => execute_scope_trade(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_scope_trade()?,
        )?,
    };

    Response::new()
        .add_messages(execute_result.messages)
        .add_attribute("action", "execute")
        .add_attribute("ask_id", &ask_order.id)
        .add_attribute("bid_id", &bid_order.id)
        .to_ok()
}

struct ExecuteResults {
    pub messages: Vec<CosmosMsg<ProvenanceMsg>>,
}
impl ExecuteResults {
    fn new(messages: Vec<CosmosMsg<ProvenanceMsg>>) -> Self {
        Self { messages }
    }
}

fn execute_coin_trade(
    deps: DepsMut<ProvenanceQuery>,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &CoinTradeAskCollateral,
    bid_collateral: &CoinTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    // Remove ask and bid - this transaction has concluded
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: ask_order.owner.to_string(),
            amount: ask_collateral.quote.to_owned(),
        }),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: bid_collateral.base.to_owned(),
        }),
    ])
    .to_ok()
}

fn execute_marker_trade(
    deps: DepsMut<ProvenanceQuery>,
    env: &Env,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &MarkerTradeAskCollateral,
    bid_collateral: &MarkerTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    // Now that the match has been made, grant all permissions on the marker to the bidder that
    // the asker once had.  The validation code has already ensured that the asker was an admin
    // of the marker, so the bidder at very least has the permission on the marker to grant
    // themselves any remaining permissions they desire.
    let mut messages = vec![];
    if let Some(asker_permissions) = ask_collateral
        .removed_permissions
        .iter()
        .find(|perm| perm.address == ask_order.owner)
    {
        messages.push(grant_marker_access(
            &ask_collateral.denom,
            bid_order.owner.clone(),
            asker_permissions.permissions.to_owned(),
        )?);
    } else {
        return ContractError::validation_error(&[
            "failed to find access permissions in the revoked permissions for the asker"
                .to_string(),
        ])
        .to_err();
    }
    // Send the entirety of the quote to the asker. They have just effectively sold their
    // marker to the bidder.
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: ask_order.owner.to_string(),
        amount: bid_collateral.quote.to_owned(),
    }));
    messages.push(revoke_marker_access(
        &bid_collateral.denom,
        env.contract.address.to_owned(),
    )?);
    // Remove ask and bid - this transaction has concluded
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(messages).to_ok()
}

fn execute_marker_share_sale(
    _deps: DepsMut<ProvenanceQuery>,
    _env: &Env,
    _ask_order: &AskOrder,
    _bid_order: &BidOrder,
    _ask_collateral: &MarkerShareSaleAskCollateral,
    _bid_collateral: &MarkerShareSaleBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    ContractError::unauthorized().to_err()
}

fn execute_scope_trade(
    _deps: DepsMut<ProvenanceQuery>,
    _env: &Env,
    _ask_order: &AskOrder,
    _bid_order: &BidOrder,
    _ask_collateral: &ScopeTradeAskCollateral,
    _bid_collateral: &ScopeTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    ContractError::unauthorized().to_err()
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn execute_with_valid_data() {
    //     // setup
    //     let mut deps = mock_dependencies(&[]);
    //     if let Err(error) = set_contract_info(
    //         &mut deps.storage,
    //         &ContractInfo::new(
    //             Addr::unchecked("contract_admin"),
    //             "contract_bind_name".into(),
    //             "contract_name".into(),
    //         ),
    //     ) {
    //         panic!("unexpected error: {:?}", error)
    //     }
    //
    //     // store valid ask order
    //     let ask_order = AskOrder {
    //         base: vec![coin(100, "base_1"), coin(200, "base_2")],
    //         id: "ask_id".into(),
    //         owner: Addr::unchecked("asker"),
    //         quote: coins(200, "quote_1"),
    //     };
    //
    //     let mut ask_storage = get_ask_storage(&mut deps.storage);
    //     if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
    //         panic!("unexpected error: {:?}", error)
    //     };
    //
    //     // store valid bid order
    //     let bid_order = BidOrder {
    //         base: vec![coin(200, "base_2"), coin(100, "base_1")],
    //         effective_time: Some(Timestamp::default()),
    //         id: "bid_id".to_string(),
    //         owner: Addr::unchecked("bidder"),
    //         quote: coins(200, "quote_1"),
    //     };
    //
    //     let mut bid_storage = get_bid_storage(&mut deps.storage);
    //     if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
    //         panic!("unexpected error: {:?}", error);
    //     };
    //
    //     // execute on matched ask order and bid order
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: ask_order.id,
    //         bid_id: bid_order.id,
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("contract_admin", &[]),
    //         execute_msg,
    //     );
    //
    //     // validate execute response
    //     match execute_response {
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(execute_response) => {
    //             assert_eq!(execute_response.attributes.len(), 1);
    //             assert_eq!(execute_response.attributes[0], attr("action", "execute"));
    //             assert_eq!(execute_response.messages.len(), 2);
    //             assert_eq!(
    //                 execute_response.messages[0].msg,
    //                 CosmosMsg::Bank(BankMsg::Send {
    //                     to_address: ask_order.owner.to_string(),
    //                     amount: ask_order.quote,
    //                 })
    //             );
    //             assert_eq!(
    //                 execute_response.messages[1].msg,
    //                 CosmosMsg::Bank(BankMsg::Send {
    //                     to_address: bid_order.owner.to_string(),
    //                     amount: bid_order.base,
    //                 })
    //             );
    //         }
    //     }
    // }
    //
    // #[test]
    // fn execute_with_invalid_data() {
    //     // setup
    //     let mut deps = mock_dependencies(&[]);
    //     if let Err(error) = set_contract_info(
    //         &mut deps.storage,
    //         &ContractInfo::new(
    //             Addr::unchecked("contract_admin"),
    //             "contract_bind_name".into(),
    //             "contract_name".into(),
    //         ),
    //     ) {
    //         panic!("unexpected error: {:?}", error)
    //     }
    //
    //     // store valid ask order
    //     let ask_order = AskOrder {
    //         base: coins(200, "base_1"),
    //         id: "ask_id".into(),
    //         owner: Addr::unchecked("asker"),
    //         quote: coins(100, "quote_1"),
    //     };
    //
    //     let mut ask_storage = get_ask_storage(&mut deps.storage);
    //     if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
    //         panic!("unexpected error: {:?}", error)
    //     };
    //
    //     // store valid bid order
    //     let bid_order = BidOrder {
    //         base: coins(100, "base_1"),
    //         effective_time: Some(Timestamp::default()),
    //         id: "bid_id".into(),
    //         owner: Addr::unchecked("bidder"),
    //         quote: coins(100, "quote_1"),
    //     };
    //
    //     let mut bid_storage = get_bid_storage(&mut deps.storage);
    //     if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
    //         panic!("unexpected error: {:?}", error);
    //     };
    //
    //     // execute by non-admin ContractError::Unauthorized
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: "ask_id".into(),
    //         bid_id: "bid_id".into(),
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("user", &[]),
    //         execute_msg,
    //     );
    //
    //     match execute_response {
    //         Err(ContractError::Unauthorized) => {}
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(_) => panic!("expected error, but execute_response ok"),
    //     }
    //
    //     // execute on mismatched ask order and bid order returns ContractError::AskBidMismatch
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: "ask_id".into(),
    //         bid_id: "bid_id".into(),
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("contract_admin", &[]),
    //         execute_msg,
    //     );
    //
    //     match execute_response {
    //         Err(ContractError::AskBidMismatch) => {}
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(_) => panic!("expected error, but execute_response ok"),
    //     }
    //
    //     // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: "no_ask_id".into(),
    //         bid_id: "bid_id".into(),
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("contract_admin", &[]),
    //         execute_msg,
    //     );
    //
    //     match execute_response {
    //         Err(ContractError::AskBidMismatch) => {}
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(_) => panic!("expected error, but execute_response ok"),
    //     }
    //
    //     // execute on non-existent ask order and bid order returns ContractError::AskBidMismatch
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: "ask_id".into(),
    //         bid_id: "no_bid_id".into(),
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("contract_admin", &[]),
    //         execute_msg,
    //     );
    //
    //     match execute_response {
    //         Err(ContractError::AskBidMismatch) => {}
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(_) => panic!("expected error, but execute_response ok"),
    //     }
    //
    //     // execute with sent_funds returns ContractError::ExecuteWithFunds
    //     let execute_msg = ExecuteMsg::ExecuteMatch {
    //         ask_id: "ask_id".into(),
    //         bid_id: "bid_id".into(),
    //     };
    //
    //     let execute_response = execute(
    //         deps.as_mut(),
    //         mock_env(),
    //         mock_info("contract_admin", &coins(100, "funds")),
    //         execute_msg,
    //     );
    //
    //     match execute_response {
    //         Err(ContractError::ExecuteWithFunds) => {}
    //         Err(error) => panic!("unexpected error: {:?}", error),
    //         Ok(_) => panic!("expected error, but execute_response ok"),
    //     }
    // }
}
