use crate::storage::bid_order_storage::{get_bid_order_by_id, insert_bid_order};
use crate::types::core::error::ContractError;
use crate::types::request::bid_types::bid::{
    Bid, CoinTradeBid, MarkerShareSaleBid, MarkerTradeBid, ScopeTradeBid,
};
use crate::types::request::bid_types::bid_collateral::BidCollateral;
use crate::types::request::bid_types::bid_order::BidOrder;
use crate::types::request::request_descriptor::RequestDescriptor;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::get_single_marker_coin_holding;
use cosmwasm_std::{to_binary, DepsMut, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

// create bid entrypoint
pub fn create_bid(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    bid: Bid,
    descriptor: Option<RequestDescriptor>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    if get_bid_order_by_id(deps.storage, bid.get_id()).is_ok() {
        return ContractError::ExistingId {
            id_type: "bid".to_string(),
            id: bid.get_id().to_string(),
        }
        .to_err();
    }
    let collateral = match &bid {
        Bid::CoinTrade(coin_trade) => create_coin_trade_collateral(&info, coin_trade),
        Bid::MarkerTrade(marker_trade) => {
            create_marker_trade_collateral(&deps, &info, marker_trade)
        }
        Bid::MarkerShareSale(marker_share_sale) => {
            create_marker_share_sale_collateral(&deps, &info, marker_share_sale)
        }
        Bid::ScopeTrade(scope_trade) => create_scope_trade_collateral(&info, scope_trade),
    }?;
    let bid_order = BidOrder::new(bid.get_id(), info.sender, collateral, descriptor)?;
    insert_bid_order(deps.storage, &bid_order)?;
    Response::new()
        .add_attribute("action", "create_bid")
        .set_data(to_binary(&bid_order)?)
        .to_ok()
}

fn create_coin_trade_collateral(
    info: &MessageInfo,
    coin_trade: &CoinTradeBid,
) -> Result<BidCollateral, ContractError> {
    if coin_trade.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if coin_trade.base.is_empty() {
        return ContractError::MissingField {
            field: "base".to_string(),
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "coin trade bid requests should include funds".to_string(),
        }
        .to_err();
    }
    BidCollateral::coin_trade(&coin_trade.base, &info.funds).to_ok()
}

fn create_marker_trade_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    marker_trade: &MarkerTradeBid,
) -> Result<BidCollateral, ContractError> {
    if marker_trade.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if marker_trade.denom.is_empty() {
        return ContractError::MissingField {
            field: "denom".to_string(),
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds must be provided during a marker trade bid to establish a quote"
                .to_string(),
        }
        .to_err();
    }
    // This grants us access to the marker address, as well as ensuring that the marker is real
    let marker = ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_trade.denom)?;
    BidCollateral::marker_trade(marker.address, &marker_trade.denom, &info.funds).to_ok()
}

fn create_marker_share_sale_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    info: &MessageInfo,
    marker_share_sale: &MarkerShareSaleBid,
) -> Result<BidCollateral, ContractError> {
    if marker_share_sale.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if marker_share_sale.denom.is_empty() {
        return ContractError::MissingField {
            field: "denom".to_string(),
        }
        .to_err();
    }
    if marker_share_sale.share_count.is_zero() {
        return ContractError::ValidationError {
            messages: vec!["share count must be at least one for a marker share sale".to_string()],
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds must be provided during a marker share trade bid to establish a quote"
                .to_string(),
        }
        .to_err();
    }
    let marker =
        ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&marker_share_sale.denom)?;
    let marker_shares_available = get_single_marker_coin_holding(&marker)?.amount.u128();
    if marker_share_sale.share_count.u128() > marker_shares_available {
        return ContractError::ValidationError {
            messages: vec![
                format!(
                    "share count [{}] must be less than or equal to remaining [{}] shares available [{}]",
                    marker_share_sale.share_count.u128(),
                    marker_share_sale.denom,
                    marker_shares_available,
                )
            ]
        }.to_err();
    }
    BidCollateral::marker_share_sale(
        marker.address,
        &marker_share_sale.denom,
        marker_share_sale.share_count.u128(),
        &info.funds,
    )
    .to_ok()
}

fn create_scope_trade_collateral(
    info: &MessageInfo,
    scope_trade: &ScopeTradeBid,
) -> Result<BidCollateral, ContractError> {
    if scope_trade.id.is_empty() {
        return ContractError::MissingField {
            field: "id".to_string(),
        }
        .to_err();
    }
    if scope_trade.scope_address.is_empty() {
        return ContractError::MissingField {
            field: "scope_address".to_string(),
        }
        .to_err();
    }
    if info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds must be provided during a scope trade bid to establish a quote"
                .to_string(),
        }
        .to_err();
    }
    BidCollateral::scope_trade(&scope_trade.scope_address, &info.funds).to_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::execute;
    use crate::storage::contract_info::{set_contract_info, ContractInfo};
    use crate::types::core::msg::ExecuteMsg;
    use crate::types::request::request_type::RequestType;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{attr, coins, Addr};
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
            bid: Bid::new_coin("bid_id", &coins(100, "base_1")),
            descriptor: None,
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
            bid: Bid::CoinTrade(CoinTradeBid { id, base }),
            descriptor: None,
        } = create_bid_msg
        {
            match get_bid_order_by_id(deps.as_ref().storage, "bid_id") {
                Ok(stored_order) => {
                    assert_eq!(
                        stored_order,
                        BidOrder {
                            id,
                            bid_type: RequestType::CoinTrade,
                            owner: bidder_info.sender,
                            collateral: BidCollateral::coin_trade(&base, &bidder_info.funds),
                            descriptor: None,
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
            bid: Bid::new_coin("", &coins(100, "base_1")),
            descriptor: None,
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
            bid: Bid::new_coin("id", &[]),
            descriptor: None,
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
            bid: Bid::new_coin("id", &coins(100, "base_1")),
            descriptor: None,
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
                assert_eq!("coin trade bid requests should include funds", message,);
            }
            e => panic!(
                "unexpected error when no funds provided to create bid: {:?}",
                e
            ),
        }
    }
}
