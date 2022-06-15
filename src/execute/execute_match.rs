use crate::storage::ask_order_storage::{delete_ask_order_by_id, get_ask_order_by_id};
use crate::storage::bid_order_storage::{delete_bid_order_by_id, get_bid_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::ask_collateral::AskCollateral;
use crate::types::bid_collateral::BidCollateral;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::validation::execute_match_validation::validate_execute_match;
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

    validate_execute_match(&deps, &ask_order, &bid_order)?;

    let mut messages: Vec<CosmosMsg<ProvenanceMsg>> = vec![];
    match &ask_order.collateral {
        // Send quote to asker when a coin match is made
        AskCollateral::Coin(collateral) => messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: ask_order.owner.to_string(),
            amount: collateral.quote.to_owned(),
        })),
        // Now that the match has been made, grant all permissions on the marker to the bidder that
        // the asker once had.  The validation code has already ensured that the asker was an admin
        // of the marker, so the bidder at very least has the permission on the marker to grant
        // themselves any remaining permissions they desire.
        AskCollateral::Marker(collateral) => {
            if let Some(asker_permissions) = collateral
                .removed_permissions
                .iter()
                .find(|perm| perm.address == ask_order.owner)
            {
                messages.push(grant_marker_access(
                    &collateral.denom,
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
        BidCollateral::Coin(collateral) => messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: collateral.base.to_owned(),
        })),
        BidCollateral::Marker(collateral) => {
            // Send the entirety of the quote to the asker. They have just effectively sold their
            // marker to the bidder.
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: ask_order.owner.to_string(),
                amount: collateral.quote.to_owned(),
            }));
            messages.push(revoke_marker_access(
                &collateral.denom,
                env.contract.address,
            )?);
        }
    }
    // Now that all matching has concluded, we simply need to delete the ask and bid from storage
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;

    Response::new()
        .add_messages(messages)
        .add_attribute("action", "execute")
        .add_attribute("ask_id", &ask_order.id)
        .add_attribute("bid_id", &bid_order.id)
        .to_ok()
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

    // #[test]
    // fn test_is_executable() {
    //     assert!(is_executable(
    //         &AskOrder {
    //             base: coins(100, "base_1"),
    //             id: "ask_id".to_string(),
    //             owner: Addr::unchecked("asker"),
    //             quote: coins(100, "quote_1"),
    //         },
    //         &BidOrder {
    //             base: coins(100, "base_1"),
    //             effective_time: Some(Timestamp::default()),
    //             id: "bid_id".to_string(),
    //             owner: Addr::unchecked("bidder"),
    //             quote: coins(100, "quote_1"),
    //         }
    //     ));
    //     assert!(is_executable(
    //         &AskOrder {
    //             base: vec![coin(100, "base_1"), coin(200, "base_2")],
    //             id: "ask_id".to_string(),
    //             owner: Addr::unchecked("asker"),
    //             quote: coins(100, "quote_1"),
    //         },
    //         &BidOrder {
    //             base: vec![coin(200, "base_2"), coin(100, "base_1")],
    //             effective_time: Some(Timestamp::default()),
    //             id: "bid_id".to_string(),
    //             owner: Addr::unchecked("bidder"),
    //             quote: coins(100, "quote_1"),
    //         }
    //     ));
    //     assert!(!is_executable(
    //         &AskOrder {
    //             base: coins(100, "base_1"),
    //             id: "ask_id".to_string(),
    //             owner: Addr::unchecked("asker"),
    //             quote: coins(100, "quote_1"),
    //         },
    //         &BidOrder {
    //             base: coins(100, "base_2"),
    //             effective_time: Some(Timestamp::default()),
    //             id: "bid_id".to_string(),
    //             owner: Addr::unchecked("bidder"),
    //             quote: coins(100, "quote_1"),
    //         }
    //     ));
    //     assert!(!is_executable(
    //         &AskOrder {
    //             base: coins(100, "base_1"),
    //             id: "ask_id".to_string(),
    //             owner: Addr::unchecked("asker"),
    //             quote: coins(100, "quote_1"),
    //         },
    //         &BidOrder {
    //             base: coins(100, "base_1"),
    //             effective_time: Some(Timestamp::default()),
    //             id: "bid_id".to_string(),
    //             owner: Addr::unchecked("bidder"),
    //             quote: coins(100, "quote_2"),
    //         }
    //     ));
    // }

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
