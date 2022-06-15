use crate::types::ask_order::AskOrder;
use crate::types::bid_order::BidOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::DepsMut;
use provwasm_std::ProvenanceQuery;

pub fn validate_execute_match(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
) -> Result<(), ContractError> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "AskOrder [{}] and BidOrder [{}] validation:",
        &ask.id, &bid.id
    );

    if ask.get_matching_bid_type() != bid.bid_type {
        validation_messages.push(format!(
            "{} Ask type [{}] does not match bid type [{}]",
            &identifiers, &ask.ask_type, &bid.bid_type,
        ));
    }
    //
    // match ask.collateral {
    //     AskCollateral { }
    // }

    if validation_messages.is_empty() {
        ().to_ok()
    } else {
        ContractError::ValidationError {
            messages: validation_messages,
        }
        .to_err()
    }
}
