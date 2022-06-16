use crate::types::ask_collateral::{AskCollateral, CoinAskCollateral, MarkerAskCollateral};
use crate::types::ask_order::AskOrder;
use crate::types::bid_collateral::{BidCollateral, CoinBidCollateral, MarkerBidCollateral};
use crate::types::bid_order::BidOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{get_marker_quote, get_single_marker_coin_holding};
use cosmwasm_std::{Coin, DepsMut};
use provwasm_std::{ProvenanceQuerier, ProvenanceQuery};

pub fn validate_match(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
) -> Result<(), ContractError> {
    let validation_messages = get_match_validation(deps, ask, bid);
    if validation_messages.is_empty() {
        ().to_ok()
    } else {
        ContractError::ValidationError {
            messages: validation_messages,
        }
        .to_err()
    }
}

fn get_match_validation(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
) -> Vec<String> {
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
                &mut get_coin_collateral_validation(ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type coin, which did not match bid collateral",
                identifiers
            )),
        },
        AskCollateral::Marker(ask_collat) => match &bid.collateral {
            BidCollateral::Marker(bid_collat) => validation_messages.append(
                &mut get_marker_collateral_validation(deps, ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type marker, which did not match bid collateral",
                identifiers
            )),
        },
    }
    validation_messages
}

fn get_coin_collateral_validation(
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
        validation_messages.push(format!("{} Ask base does not match bid base", &identifiers,));
    }
    if ask_quote != bid_quote {
        validation_messages.push(format!(
            "{} Ask quote does not match bid quote",
            &identifiers
        ));
    }
    validation_messages
}

fn get_marker_collateral_validation(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &MarkerAskCollateral,
    bid_collateral: &MarkerBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "MARKER Match Validation for AskOrder [{}] and BidOrder [{}]:",
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
                    "{} Failed to find marker for denom [{}]",
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
        validation_messages.push(format!(
            "{} Marker had invalid coin holdings for match: {:?}. Expected a single instance of coin [{}]",
            &identifiers,
            marker
                .coins
                .into_iter()
                .map(|coin| coin.denom)
                .collect::<Vec<String>>(),
            &ask_collateral.denom,
        ));
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
            "{} Ask quote did not match bid quote",
            &identifiers,
        ));
    }
    validation_messages
}

#[cfg(test)]
mod tests {
    use crate::test::mock_marker::MockMarker;
    use crate::types::ask_collateral::AskCollateral;
    use crate::types::ask_order::AskOrder;
    use crate::types::bid_collateral::BidCollateral;
    use crate::types::bid_order::BidOrder;
    use crate::types::constants::{ASK_TYPE_COIN, ASK_TYPE_MARKER, BID_TYPE_COIN, BID_TYPE_MARKER};
    use crate::types::error::ContractError;
    use crate::types::request_descriptor::RequestDescriptor;
    use crate::validation::ask_order_validation::validate_ask_order;
    use crate::validation::bid_order_validation::validate_bid_order;
    use crate::validation::execute_match_validation::validate_match;
    use cosmwasm_std::{coin, coins, Addr, Coin, DepsMut, Timestamp};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::{AccessGrant, MarkerAccess, ProvenanceQuery};

    #[test]
    fn test_successful_coin_validation() {
        let mut deps = mock_dependencies(&[]);
        let mut ask_order = AskOrder::new(
            "ask_id",
            Addr::unchecked("asker"),
            AskCollateral::coin(&coins(100, "nhash"), &coins(250, "othercoin")),
            None,
        )
        .expect("expected validation to pass for the new ask order");
        let mut bid_order = BidOrder::new(
            "bid_id",
            Addr::unchecked("bidder"),
            BidCollateral::coin(&coins(100, "nhash"), &coins(250, "othercoin")),
            None,
        )
        .expect("expected validation to pass for the new bid order");
        validate_match(&deps.as_mut(), &ask_order, &bid_order)
            .expect("expected validation to pass for a simple coin to coin trade");
        ask_order.collateral = AskCollateral::coin(
            &[coin(10, "a"), coin(20, "b"), coin(30, "c")],
            &[coin(50, "d"), coin(60, "e"), coin(70, "f")],
        );
        validate_ask_order(&ask_order).expect("expected modified ask order to remain valid");
        bid_order.collateral = BidCollateral::coin(
            &[coin(30, "c"), coin(10, "a"), coin(20, "b")],
            &[coin(50, "d"), coin(70, "f"), coin(60, "e")],
        );
        validate_bid_order(&bid_order).expect("expected modified bid order to remain valid");
        validate_match(&deps.as_mut(), &ask_order, &bid_order)
            .expect("expected validation to pass for a complex coin trade with mismatched orders");
    }

    #[test]
    fn test_successful_marker_validation() {
        let mut deps = mock_dependencies(&[]);
        let marker = MockMarker {
            denom: "targetcoin".to_string(),
            coins: coins(10, "targetcoin"),
            ..MockMarker::default()
        }
        .to_marker();
        deps.querier.with_markers(vec![marker.clone()]);
        let mut ask_order = AskOrder::new(
            "ask_id",
            Addr::unchecked("asker"),
            AskCollateral::marker(
                Addr::unchecked("marker"),
                "targetcoin",
                10,
                &coins(100, "nhash"),
                &[AccessGrant {
                    address: Addr::unchecked("asker"),
                    permissions: vec![MarkerAccess::Admin],
                }],
            ),
            Some(RequestDescriptor {
                description: Some("Best ask ever".to_string()),
                effective_time: Some(Timestamp::default()),
            }),
        )
        .expect("expected the ask order to be valid");
        let mut bid_order = BidOrder::new(
            "bid_id",
            Addr::unchecked("bidder"),
            BidCollateral::marker(
                Addr::unchecked("marker"),
                "targetcoin",
                &coins(1000, "nhash"),
            ),
            Some(RequestDescriptor {
                description: Some("Best bid ever".to_string()),
                effective_time: Some(Timestamp::default()),
            }),
        )
        .expect("expected the bid order to be valid");
        validate_match(&deps.as_mut(), &ask_order, &bid_order)
            .expect("expected validation to pass for a single coin quote");
        ask_order.collateral = AskCollateral::marker(
            Addr::unchecked("marker"),
            "targetcoin",
            10,
            &[
                coin(10, "nhash"),
                coin(5, "otherthing"),
                coin(13, "worstthing"),
            ],
            &[AccessGrant {
                address: Addr::unchecked("asker"),
                permissions: vec![MarkerAccess::Admin],
            }],
        );
        validate_ask_order(&ask_order)
            .expect("expected the ask order to remain valid after changes");
        bid_order.collateral = BidCollateral::marker(
            Addr::unchecked("marker"),
            "targetcoin",
            &[
                coin(100, "nhash"),
                coin(50, "otherthing"),
                coin(130, "worstthing"),
            ],
        );
        validate_bid_order(&bid_order)
            .expect("expected the bid order to remain valid after changes");
        validate_match(&deps.as_mut(), &ask_order, &bid_order)
            .expect("expected the validation to pass for a multi-coin quote");
    }

    #[test]
    fn test_mismatched_ask_and_bid_types() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "Ask type coin and bid type marker mismatch",
            &deps.as_mut(),
            &AskOrder {
                id: "ask_id".to_string(),
                ask_type: ASK_TYPE_COIN.to_string(),
                owner: Addr::unchecked("ask_addr"),
                collateral: AskCollateral::coin(&[], &[]),
                descriptor: None,
            },
            &BidOrder {
                id: "bid_id".to_string(),
                bid_type: BID_TYPE_MARKER.to_string(),
                owner: Addr::unchecked("bid_addr"),
                collateral: BidCollateral::coin(&[], &[]),
                descriptor: None,
            },
            expected_error("Ask type [coin] does not match bid type [marker]"),
        );
        assert_validation_failure(
            "Ask type marker and bid type coin mismatch",
            &deps.as_mut(),
            &AskOrder {
                id: "ask_id".to_string(),
                ask_type: ASK_TYPE_MARKER.to_string(),
                owner: Addr::unchecked("ask_addr"),
                collateral: AskCollateral::coin(&[], &[]),
                descriptor: None,
            },
            &BidOrder {
                id: "bid_id".to_string(),
                bid_type: BID_TYPE_COIN.to_string(),
                owner: Addr::unchecked("bid_addr"),
                collateral: BidCollateral::coin(&[], &[]),
                descriptor: None,
            },
            expected_error("Ask type [marker] does not match bid type [coin]"),
        );
    }

    #[test]
    fn test_mismatched_collateral_types() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "Ask collateral coin and bid collateral marker mismatch",
            &deps.as_mut(),
            &mock_ask(AskCollateral::coin(&[], &[])),
            &mock_bid(mock_bid_marker("marker", "somecoin", &[])),
            expected_error("Ask collateral was of type coin, which did not match bid collateral"),
        );
        assert_validation_failure(
            "Ask collateral marker and bid collateral coin mismatch",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker", "somecoin", 400, &[])),
            &mock_bid(BidCollateral::coin(&[], &[])),
            expected_error("Ask collateral was of type marker, which did not match bid collateral"),
        );
    }

    #[test]
    fn test_mismatched_coin_bases() {
        let mut deps = mock_dependencies(&[]);
        let mut ask_order = mock_ask(AskCollateral::coin(&coins(150, "nhash"), &[]));
        let mut bid_order = mock_bid(BidCollateral::coin(&coins(100, "nhash"), &[]));
        assert_validation_failure(
            "Ask base denoms match but amounts do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin(&coins(100, "a"), &[]);
        bid_order.collateral = BidCollateral::coin(&coins(100, "b"), &[]);
        assert_validation_failure(
            "Ask base amounts match but denoms do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin(&[coin(100, "a"), coin(100, "b")], &[]);
        bid_order.collateral = BidCollateral::coin(&coins(100, "a"), &[]);
        assert_validation_failure(
            "Ask base includes coin not in bid base",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin(&coins(100, "a"), &[]);
        bid_order.collateral = BidCollateral::coin(&[coin(100, "a"), coin(100, "b")], &[]);
        assert_validation_failure(
            "Bid base includes coin not in ask base",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask base does not match bid base"),
        );
    }

    #[test]
    fn test_mismatched_coin_quotes() {
        let mut deps = mock_dependencies(&[]);
        let mut ask_order = mock_ask(AskCollateral::coin(&[], &coins(1, "nhash")));
        let mut bid_order = mock_bid(BidCollateral::coin(&[], &coins(2, "nhash")));
        assert_validation_failure(
            "Ask quote denoms match but amounts do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral = AskCollateral::coin(&[], &coins(4000, "acoin"));
        bid_order.collateral = BidCollateral::coin(&[], &coins(4000, "bcoin"));
        assert_validation_failure(
            "Ask quote amounts match but denoms do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral = AskCollateral::coin(&[], &[coin(200, "acoin"), coin(200, "bcoin")]);
        bid_order.collateral = BidCollateral::coin(&[], &coins(200, "acoin"));
        assert_validation_failure(
            "Ask quote includes coin not in bid quote",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral = AskCollateral::coin(&[], &coins(200, "acoin"));
        bid_order.collateral = BidCollateral::coin(&[], &[coin(200, "acoin"), coin(200, "bcoin")]);
        assert_validation_failure(
            "Bid quote includes coin not in ask quote",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            expected_coin_error("Ask quote does not match bid quote"),
        );
    }

    #[test]
    fn test_mismatched_marker_denoms() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "Ask marker denom does not match bid marker denom",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker", "firstmarkerdenom", 10, &[])),
            &mock_bid(mock_bid_marker("marker", "secondmarkerdenom", &[])),
            expected_marker_error("Ask marker denom [firstmarkerdenom] does not match bid marker denom [secondmarkerdenom]"),
        );
    }

    #[test]
    fn test_mismatched_marker_addresses() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "Ask marker address does not match bid marker address",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker1", "test", 10, &[])),
            &mock_bid(mock_bid_marker("marker2", "test", &[])),
            expected_marker_error(
                "Ask marker address [marker1] does not match bid marker address [marker2]",
            ),
        );
    }

    #[test]
    fn test_missing_marker_in_provland() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "No marker was mocked for target marker address",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker", "test", 10, &[])),
            &mock_bid(mock_bid_marker("marker", "test", &[])),
            expected_marker_error("Failed to find marker for denom [test]"),
        );
    }

    #[test]
    fn test_marker_unexpected_holdings() {
        let mut deps = mock_dependencies(&[]);
        let mut marker = MockMarker {
            denom: "targetcoin".to_string(),
            coins: vec![coin(100, "nhash"), coin(50, "mydenom")],
            ..MockMarker::default()
        }
        .to_marker();
        deps.querier.with_markers(vec![marker.clone()]);
        let ask = mock_ask(mock_ask_marker("marker", "targetcoin", 10, &[]));
        let bid = mock_bid(mock_bid_marker("marker", "targetcoin", &[]));
        assert_validation_failure(
            "Marker contained none of its own denom",
            &deps.as_mut(),
            &ask,
            &bid,
            expected_marker_error("Marker had invalid coin holdings for match: [\"nhash\", \"mydenom\"]. Expected a single instance of coin [targetcoin]"),
        );
        marker.coins = vec![];
        deps.querier.with_markers(vec![marker.clone()]);
        assert_validation_failure(
            "Marker contained no coins whatsoever",
            &deps.as_mut(),
            &ask,
            &bid,
            expected_marker_error("Marker had invalid coin holdings for match: []. Expected a single instance of coin [targetcoin]"),
        );
        marker.coins = vec![coin(10, "targetcoin"), coin(20, "targetcoin")];
        deps.querier.with_markers(vec![marker]);
        assert_validation_failure(
            "Marker contained duplicates of the target coin",
            &deps.as_mut(),
            &ask,
            &bid,
            expected_marker_error("Marker had invalid coin holdings for match: [\"targetcoin\", \"targetcoin\"]. Expected a single instance of coin [targetcoin]"),
        );
    }

    #[test]
    fn test_marker_unexpected_share_count() {
        let mut deps = mock_dependencies(&[]);
        let marker = MockMarker {
            denom: "targetcoin".to_string(),
            coins: coins(50, "targetcoin"),
            ..MockMarker::default()
        }
        .to_marker();
        deps.querier.with_markers(vec![marker]);
        assert_validation_failure(
            "Marker contained a coin count that did not match the value recorded when the ask was made",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker", "targetcoin", 49, &[])),
            &mock_bid(mock_bid_marker("marker", "targetcoin", &[])),
            expected_marker_error("Marker share count was [50] but the original value when added to the contract was [49]"),
        );
    }

    #[test]
    fn test_mismatched_marker_ask_and_bid_quotes() {
        let mut deps = mock_dependencies(&[]);
        let marker = MockMarker {
            denom: "targetcoin".to_string(),
            coins: coins(10, "targetcoin"),
            ..MockMarker::default()
        }
        .to_marker();
        deps.querier.with_markers(vec![marker]);
        assert_validation_failure(
            "Marker bid had a bad value to match the calculated marker quote",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker(
                "marker",
                "targetcoin",
                10,
                &coins(50, "nhash"),
            )),
            &mock_bid(mock_bid_marker(
                "marker",
                "targetcoin",
                &coins(200, "nhash"),
            )),
            expected_marker_error("Ask quote did not match bid quote"),
        );
    }

    fn assert_validation_failure<S1: Into<String>, S2: Into<String>>(
        test_name: S1,
        deps: &DepsMut<ProvenanceQuery>,
        ask_order: &AskOrder,
        bid_order: &BidOrder,
        expected_error_message: S2,
    ) {
        let test_name = test_name.into();
        let message = expected_error_message.into();
        let messages = match validate_match(deps, ask_order, bid_order) {
            Err(e) => match e {
                ContractError::ValidationError { messages } => messages,
                e => panic!(
                    "{}: Expected message [{}], but got unexpected error instead during validation: {:?}",
                    test_name, message, e
                ),
            },
            Ok(_) => panic!(
                "{}: Expected message [{}] to be be output for input values, but validation passed",
                test_name, message,
            ),
        };
        assert!(
            messages.contains(&message),
            "expected message [{}] to be in result list {:?} for ask [{}] and bid [{}]",
            &message,
            &messages,
            &ask_order.id,
            &bid_order.id,
        )
    }

    fn expected_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into()
        )
    }

    fn expected_coin_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "COIN Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into()
        )
    }

    fn expected_marker_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "MARKER Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into()
        )
    }

    fn mock_ask(collateral: AskCollateral) -> AskOrder {
        AskOrder::new_unchecked("ask_id", Addr::unchecked("asker"), collateral, None)
    }

    fn mock_ask_marker<S1: Into<String>, S2: Into<String>>(
        addr: S1,
        denom: S2,
        share_count: u128,
        share_quote: &[Coin],
    ) -> AskCollateral {
        AskCollateral::marker(
            Addr::unchecked(addr),
            denom,
            share_count,
            share_quote,
            &[AccessGrant {
                address: Addr::unchecked("asker"),
                permissions: vec![MarkerAccess::Admin],
            }],
        )
    }

    fn mock_bid(collateral: BidCollateral) -> BidOrder {
        BidOrder::new_unchecked("bid_id", Addr::unchecked("bidder"), collateral, None)
    }

    fn mock_bid_marker<S1: Into<String>, S2: Into<String>>(
        addr: S1,
        denom: S2,
        quote: &[Coin],
    ) -> BidCollateral {
        BidCollateral::marker(Addr::unchecked(addr), denom, quote)
    }
}
