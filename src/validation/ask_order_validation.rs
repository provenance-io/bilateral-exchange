use crate::types::ask_collateral::AskCollateral;
use crate::types::ask_order::AskOrder;
use crate::types::constants::{ASK_TYPE_COIN, ASK_TYPE_MARKER};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::Coin;

pub fn validate_ask_order(ask_order: &AskOrder) -> Result<(), ContractError> {
    let mut invalid_field_messages: Vec<String> = vec![];
    if ask_order.id.is_empty() {
        invalid_field_messages.push("id for AskOrder must not be empty".to_string());
    }
    if ask_order.owner.as_str().is_empty() {
        invalid_field_messages.push("owner for AskOrder must not be empty".to_string());
    }
    match ask_order.ask_type.as_str() {
        ASK_TYPE_COIN => {
            if !matches!(ask_order.collateral, AskCollateral::Coin(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::Coin",
                    ask_order.ask_type, ask_order.id,
                ));
            }
        }
        ASK_TYPE_MARKER => {
            if !matches!(ask_order.collateral, AskCollateral::Marker(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::Marker",
                    ask_order.ask_type, ask_order.id,
                ));
            }
        }
        _ => {
            invalid_field_messages.push(format!(
                "ask type [{}] for AskOrder [{}] is invalid. Must be one of: {:?}",
                ask_order.ask_type,
                ask_order.id,
                vec![ASK_TYPE_COIN, ASK_TYPE_MARKER],
            ));
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
        AskCollateral::Coin(collateral) => {
            if collateral.base.is_empty() {
                invalid_field_messages.push(format!(
                    "AskCollateral for AskOrder [{}] of type coin must include base funds",
                    ask_order.id
                ));
                invalid_field_messages.append(
                    &mut collateral
                        .base
                        .iter()
                        .flat_map(|coin| validate_coin(coin, "AskCollateral Base Coin"))
                        .collect(),
                );
            }
            if collateral.quote.is_empty() {
                invalid_field_messages.push(format!(
                    "AskCollateral for AskOrder [{}] of type coin must include quote funds",
                    ask_order.id,
                ));
                invalid_field_messages.append(
                    &mut collateral
                        .quote
                        .iter()
                        .flat_map(|coin| validate_coin(coin, "AskCollateral Quote Coin"))
                        .collect(),
                );
            }
        }
        AskCollateral::Marker(collateral) => {
            if collateral.address.as_str().is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must have a valid marker address",
                    ask_order.id,
                ));
            }
            if collateral.denom.is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must have a specified denom",
                    ask_order.id,
                ));
            }
            if collateral.share_count.is_zero() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must refer to a marker with at least one of its coins held",
                    ask_order.id,
                ))
            }
            if collateral.quote_per_share.is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must have a quote per share",
                    ask_order.id,
                ))
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
                    "AskOrder [{}] of type marker does not have a permission for owner [{}]",
                    ask_order.id,
                    ask_order.owner.as_str()
                ));
            }
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
