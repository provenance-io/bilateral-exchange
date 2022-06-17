use crate::types::bid_collateral::BidCollateral;
use crate::types::bid_order::BidOrder;
use crate::types::constants::{
    BID_TYPE_COIN_TRADE, BID_TYPE_MARKER_SHARE_SALE, BID_TYPE_MARKER_TRADE, BID_TYPE_SCOPE_TRADE,
};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::Coin;

pub fn validate_bid_order(bid_order: &BidOrder) -> Result<(), ContractError> {
    let mut invalid_field_messages: Vec<String> = vec![];
    if bid_order.id.is_empty() {
        invalid_field_messages.push("id for BidOrder must not be empty".to_string());
    }
    if bid_order.owner.as_str().is_empty() {
        invalid_field_messages.push("owner for BidOrder must not be empty".to_string());
    }
    match bid_order.bid_type.as_str() {
        BID_TYPE_COIN_TRADE => {
            if !matches!(bid_order.collateral, BidCollateral::CoinTrade(_)) {
                invalid_field_messages.push(format!(
                    "bid type [{}] for BidOrder [{}] is invalid. type requires collateral of type BidCollateral::CoinTrade",
                    bid_order.bid_type, bid_order.id,
                ));
            }
        }
        BID_TYPE_MARKER_TRADE => {
            if !matches!(bid_order.collateral, BidCollateral::MarkerTrade(_)) {
                invalid_field_messages.push(format!(
                   "bid type [{}] for BidOrder [{}] is invalid. type requires collateral of type BidCollateral::MarkerTrade",
                   bid_order.bid_type, bid_order.id,
               ));
            }
        }
        BID_TYPE_MARKER_SHARE_SALE => {
            if !matches!(bid_order.collateral, BidCollateral::MarkerShareSale(_)) {
                invalid_field_messages.push(format!(
                    "bid type [{}] for BidOrder [{}] is invalid. type requires collateral of type BidCollateral::MarkerShareSale",
                    bid_order.bid_type, bid_order.id,
                ))
            }
        }
        BID_TYPE_SCOPE_TRADE => {
            if !matches!(bid_order.collateral, BidCollateral::ScopeTrade(_)) {
                invalid_field_messages.push(format!(
                    "bid type [{}] for BidOrder [{}] is invalid. type requires collateral of type BidCollateral::ScopeTrade",
                    bid_order.bid_type, bid_order.id,
                ))
            }
        }
        _ => {
            invalid_field_messages.push(format!(
                "bid type [{}] for BidOrder [{}] is invalid. Must be one of: {:?}",
                bid_order.bid_type,
                bid_order.id,
                vec![BID_TYPE_COIN_TRADE, BID_TYPE_MARKER_TRADE],
            ));
        }
    };
    let validate_coin = |coin: &Coin, coin_type: &str| {
        let mut messages: Vec<String> = vec![];
        if coin.amount.u128() == 0 {
            messages.push(
                format!(
                    "Zero amounts not allowed on coins. Coin denom [{}] and type [{}] for BidOrder [{}]",
                    &coin.denom,
                    coin_type,
                    &bid_order.id,
                )
            );
        }
        if coin.denom.is_empty() {
            messages.push(
                format!(
                    "Blank denoms not allowed on coins. Coin amount [{}] and type [{}] for BidOrder [{}]",
                    coin.amount.u128(),
                    coin_type,
                    &bid_order.id,
                )
            );
        }
        messages
    };
    match &bid_order.collateral {
        BidCollateral::CoinTrade(collateral) => {
            let prefix = format!("BidOrder [{}] of type coin trade", bid_order.id);
            if collateral.base.is_empty() {
                invalid_field_messages.push(format!("{} must include base funds", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .base
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "BidCollateral Base Coin"))
                    .collect(),
            );
            if collateral.quote.is_empty() {
                invalid_field_messages.push(format!("{} must include base funds", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "BidCollateral Quote Coin"))
                    .collect(),
            );
        }
        BidCollateral::MarkerTrade(collateral) => {
            let prefix = format!("BidOrder [{}] of type marker trade", bid_order.id);
            if collateral.address.as_str().is_empty() {
                invalid_field_messages
                    .push(format!("{} must include a valid marker address", prefix,));
            }
            if collateral.denom.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include a valid marker denom", prefix,));
            }
            if collateral.quote.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include at least one quote coin", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "BidCollateral Quote Coin"))
                    .collect(),
            );
        }
        BidCollateral::MarkerShareSale(collateral) => {
            let prefix = format!("BidOrder [{}] of type marker share sale", bid_order.id);
            if collateral.address.as_str().is_empty() {
                invalid_field_messages
                    .push(format!("{} must include a valid marker address", prefix));
            }
            if collateral.denom.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include a valid marker denom", prefix));
            }
            if collateral.share_count.is_zero() {
                invalid_field_messages.push(format!(
                    "{} must request to purchase at least one share",
                    prefix
                ));
            }
            if collateral.quote.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include at least one quote coin", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "BidCollateral Quote Coin"))
                    .collect(),
            );
        }
        BidCollateral::ScopeTrade(collateral) => {
            let prefix = format!("BidOrder [{}] of type scope trade", bid_order.id);
            if collateral.scope_address.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include a valid scope address", prefix));
            }
            if collateral.quote.is_empty() {
                invalid_field_messages
                    .push(format!("{} must include at least one quote coin", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "BidCollateral Quote Coin"))
                    .collect(),
            );
        }
    }
    if invalid_field_messages.is_empty() {
        ().to_ok()
    } else {
        ContractError::ValidationError {
            messages: invalid_field_messages,
        }
        .to_err()
    }
}
