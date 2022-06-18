use crate::types::ask_collateral::{
    AskCollateral, CoinTradeAskCollateral, MarkerShareSaleAskCollateral, MarkerTradeAskCollateral,
    ScopeTradeAskCollateral,
};
use crate::types::ask_order::AskOrder;
use crate::types::bid_collateral::{
    BidCollateral, CoinTradeBidCollateral, MarkerShareSaleBidCollateral, MarkerTradeBidCollateral,
    ScopeTradeBidCollateral,
};
use crate::types::bid_order::BidOrder;
use crate::types::error::ContractError;
use crate::types::share_sale_type::ShareSaleType;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{calculate_marker_quote, get_single_marker_coin_holding};
use cosmwasm_std::{Coin, DepsMut};
use provwasm_std::{ProvenanceQuerier, ProvenanceQuery};
use std::cmp::Ordering;

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

    if ask.ask_type != bid.bid_type {
        validation_messages.push(format!(
            "{} Ask type [{}] does not match bid type [{}]",
            &identifiers,
            &ask.ask_type.get_name(),
            &bid.bid_type.get_name(),
        ));
    }

    match &ask.collateral {
        AskCollateral::CoinTrade(ask_collat) => match &bid.collateral {
            BidCollateral::CoinTrade(bid_collat) => validation_messages.append(
                &mut get_coin_trade_collateral_validation(ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type coin trade, which did not match bid collateral",
                identifiers
            )),
        },
        AskCollateral::MarkerTrade(ask_collat) => match &bid.collateral {
            BidCollateral::MarkerTrade(bid_collat) => validation_messages.append(
                &mut get_marker_trade_collateral_validation(deps, ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask collateral was of type marker trade, which did not match bid collateral",
                identifiers
            )),
        },
        AskCollateral::MarkerShareSale(ask_collat) => match &bid.collateral {
            BidCollateral::MarkerShareSale(bid_collat) => validation_messages.append(
                &mut get_marker_share_sale_collateral_validation(deps, ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask Collateral was of type marker share sale, which did not match bid collateral",
                identifiers,
            )),
        },
        AskCollateral::ScopeTrade(ask_collat) => match &bid.collateral {
            BidCollateral::ScopeTrade(bid_collat) => validation_messages.append(
                &mut get_scope_trade_collateral_validation(ask, bid, ask_collat, bid_collat),
            ),
            _ => validation_messages.push(format!(
                "{} Ask Collateral was of type scope trade, which did not match bid collateral",
                identifiers,
            )),
        }
    };
    validation_messages
}

fn get_coin_trade_collateral_validation(
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &CoinTradeAskCollateral,
    bid_collateral: &CoinTradeBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "COIN TRADE Match Validation for AskOrder [{}] and BidOrder [{}]:",
        &ask.id, &bid.id
    );
    let mut ask_base = ask_collateral.base.to_owned();
    let mut ask_quote = ask_collateral.quote.to_owned();
    let mut bid_base = bid_collateral.base.to_owned();
    let mut bid_quote = bid_collateral.quote.to_owned();
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

fn get_marker_trade_collateral_validation(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &MarkerTradeAskCollateral,
    bid_collateral: &MarkerTradeBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "MARKER TRADE Match Validation for AskOrder [{}] and BidOrder [{}]:",
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
    let marker_share_count = if let Ok(marker_coin) = get_single_marker_coin_holding(&marker) {
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
        marker_coin.amount.u128()
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
    };
    let mut ask_quote = calculate_marker_quote(marker_share_count, &ask_collateral.quote_per_share);
    let mut bid_quote = bid_collateral.quote.to_owned();
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

fn get_marker_share_sale_collateral_validation(
    deps: &DepsMut<ProvenanceQuery>,
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &MarkerShareSaleAskCollateral,
    bid_collateral: &MarkerShareSaleBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "MARKER SHARE SALE Match Validation for AskOrder [{}] and BidOrder [{}]:",
        &ask.id, &bid.id,
    );
    if ask_collateral.denom != bid_collateral.denom {
        validation_messages.push(format!(
            "{} Ask marker denom [{}] does not match bid marker denom [{}]",
            &identifiers, &ask_collateral.denom, &bid_collateral.denom,
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
    match ask_collateral.sale_type {
        ShareSaleType::SingleTransaction { share_count } => {
            if bid_collateral.share_count.u128() != share_count.u128() {
                validation_messages.push(format!(
                    "{} Ask requested that [{}] shares be purchased, but bid wanted [{}]",
                    &identifiers,
                    share_count.u128(),
                    bid_collateral.share_count.u128(),
                ));
            }
        }
        ShareSaleType::MultipleTransactions {
            remove_sale_share_threshold,
        } => {
            if ask_collateral.remaining_shares.u128() < bid_collateral.share_count.u128() {
                validation_messages.push(format!(
                    "{} Bid requested [{}] but the remaining share count is [{}]",
                    &identifiers,
                    bid_collateral.share_count.u128(),
                    ask_collateral.remaining_shares.u128()
                ));
            } else {
                let shares_remaining_after_sale =
                    ask_collateral.remaining_shares.u128() - bid_collateral.share_count.u128();
                let share_threshold = remove_sale_share_threshold.map(|u| u.u128()).unwrap_or(0);
                if shares_remaining_after_sale < share_threshold {
                    validation_messages.push(
                        format!(
                            "{} Bid requested [{}] shares, which would reduce the remaining share count to [{}], which is lower than the specified threshold of [{}] shares",
                            &identifiers,
                            bid_collateral.share_count.u128(),
                            shares_remaining_after_sale,
                            share_threshold,
                        )
                    );
                }
            }
        }
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
        if marker_coin.amount.u128() < bid_collateral.share_count.u128() {
            validation_messages.push(format!(
                "{} Marker had [{}] shares remaining, but the bid requested [{}] shares",
                &identifiers,
                marker_coin.amount.u128(),
                bid_collateral.share_count.u128(),
            ));
        }
        if marker_coin.amount.u128() != ask_collateral.remaining_shares.u128() {
            validation_messages.push(format!(
                "{} Marker had [{}] shares remaining, which does not match the stored amount of [{}]",
                &identifiers,
                marker_coin.amount.u128(),
                ask_collateral.remaining_shares.u128(),
            ));
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
    let mut ask_quote = calculate_marker_quote(
        bid_collateral.share_count.u128(),
        &ask_collateral.quote_per_share,
    );
    let mut bid_quote = bid_collateral.quote.to_owned();
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);
    if ask_quote != bid_quote {
        validation_messages.push(format!(
            "{} Ask share price did not result in the same quote as the bid",
            &identifiers,
        ));
    }
    validation_messages
}

fn get_scope_trade_collateral_validation(
    ask: &AskOrder,
    bid: &BidOrder,
    ask_collateral: &ScopeTradeAskCollateral,
    bid_collateral: &ScopeTradeBidCollateral,
) -> Vec<String> {
    let mut validation_messages: Vec<String> = vec![];
    let identifiers = format!(
        "SCOPE TRADE Match Validation for AskOrder [{}] and Bid Order [{}]:",
        &ask.id, &bid.id,
    );
    if ask_collateral.scope_address != bid_collateral.scope_address {
        validation_messages.push(format!(
            "{} Ask scope address [{}] does not match bid scope address [{}]",
            &identifiers, &ask_collateral.scope_address, &bid_collateral.scope_address,
        ));
    }
    let mut ask_quote = ask_collateral.quote.to_owned();
    let mut bid_quote = bid_collateral.quote.to_owned();
    ask_quote.sort_by(coin_sorter);
    bid_quote.sort_by(coin_sorter);
    if ask_quote != bid_quote {
        validation_messages.push(format!(
            "{} Ask quote does not match bid quote",
            &identifiers,
        ));
    }
    validation_messages
}

fn coin_sorter(first: &Coin, second: &Coin) -> Ordering {
    first
        .denom
        .cmp(&second.denom)
        .then_with(|| first.amount.cmp(&second.amount))
}

#[cfg(test)]
mod tests {
    use crate::test::mock_marker::MockMarker;
    use crate::types::ask_collateral::AskCollateral;
    use crate::types::ask_order::AskOrder;
    use crate::types::bid_collateral::BidCollateral;
    use crate::types::bid_order::BidOrder;
    use crate::types::error::ContractError;
    use crate::types::request_descriptor::RequestDescriptor;
    use crate::types::request_type::RequestType;
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
            AskCollateral::coin_trade(&coins(100, "nhash"), &coins(250, "othercoin")),
            None,
        )
        .expect("expected validation to pass for the new ask order");
        let mut bid_order = BidOrder::new(
            "bid_id",
            Addr::unchecked("bidder"),
            BidCollateral::coin_trade(&coins(100, "nhash"), &coins(250, "othercoin")),
            None,
        )
        .expect("expected validation to pass for the new bid order");
        validate_match(&deps.as_mut(), &ask_order, &bid_order)
            .expect("expected validation to pass for a simple coin to coin trade");
        ask_order.collateral = AskCollateral::coin_trade(
            &[coin(10, "a"), coin(20, "b"), coin(30, "c")],
            &[coin(50, "d"), coin(60, "e"), coin(70, "f")],
        );
        validate_ask_order(&ask_order).expect("expected modified ask order to remain valid");
        bid_order.collateral = BidCollateral::coin_trade(
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
            AskCollateral::marker_trade(
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
            BidCollateral::marker_trade(
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
        ask_order.collateral = AskCollateral::marker_trade(
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
        bid_order.collateral = BidCollateral::marker_trade(
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
                ask_type: RequestType::CoinTrade,
                owner: Addr::unchecked("ask_addr"),
                collateral: AskCollateral::coin_trade(&[], &[]),
                descriptor: None,
            },
            &BidOrder {
                id: "bid_id".to_string(),
                bid_type: RequestType::MarkerTrade,
                owner: Addr::unchecked("bid_addr"),
                collateral: BidCollateral::coin_trade(&[], &[]),
                descriptor: None,
            },
            expected_error("Ask type [coin_trade] does not match bid type [marker_trade]"),
        );
        assert_validation_failure(
            "Ask type marker and bid type coin mismatch",
            &deps.as_mut(),
            &AskOrder {
                id: "ask_id".to_string(),
                ask_type: RequestType::MarkerTrade,
                owner: Addr::unchecked("ask_addr"),
                collateral: AskCollateral::coin_trade(&[], &[]),
                descriptor: None,
            },
            &BidOrder {
                id: "bid_id".to_string(),
                bid_type: RequestType::CoinTrade,
                owner: Addr::unchecked("bid_addr"),
                collateral: BidCollateral::coin_trade(&[], &[]),
                descriptor: None,
            },
            expected_error("Ask type [marker_trade] does not match bid type [coin_trade]"),
        );
    }

    #[test]
    fn test_mismatched_collateral_types() {
        let mut deps = mock_dependencies(&[]);
        assert_validation_failure(
            "Ask collateral coin and bid collateral marker mismatch",
            &deps.as_mut(),
            &mock_ask(AskCollateral::coin_trade(&[], &[])),
            &mock_bid(mock_bid_marker("marker", "somecoin", &[])),
            expected_error(
                "Ask collateral was of type coin trade, which did not match bid collateral",
            ),
        );
        assert_validation_failure(
            "Ask collateral marker and bid collateral coin mismatch",
            &deps.as_mut(),
            &mock_ask(mock_ask_marker("marker", "somecoin", 400, &[])),
            &mock_bid(BidCollateral::coin_trade(&[], &[])),
            expected_error(
                "Ask collateral was of type marker trade, which did not match bid collateral",
            ),
        );
    }

    #[test]
    fn test_mismatched_coin_bases() {
        let mut deps = mock_dependencies(&[]);
        let mut ask_order = mock_ask(AskCollateral::coin_trade(&coins(150, "nhash"), &[]));
        let mut bid_order = mock_bid(BidCollateral::coin_trade(&coins(100, "nhash"), &[]));
        assert_validation_failure(
            "Ask base denoms match but amounts do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin_trade(&coins(100, "a"), &[]);
        bid_order.collateral = BidCollateral::coin_trade(&coins(100, "b"), &[]);
        assert_validation_failure(
            "Ask base amounts match but denoms do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin_trade(&[coin(100, "a"), coin(100, "b")], &[]);
        bid_order.collateral = BidCollateral::coin_trade(&coins(100, "a"), &[]);
        assert_validation_failure(
            "Ask base includes coin not in bid base",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask base does not match bid base"),
        );
        ask_order.collateral = AskCollateral::coin_trade(&coins(100, "a"), &[]);
        bid_order.collateral = BidCollateral::coin_trade(&[coin(100, "a"), coin(100, "b")], &[]);
        assert_validation_failure(
            "Bid base includes coin not in ask base",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask base does not match bid base"),
        );
    }

    #[test]
    fn test_mismatched_coin_quotes() {
        let mut deps = mock_dependencies(&[]);
        let mut ask_order = mock_ask(AskCollateral::coin_trade(&[], &coins(1, "nhash")));
        let mut bid_order = mock_bid(BidCollateral::coin_trade(&[], &coins(2, "nhash")));
        assert_validation_failure(
            "Ask quote denoms match but amounts do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral = AskCollateral::coin_trade(&[], &coins(4000, "acoin"));
        bid_order.collateral = BidCollateral::coin_trade(&[], &coins(4000, "bcoin"));
        assert_validation_failure(
            "Ask quote amounts match but denoms do not match",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral =
            AskCollateral::coin_trade(&[], &[coin(200, "acoin"), coin(200, "bcoin")]);
        bid_order.collateral = BidCollateral::coin_trade(&[], &coins(200, "acoin"));
        assert_validation_failure(
            "Ask quote includes coin not in bid quote",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask quote does not match bid quote"),
        );
        ask_order.collateral = AskCollateral::coin_trade(&[], &coins(200, "acoin"));
        bid_order.collateral =
            BidCollateral::coin_trade(&[], &[coin(200, "acoin"), coin(200, "bcoin")]);
        assert_validation_failure(
            "Bid quote includes coin not in ask quote",
            &deps.as_mut(),
            &ask_order,
            &bid_order,
            coin_trade_error("Ask quote does not match bid quote"),
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
            marker_trade_error("Ask marker denom [firstmarkerdenom] does not match bid marker denom [secondmarkerdenom]"),
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
            marker_trade_error(
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
            marker_trade_error("Failed to find marker for denom [test]"),
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
            marker_trade_error("Marker had invalid coin holdings for match: [\"nhash\", \"mydenom\"]. Expected a single instance of coin [targetcoin]"),
        );
        marker.coins = vec![];
        deps.querier.with_markers(vec![marker.clone()]);
        assert_validation_failure(
            "Marker contained no coins whatsoever",
            &deps.as_mut(),
            &ask,
            &bid,
            marker_trade_error("Marker had invalid coin holdings for match: []. Expected a single instance of coin [targetcoin]"),
        );
        marker.coins = vec![coin(10, "targetcoin"), coin(20, "targetcoin")];
        deps.querier.with_markers(vec![marker]);
        assert_validation_failure(
            "Marker contained duplicates of the target coin",
            &deps.as_mut(),
            &ask,
            &bid,
            marker_trade_error("Marker had invalid coin holdings for match: [\"targetcoin\", \"targetcoin\"]. Expected a single instance of coin [targetcoin]"),
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
            marker_trade_error("Marker share count was [50] but the original value when added to the contract was [49]"),
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
            marker_trade_error("Ask quote did not match bid quote"),
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

    fn coin_trade_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "COIN TRADE Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into()
        )
    }

    fn marker_trade_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "MARKER TRADE Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into()
        )
    }

    fn marker_share_sale_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "MARKER SHARE SALE Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into(),
        )
    }

    fn scope_trade_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "SCOPE TRADE Match Validation for AskOrder [ask_id] and BidOrder [bid_id]: {}",
            suffix.into(),
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
        AskCollateral::marker_trade(
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
        BidCollateral::marker_trade(Addr::unchecked(addr), denom, quote)
    }
}
