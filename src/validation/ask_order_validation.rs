use crate::storage::ask_order::{AskCollateral, AskOrder};
use crate::types::ask_base::{COIN_ASK_TYPE, MARKER_ASK_TYPE};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;

pub fn validate_ask_order(ask_order: &AskOrder) -> Result<(), ContractError> {
    let mut invalid_field_messages: Vec<String> = vec![];
    if ask_order.id.is_empty() {
        invalid_field_messages.push("id for AskOrder must not be empty".to_string());
    }
    match ask_order.ask_type.as_str() {
        COIN_ASK_TYPE => {
            if !matches!(ask_order.collateral, AskCollateral::Coin { .. }) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::Coin",
                    ask_order.ask_type, ask_order.id,
                ));
            }
        }
        MARKER_ASK_TYPE => {
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
                vec![COIN_ASK_TYPE, MARKER_ASK_TYPE],
            ));
        }
    };
    match &ask_order.collateral {
        AskCollateral::Coin { base, quote } => {
            if base.is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type coin must include base funds",
                    ask_order.id
                ));
            }
            if quote.is_empty() {
                invalid_field_messages.push(format!(
                    "AskOrder [{}] of type coin must include quote funds",
                    ask_order.id,
                ));
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
