use crate::storage::bid_order::{get_bid_order_by_id, insert_bid_order, BidCollateral, BidOrder};
use crate::types::bid::{Bid, CoinBid, MarkerBid};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{attr, to_binary, DepsMut, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

// create bid entrypoint
pub fn create_bid(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    bid: Bid,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if get_bid_order_by_id(deps.storage, bid.get_id()).is_ok() {
        return ContractError::ExistingId {
            id_type: "bid".to_string(),
            id: bid.get_id().to_string(),
        }
        .to_err();
    }
    match bid {
        Bid::Coin(coin_bid) => create_coin_bid(deps, info, coin_bid),
        Bid::Marker(marker_bid) => create_marker_bid(deps, info, marker_bid),
    }
}

pub fn create_coin_bid(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    coin_bid: CoinBid,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if coin_bid.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if coin_bid.base.is_empty() {
        return ContractError::MissingField {
            field: "base".to_string(),
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "coin bid requests should include funds".to_string(),
        }
        .to_err();
    }
    let bid_order = BidOrder::new(
        &coin_bid.id,
        info.sender,
        BidCollateral::coin(coin_bid.base, info.funds),
        coin_bid.effective_time,
    )?;
    insert_bid_order(deps.storage, &bid_order)?;
    Response::new()
        .add_attributes(vec![attr("action", "create_bid")])
        .set_data(to_binary(&bid_order)?)
        .to_ok()
}

pub fn create_marker_bid(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    marker_bid: MarkerBid,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if marker_bid.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if marker_bid.denom.is_empty() {
        return ContractError::MissingField {
            field: "denom".to_string(),
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds must be provided during a marker bid to establish a quote".to_string(),
        }
        .to_err();
    }
    // This grants us access to the marker address, as well as ensuring that the marker is real
    let marker = ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_bid.denom)?;
    let bid_order = BidOrder::new(
        &marker_bid.id,
        info.sender,
        BidCollateral::marker(marker.address, &marker_bid.denom, info.funds),
        marker_bid.effective_time,
    )?;
    insert_bid_order(deps.storage, &bid_order)?;
    Response::new()
        .add_attribute("action", "create_bid")
        .set_data(to_binary(&bid_order)?)
        .to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::constants::BID_TYPE_COIN;
    use crate::types::msg::ExecuteMsg;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, Timestamp};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn create_coin_bid_with_valid_data() {
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
            bid: Bid::new_coin("bid_id", coins(100, "base_1"), Some(Timestamp::default())),
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
        if let ExecuteMsg::CreateBid {
            bid:
                Bid::Coin(CoinBid {
                    id,
                    base,
                    effective_time,
                }),
        } = create_bid_msg
        {
            match get_bid_order_by_id(deps.as_ref().storage, "bid_id") {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrder {
                            id,
                            bid_type: BID_TYPE_COIN.to_string(),
                            owner: bidder_info.sender,
                            collateral: BidCollateral::Coin {
                                base,
                                quote: bidder_info.funds,
                            },
                            effective_time,
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
    fn create_coin_bid_with_invalid_data() {
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
            bid: Bid::new_coin("", coins(100, "base_1"), Some(Timestamp::default())),
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
            bid: Bid::new_coin("id", vec![], Some(Timestamp::default())),
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
            bid: Bid::new_coin("id", coins(100, "base_1"), Some(Timestamp::default())),
        };

        // execute create bid
        let create_bid_response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bidder", &[]),
            create_bid_msg,
        )
        .expect_err("expected an error for a missing quote on a bid");

        // verify execute create bid response returns ContractError::BidMissingQuote
        match create_bid_response {
            ContractError::InvalidFundsProvided { message } => {
                assert_eq!("coin bid requests should include funds", message,);
            }
            e => panic!(
                "unexpected error when no funds provided to create bid: {:?}",
                e
            ),
        }
    }
}
