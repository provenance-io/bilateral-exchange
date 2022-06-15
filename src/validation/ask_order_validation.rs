use crate::storage::ask_order::{AskCollateral, AskOrder};
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
            if !matches!(ask_order.collateral, AskCollateral::Coin { .. }) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::Coin",
                    ask_order.ask_type, ask_order.id,
                ));
            }
        }
        ASK_TYPE_MARKER => {
            if !matches!(ask_order.collateral, AskCollateral::Marker { .. }) {
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
        if coin.amount.u128() == 0 {
            invalid_field_messages.push(
                format!(
                    "Zero amounts not allowed on coins. Coin denom [{}] and type [{}] for AsKOrder [{}]",
                    coin.denom,
                    coin_type,
                    ask_order.id,
                )
            );
        }
        if coin.denom.is_empty() {
            invalid_field_messages.push(
                format!(
                    "Blank denoms not allowed on coins. Coin amount [{}] and type [{}] for AskOrder [{}]",
                    coin.amount.u128(),
                    coin_type,
                    ask_order.id,
                )
            );
        }
    };
    match &ask_order.collateral {
        AskCollateral::Coin { base, quote } => {
            if base.is_empty() {
                invalid_field_messages.push(format!(
                    "AskCollateral for AskOrder [{}] of type coin must include base funds",
                    ask_order.id
                ));
                base.iter()
                    .for_each(|coin| validate_coin(coin, "AskCollateral Base Coin"));
            }
            if quote.is_empty() {
                invalid_field_messages.push(format!(
                    "AskCollateral for AskOrder [{}] of type coin must include quote funds",
                    ask_order.id,
                ));
                quote
                    .iter()
                    .for_each(|coin| validate_coin(coin, "AskCollateral Quote Coin"));
            }
        }
        AskCollateral::Marker {
            address,
            denom,
            removed_permissions,
        } => {
            if address.as_str().is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must have a valid marker address",
                    ask_order.id,
                ));
            }
            if denom.is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type marker must have a specified denom",
                    ask_order.id
                ));
            }
            if !removed_permissions
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
        ContractError::InvalidFields {
            messages: invalid_field_messages,
        }
        .to_err()
    }
}
