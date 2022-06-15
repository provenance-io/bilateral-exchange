use crate::types::ask_collateral::{AskCollateral, CoinAskCollateral, MarkerAskCollateral};
use crate::types::ask_order::AskOrder;
use crate::types::bid_collateral::{BidCollateral, CoinBidCollateral, MarkerBidCollateral};
use crate::types::bid_order::BidOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{get_marker_quote, get_single_marker_coin_holding};
use cosmwasm_std::{Coin, DepsMut};
use provwasm_std::{ProvenanceQuerier, ProvenanceQuery};

pub fn validate_execute_match(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
) -> Result<(), ContractError> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "Match Validation for AskOrder [{}] and BidOrder [{}]:",
        &ask.id, &bid.id
    );

    if ask.get_matching_bid_type() != bid.bid_type {
        validation_messages.push(format!(
            "{} Ask type [{}] does not match bid type [{}]",
            &identifiers, &ask.ask_type, &bid.bid_type,
        ));
    }

    match &ask.collateral {
        AskCollateral::Coin(ask_collat) => match &bid.collateral {
            BidCollateral::Coin(bid_collat) => validation_messages.append(
                &mut validate_coin_collateral(ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type coin, which did not match bid collateral",
                identifiers
            )),
        },
        AskCollateral::Marker(ask_collat) => match &bid.collateral {
            BidCollateral::Marker(bid_collat) => validation_messages.append(
                &mut validate_marker_collateral(deps, ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type marker, which did not match bid collateral",
                identifiers
            )),
        },
    }

    if validation_messages.is_empty() {
        ().to_ok()
    } else {
        ContractError::ValidationError {
            messages: validation_messages,
        }
        .to_err()
    }
}

fn validate_coin_collateral(
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &CoinAskCollateral,
    bid_collateral: &CoinBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "COIN Match Validation for AskOrder [{}] and BidOrder [{}]:",
        &ask.id, &bid.id
    );
    let mut ask_base = ask_collateral.base.to_owned();
    let mut ask_quote = ask_collateral.quote.to_owned();
    let mut bid_base = bid_collateral.base.to_owned();
    let mut bid_quote = bid_collateral.quote.to_owned();
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));
    ask_base.sort_by(coin_sorter);
    bid_base.sort_by(coin_sorter);
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);
    if ask_base != bid_base {
        validation_messages.push(format!("{} Ask base does not match bid base", &identifiers));
    }
    if ask_quote != bid_quote {
        validation_messages.push(format!(
            "{} Ask quote does not match bid quote",
            &identifiers
        ));
    }
    validation_messages
}

fn validate_marker_collateral(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &MarkerAskCollateral,
    bid_collateral: &MarkerBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "MARKER Match Validation for AskOrder [{}] and BidOrder [{}]",
        &ask.id, &bid.id
    );
    if ask_collateral.denom != bid_collateral.denom {
        validation_messages.push(format!(
            "{} Ask marker denom [{}] does not match bid marker denom [{}]",
            &identifiers, &ask_collateral.denom, &bid_collateral.denom
        ));
    }
    if ask_collateral.address.as_str() != bid_collateral.address.as_str() {
        validation_messages.push(format!(
            "{} Ask marker address [{}] does not match bid marker address [{}]",
            &identifiers,
            &ask_collateral.address.as_str(),
            &bid_collateral.address.as_str()
        ));
    }
    // If a denom or address mismatch exists between the ask and bid, no other sane checks can be
    // made because each refers to a different marker
    if !validation_messages.is_empty() {
        return validation_messages;
    }
    let marker =
        match ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(&ask_collateral.denom) {
            Ok(marker) => marker,
            // Exit early if the marker does not appear to be available in the Provenance Blockchain
            // system.  No marker means the remaining checks are meaningless.
            Err(_) => {
                validation_messages.push(format!(
                    "{} Failed to find marker by ask denom [{}]",
                    &identifiers, &ask_collateral.denom
                ));
                return validation_messages;
            }
        };
    if let Ok(marker_coin) = get_single_marker_coin_holding(&marker) {
        if marker_coin.amount.u128() != ask_collateral.share_count.u128() {
            validation_messages.push(
                format!(
                    "{} Marker share count was [{}] but the original value when added to the contract was [{}]",
                    &identifiers,
                    marker_coin.amount.u128(),
                    ask_collateral.share_count.u128(),
                )
            );
        }
    } else {
        validation_messages.push(
            format!("{} Marker contained multiple coin instances and could not be validated. Had coins of type: {:?}",
                    &identifiers,
                    marker.coins.into_iter().map(|coin| coin.denom).collect::<Vec<String>>(),
            )
        );
        return validation_messages;
    }
    let mut ask_quote = match get_marker_quote(&marker, &ask_collateral.quote_per_share) {
        Ok(ask_quote) => ask_quote,
        Err(e) => {
            validation_messages.push(format!(
                "{} Could not determine ask quote from marker coin balances: {:?}",
                &identifiers, e,
            ));
            return validation_messages;
        }
    };
    let mut bid_quote = bid_collateral.quote.to_owned();
    // sort the base and quote vectors by the order chain: denom, amount
    let coin_sorter =
        |a: &Coin, b: &Coin| a.denom.cmp(&b.denom).then_with(|| a.amount.cmp(&b.amount));
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);
    if ask_quote != bid_quote {
        validation_messages.push(format!(
            "{} Ask quote {:?} did not match bid quote {:?}",
            &identifiers, ask_quote, bid_quote
        ));
    }
    validation_messages
}
