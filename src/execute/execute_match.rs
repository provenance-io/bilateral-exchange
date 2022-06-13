use crate::contract_info::get_contract_info;
use crate::error::ContractError;
use crate::state::{
    get_ask_storage, get_ask_storage_read, get_bid_storage, get_bid_storage_read, AskOrder,
    BidOrder,
};
use cosmwasm_std::{attr, BankMsg, Coin, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

// match and execute an ask and bid order
pub fn execute_match(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
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

    // 'send quote to asker' and 'send base to bidder' messages
    let response = Response::new()
        .add_messages(vec![
            BankMsg::Send {
                to_address: ask_order.owner.to_string(),
                amount: ask_order.quote,
            },
            BankMsg::Send {
                to_address: bid_order.owner.to_string(),
                amount: bid_order.base,
            },
        ])
        .add_attributes(vec![attr("action", "execute")]);

    // finally remove the orders from storage
    get_ask_storage(deps.storage).remove(ask_id.as_bytes());
    get_bid_storage(deps.storage).remove(bid_id.as_bytes());

    Ok(response)
}

fn is_executable(ask_order: &AskOrder, bid_order: &BidOrder) -> bool {
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));

    let mut ask_base = ask_order.base.to_owned();
    ask_base.sort_by(coin_sorter);
    let mut bid_base = bid_order.base.to_owned();
    bid_base.sort_by(coin_sorter);

    let mut ask_quote = ask_order.quote.to_owned();
    ask_quote.sort_by(coin_sorter);
    let mut bid_quote = bid_order.quote.to_owned();
    bid_quote.sort_by(coin_sorter);

    ask_base == bid_base && ask_quote == bid_quote
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::contract_info::{set_contract_info, ContractInfo};
    use crate::msg::ExecuteMsg;
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
}
