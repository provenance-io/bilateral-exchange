use crate::storage::ask_order::{
    delete_ask_order_by_id, get_ask_order_by_id, AskCollateral, AskOrder,
};
use crate::storage::bid_order::{get_bid_order_by_id, BidCollateral, BidOrder};
use crate::storage::contract_info::get_contract_info;
use crate::types::constants::{ASK_TYPE_COIN, ASK_TYPE_MARKER, BID_TYPE_COIN, BID_TYPE_MARKER};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::get_marker_quote;
use cosmwasm_std::WasmQuery::ContractInfo;
use cosmwasm_std::{attr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{
    grant_marker_access, revoke_marker_access, MarkerAccess, ProvenanceMsg, ProvenanceQuerier,
    ProvenanceQuery,
};

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
        return ContractError::InvalidFields {
            messages: invalid_fields,
        }
        .to_err();
    }

    let ask_order = get_ask_order_by_id(deps.storage, ask_id)?;
    let bid_order = get_bid_order_by_id(deps.storage, bid_id)?;

    if ask_order.get_matching_bid_type() != bid_order.bid_type {
        return ContractError::InvalidFields {
            messages: vec![format!("AskOrder with id [{}] and type [{}] must be matched with bid type [{}], but BidOrder with id [{}] had type [{}]",
            ask_order.id,
            ask_order.ask_type,
            ask_order.get_matching_bid_type(),
            bid_order.id,
            bid_order.bid_type)]
        }.to_err();
    }

    if !is_executable(&deps, &ask_order, &bid_order)? {
        return Err(ContractError::AskBidMismatch);
    }
    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];
    match &ask_order.collateral {
        // Send quote to asker when a coin match is made
        AskCollateral::Coin { quote, .. } => messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: ask_order.owner.to_string(),
            amount: quote.to_owned(),
        })),
        // Now that the match has been made, grant all permissions on the marker to the bidder that
        // the asker once had.  The validation code has already ensured that the asker was an admin
        // of the marker, so the bidder at very least has the permission on the marker to grant
        // themselves any remaining permissions they desire.
        AskCollateral::Marker {
            denom,
            removed_permissions,
            ..
        } => {
            if let Some(asker_permissions) = removed_permissions
                .iter()
                .find(|perm| perm.address == ask_order.owner)
            {
                messages.push(grant_marker_access(
                    denom,
                    bid_order.owner.clone(),
                    asker_permissions.permissions.to_owned(),
                )?);
            } else {
                return ContractError::AskBidMismatch.to_err();
            }
        }
    };
    match &bid_order.collateral {
        // Send base to bidder when a coin match is made
        BidCollateral::Coin { base, .. } => messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: base.to_owned(),
        })),
        BidCollateral::Marker {
            address,
            denom,
            quote,
        } => {
            // Send the entirety of the quote to the asker. They have just effectively sold their
            // marker to the bidder.
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: ask_order.owner.to_string(),
                amount: quote.to_owned(),
            }));
            messages.push(revoke_marker_access(denom, env.contract.address)?);
        }
    }
    // Now that all matching has concluded, we simply need to delete the ask and bid from storage
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_ask_order_by_id(deps.storage, &bid_order.id)?;

    Response::new()
        .add_messages(messages)
        .add_attribute("action", "execute")
        .add_attribute("ask_id", &ask_order.id)
        .add_attribute("bid_id", &bid_order.id)
        .to_ok()
}

fn validate_matching_orders(ask_order: &AskOrder, bid_order: &BidOrder) -> bool {
    ask_order.ask_type == ASK_TYPE_COIN && bid_order.bid_type == BID_TYPE_COIN
}

fn is_executable(
    deps: &DepsMut<ProvenanceQuery>,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
) -> Result<bool, ContractError> {
    match ask_order.ask_type.as_str() {
        ASK_TYPE_COIN => match bid_order.bid_type.as_str() {
            BID_TYPE_COIN => is_coin_match_executable(ask_order, bid_order),
            _ => ContractError::AskBidMismatch.to_err(),
        },
        ASK_TYPE_MARKER => match bid_order.bid_type.as_str() {
            BID_TYPE_MARKER => is_marker_match_executable(deps, ask_order, bid_order),
            _ => ContractError::AskBidMismatch.to_err(),
        },
        _ => ContractError::AskBidMismatch.to_err(),
    }
}

fn is_coin_match_executable(
    ask_order: &AskOrder,
    bid_order: &BidOrder,
) -> Result<bool, ContractError> {
    let (mut ask_base, mut ask_quote) =
        if let AskCollateral::Coin { base, quote } = &ask_order.collateral {
            (base.to_owned(), quote.to_owned())
        } else {
            return ContractError::AskBidMismatch.to_err();
        };
    let (mut bid_base, mut bid_quote) =
        if let BidCollateral::Coin { base, quote } = &bid_order.collateral {
            (base.to_owned(), quote.to_owned())
        } else {
            return ContractError::AskBidMismatch.to_err();
        };
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));
    ask_base.sort_by(coin_sorter);
    bid_base.sort_by(coin_sorter);
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);

    (ask_base == bid_base && ask_quote == bid_quote).to_ok()
}

fn is_marker_match_executable(
    deps: &DepsMut<ProvenanceQuery>,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
) -> Result<bool, ContractError> {
    let (ask_address, ask_denom, ask_quote_per_share) = if let AskCollateral::Marker {
        address,
        denom,
        quote_per_share,
        ..
    } = &ask_order.collateral
    {
        (address, denom, quote_per_share)
    } else {
        return ContractError::AskBidMismatch.to_err();
    };
    let (bid_address, bid_denom, mut bid_quote) = if let BidCollateral::Marker {
        address,
        denom,
        quote,
    } = &bid_order.collateral
    {
        (address, denom, quote.to_owned())
    } else {
        return ContractError::AskBidMismatch.to_err();
    };
    let marker = ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(ask_denom)?;
    let mut ask_quote = get_marker_quote(&marker, &ask_quote_per_share);
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);
    (ask_address == bid_address && ask_denom == bid_denom && ask_quote == bid_quote).to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr, CosmosMsg, Timestamp};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn test_is_executable() {
        assert!(is_executable(
            &AskOrder {
                base: coins(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrder {
                base: coins(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(is_executable(
            &AskOrder {
                base: vec![coin(100, "base_1"), coin(200, "base_2")],
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrder {
                base: vec![coin(200, "base_2"), coin(100, "base_1")],
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrder {
                base: coins(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrder {
                base: coins(100, "base_2"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_1"),
            }
        ));
        assert!(!is_executable(
            &AskOrder {
                base: coins(100, "base_1"),
                id: "ask_id".to_string(),
                owner: Addr::unchecked("asker"),
                quote: coins(100, "quote_1"),
            },
            &BidOrder {
                base: coins(100, "base_1"),
                effective_time: Some(Timestamp::default()),
                id: "bid_id".to_string(),
                owner: Addr::unchecked("bidder"),
                quote: coins(100, "quote_2"),
            }
        ));
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
        let ask_order = AskOrder {
            base: vec![coin(100, "base_1"), coin(200, "base_2")],
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(200, "quote_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrder {
            base: vec![coin(200, "base_2"), coin(100, "base_1")],
            effective_time: Some(Timestamp::default()),
            id: "bid_id".to_string(),
            owner: Addr::unchecked("bidder"),
            quote: coins(200, "quote_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
        if let Err(error) = bid_storage.save(bid_order.id.as_bytes(), &bid_order) {
            panic!("unexpected error: {:?}", error);
        };

        // execute on matched ask order and bid order
        let execute_msg = ExecuteMsg::ExecuteMatch {
            ask_id: ask_order.id,
            bid_id: bid_order.id,
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
                assert_eq!(
                    execute_response.messages[1].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: bid_order.owner.to_string(),
                        amount: bid_order.base,
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
            &ContractInfo::new(
                Addr::unchecked("contract_admin"),
                "contract_bind_name".into(),
                "contract_name".into(),
            ),
        ) {
            panic!("unexpected error: {:?}", error)
        }

        // store valid ask order
        let ask_order = AskOrder {
            base: coins(200, "base_1"),
            id: "ask_id".into(),
            owner: Addr::unchecked("asker"),
            quote: coins(100, "quote_1"),
        };

        let mut ask_storage = get_ask_storage(&mut deps.storage);
        if let Err(error) = ask_storage.save(ask_order.id.as_bytes(), &ask_order) {
            panic!("unexpected error: {:?}", error)
        };

        // store valid bid order
        let bid_order = BidOrder {
            base: coins(100, "base_1"),
            effective_time: Some(Timestamp::default()),
            id: "bid_id".into(),
            owner: Addr::unchecked("bidder"),
            quote: coins(100, "quote_1"),
        };

        let mut bid_storage = get_bid_storage(&mut deps.storage);
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
            Err(ContractError::Unauthorized) => {}
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
            Err(ContractError::AskBidMismatch) => {}
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
            Err(ContractError::AskBidMismatch) => {}
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
            Err(ContractError::AskBidMismatch) => {}
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
            Err(ContractError::ExecuteWithFunds) => {}
            Err(error) => panic!("unexpected error: {:?}", error),
            Ok(_) => panic!("expected error, but execute_response ok"),
        }
    }
}
