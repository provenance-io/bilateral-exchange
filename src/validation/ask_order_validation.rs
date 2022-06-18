use crate::types::ask_collateral::AskCollateral;
use crate::types::ask_order::AskOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::Coin;
use crate::types::request_type::RequestType;

pub fn validate_ask_order(ask_order: &AskOrder) -> Result<(), ContractError> {
    let mut invalid_field_messages: Vec<String> = vec![];
    if ask_order.id.is_empty() {
        invalid_field_messages.push("id for AskOrder must not be empty".to_string());
    }
    if ask_order.owner.as_str().is_empty() {
        invalid_field_messages.push("owner for AskOrder must not be empty".to_string());
    }
    match ask_order.ask_type {
        RequestType::CoinTrade => {
            if !matches!(ask_order.collateral, AskCollateral::CoinTrade(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::CoinTrade",
                    ask_order.ask_type.get_name(), ask_order.id,
                ));
            }
        }
        RequestType::MarkerTrade => {
            if !matches!(ask_order.collateral, AskCollateral::MarkerTrade(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::MarkerTrade",
                    ask_order.ask_type.get_name(), ask_order.id,
                ));
            }
        }
        RequestType::MarkerShareSale => {
            if !matches!(ask_order.collateral, AskCollateral::MarkerShareSale(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral of type AskCollateral::MarkerShareSale",
                    ask_order.ask_type.get_name(), ask_order.id,
                ))
            }
        }
        RequestType::ScopeTrade => {
            if !matches!(ask_order.collateral, AskCollateral::ScopeTrade(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral of type AskCollateral::ScopeTrade",
                    ask_order.ask_type.get_name(), ask_order.id,
                ))
            }
        }
    };
    let validate_coin = |coin: &Coin, coin_type: &str| {
        let mut messages: Vec<String> = vec![];
        if coin.amount.u128() == 0 {
            messages.push(
                format!(
                    "Zero amounts not allowed on coins. Coin denom [{}] and type [{}] for AskOrder [{}]",
                    &coin.denom,
                    coin_type,
                    &ask_order.id,
                )
            );
        }
        if coin.denom.is_empty() {
            messages.push(
                format!(
                    "Blank denoms not allowed on coins. Coin amount [{}] and type [{}] for AskOrder [{}]",
                    coin.amount.u128(),
                    coin_type,
                    &ask_order.id,
                )
            );
        }
        messages
    };
    match &ask_order.collateral {
        AskCollateral::CoinTrade(collateral) => {
            let prefix = format!("AskOrder [{}] of type coin trade", ask_order.id);
            if collateral.base.is_empty() {
                invalid_field_messages.push(format!("{} must include base funds", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .base
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Base Coin"))
                    .collect(),
            );
            if collateral.quote.is_empty() {
                invalid_field_messages.push(format!("{} must include quote funds", prefix,));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Quote Coin"))
                    .collect(),
            );
        }
        AskCollateral::MarkerTrade(collateral) => {
            let prefix = format!("AskOrder [{}] of type marker trade", ask_order.id);
            if collateral.address.as_str().is_empty() {
                invalid_field_messages
                    .push(format!("{} must have a valid marker address", prefix,));
            }
            if collateral.denom.is_empty() {
                invalid_field_messages.push(format!("{} must have a specified denom", prefix,));
            }
            if collateral.share_count.is_zero() {
                invalid_field_messages.push(format!(
                    "{} must refer to a marker with at least one of its coins held",
                    prefix,
                ))
            }
            if collateral.quote_per_share.is_empty() {
                invalid_field_messages.push(format!("{} must have a quote per share", prefix,))
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote_per_share
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Quote per Share Coin"))
                    .collect(),
            );
            if !collateral
                .removed_permissions
                .iter()
                .any(|perm| perm.address == ask_order.owner)
            {
                invalid_field_messages.push(format!(
                    "{} does not have a permission for owner [{}]",
                    prefix,
                    ask_order.owner.as_str()
                ));
            }
        }
        AskCollateral::MarkerShareSale(collateral) => {
            let prefix = format!("AskOrder [{}] of type marker share sale", ask_order.id);
            if collateral.address.as_str().is_empty() {
                invalid_field_messages.push(format!("{} must have a valid marker address", prefix));
            }
            if collateral.denom.is_empty() {
                invalid_field_messages.push(format!("{} must have a specified denom", prefix));
            }
            if collateral.remaining_shares.is_zero() {
                invalid_field_messages.push(format!(
                    "{} must refer to a marker with at least one of its coins held",
                    prefix,
                ))
            }
            if collateral.quote_per_share.is_empty() {
                invalid_field_messages.push(format!("{} must have a quote per share", prefix))
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote_per_share
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Quote per Share Coin"))
                    .collect(),
            );
            if !collateral
                .removed_permissions
                .iter()
                .any(|perm| perm.address == ask_order.owner)
            {
                invalid_field_messages.push(format!(
                    "{} does not have a permission for owner [{}]",
                    prefix,
                    ask_order.owner.as_str()
                ));
            }
        }
        AskCollateral::ScopeTrade(collateral) => {
            let prefix = format!("AskOrder [{}] of type scope trade", ask_order.id);
            if collateral.scope_address.is_empty() {
                invalid_field_messages.push(format!("{} must have a valid scope address", prefix));
            }
            if collateral.quote.is_empty() {
                invalid_field_messages
                    .push(format!("{} must have a valid quote specified", prefix));
            }
            invalid_field_messages.append(
                &mut collateral
                    .quote
                    .iter()
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Scope Trade Coin"))
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
