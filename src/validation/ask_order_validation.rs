use crate::types::core::error::ContractError;
use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::types::request::ask_types::ask_order::AskOrder;
use crate::types::request::request_type::RequestType;
use crate::types::request::share_sale_type::ShareSaleType;
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
    if let Some(attribute_requirement) = ask_order
        .descriptor
        .as_ref()
        .and_then(|d| d.attribute_requirement.as_ref())
    {
        if attribute_requirement.attributes.is_empty() {
            invalid_field_messages.push(format!(
                "AskOrder [{}] specified RequiredAttributes, but the value included no attributes to check",
                ask_order.id,
            ));
        }
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
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::MarkerShareSale",
                    ask_order.ask_type.get_name(), ask_order.id,
                ))
            }
        }
        RequestType::ScopeTrade => {
            if !matches!(ask_order.collateral, AskCollateral::ScopeTrade(_)) {
                invalid_field_messages.push(format!(
                    "ask type [{}] for AskOrder [{}] is invalid. type requires collateral type of AskCollateral::ScopeTrade",
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
            match collateral.sale_type {
                ShareSaleType::SingleTransaction { share_count } => {
                    if share_count.is_zero() {
                        invalid_field_messages.push(
                            format!(
                                "{} specified share count for single transaction must be greater than zero",
                                prefix,
                            )
                        );
                    }
                    if share_count.u128() > collateral.remaining_shares.u128() {
                        invalid_field_messages.push(
                            format!(
                                "{} specified share count [{}] cannot be greater than marker remaining shares [{}]", 
                                prefix,
                                share_count.u128(),
                                collateral.remaining_shares.u128(),
                            )
                        );
                    }
                }
                ShareSaleType::MultipleTransactions {
                    remove_sale_share_threshold,
                } => {
                    if let Some(threshold) = remove_sale_share_threshold {
                        if threshold.u128() > collateral.remaining_shares.u128() {
                            invalid_field_messages.push(
                                format!(
                                    "{} specified ask removal threshold [{}] cannot be greater than marker remaining shares [{}]",
                                    prefix,
                                    threshold.u128(),
                                    collateral.remaining_shares.u128(),
                                )
                            );
                        }
                    }
                }
            };
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
                    .flat_map(|coin| validate_coin(coin, "AskCollateral Quote"))
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

#[cfg(test)]
#[cfg(feature = "enable-test-utils")]
mod tests {
    use crate::test::request_helpers::{
        mock_ask, mock_ask_marker_share_multi, mock_ask_marker_share_single, mock_ask_marker_trade,
        mock_ask_scope_trade, mock_ask_with_descriptor,
    };
    use crate::types::core::error::ContractError;
    use crate::types::request::ask_types::ask_collateral::AskCollateral;
    use crate::types::request::ask_types::ask_order::AskOrder;
    use crate::types::request::request_descriptor::{AttributeRequirement, RequestDescriptor};
    use crate::types::request::request_type::RequestType;
    use crate::types::request::share_sale_type::ShareSaleType;
    use crate::validation::ask_order_validation::validate_ask_order;
    use cosmwasm_std::{coins, Addr};
    use provwasm_std::AccessGrant;

    #[test]
    fn test_missing_id() {
        assert_validation_failure(
            "ask order id is empty",
            &AskOrder::new_unchecked(
                "",
                Addr::unchecked("addr"),
                AskCollateral::coin_trade(&[], &[]),
                None,
            ),
            "id for AskOrder must not be empty",
        );
    }

    #[test]
    fn test_missing_owner_address() {
        assert_validation_failure(
            "ask order address is empty",
            &AskOrder::new_unchecked(
                "ask_id",
                Addr::unchecked(""),
                AskCollateral::coin_trade(&[], &[]),
                None,
            ),
            "owner for AskOrder must not be empty",
        );
    }

    #[test]
    fn test_attribute_requirement_provided_with_empty_attributes_list() {
        assert_validation_failure(
            "ask order provided an empty attributes list for RequiredAttributes",
            &mock_ask_with_descriptor(
                AskCollateral::coin_trade(&[], &[]),
                RequestDescriptor::new_populated_attributes(
                    "hello",
                    AttributeRequirement::all::<String>(&[]),
                ),
            ),
            "AskOrder [ask_id] specified RequiredAttributes, but the value included no attributes to check",
        );
    }

    #[test]
    fn test_ask_type_mismatches() {
        let mut ask_order = AskOrder {
            id: "ask_id".to_string(),
            ask_type: RequestType::CoinTrade,
            owner: Addr::unchecked("addr"),
            collateral: AskCollateral::scope_trade("scope", &[]),
            descriptor: None,
        };
        assert_validation_failure(
            "ask order provided coin_trade request type but wrong collateral type",
            &ask_order,
            "ask type [coin_trade] for AskOrder [ask_id] is invalid. type requires collateral type of AskCollateral::CoinTrade",
        );
        ask_order.ask_type = RequestType::MarkerTrade;
        assert_validation_failure(
            "ask order provided marker_trade request type but wrong collateral type",
            &ask_order,
            "ask type [marker_trade] for AskOrder [ask_id] is invalid. type requires collateral type of AskCollateral::MarkerTrade",
        );
        ask_order.ask_type = RequestType::MarkerShareSale;
        assert_validation_failure(
            "ask order provided marker_share_sale request type but wrong collateral type",
            &ask_order,
            "ask type [marker_share_sale] for AskOrder [ask_id] is invalid. type requires collateral type of AskCollateral::MarkerShareSale",
        );
        ask_order.ask_type = RequestType::ScopeTrade;
        ask_order.collateral = AskCollateral::coin_trade(&[], &[]);
        assert_validation_failure(
            "ask order provided scope_trade request type but wrong collateral type",
            &ask_order,
            "ask type [scope_trade] for AskOrder [ask_id] is invalid. type requires collateral type of AskCollateral::ScopeTrade",
        );
    }

    #[test]
    fn test_coin_trade_empty_base() {
        assert_validation_failure(
            "ask order is missing coin trade base funds",
            &mock_ask(AskCollateral::coin_trade(&[], &coins(100, "nhash"))),
            coin_trade_error("must include base funds"),
        );
    }

    #[test]
    fn test_coin_trade_base_funds_include_invalid_coins() {
        assert_validation_failure(
            "ask order includes base coin with zero amount",
            &mock_ask(AskCollateral::coin_trade(&coins(0, "nhash"), &[])),
            zero_coin_error("nhash", "AskCollateral Base Coin"),
        );
        assert_validation_failure(
            "ask order includes base coin with invalid denom",
            &mock_ask(AskCollateral::coin_trade(&coins(100, ""), &[])),
            blank_denom_error(100, "AskCollateral Base Coin"),
        );
    }

    #[test]
    fn test_coin_trade_empty_quote() {
        assert_validation_failure(
            "ask order is missing coin trade quote funds",
            &mock_ask(AskCollateral::coin_trade(&coins(100, "nhash"), &[])),
            coin_trade_error("must include quote funds"),
        );
    }

    #[test]
    fn test_coin_trade_quote_funds_include_invalid_coins() {
        assert_validation_failure(
            "ask order includes quote coin with zero amount",
            &mock_ask(AskCollateral::coin_trade(&[], &coins(0, "nhash"))),
            zero_coin_error("nhash", "AskCollateral Quote Coin"),
        );
        assert_validation_failure(
            "ask order includes base coin with invalid denom",
            &mock_ask(AskCollateral::coin_trade(&[], &coins(100, ""))),
            blank_denom_error(100, "AskCollateral Quote Coin"),
        );
    }

    #[test]
    fn test_marker_trade_empty_marker_address() {
        assert_validation_failure(
            "ask order does not include a valid marker address",
            &mock_ask(mock_ask_marker_trade(
                "",
                "denom",
                100,
                &coins(100, "nhash"),
            )),
            marker_trade_error("must have a valid marker address"),
        );
    }

    #[test]
    fn test_marker_trade_empty_marker_denom() {
        assert_validation_failure(
            "ask order does not include a valid marker denom",
            &mock_ask(mock_ask_marker_trade(
                "marker_addr",
                "",
                100,
                &coins(100, "nhash"),
            )),
            marker_trade_error("must have a specified denom"),
        );
    }

    #[test]
    fn test_marker_trade_zero_share_count() {
        assert_validation_failure(
            "ask order specifies that the marker has zero of its own coin in holdings",
            &mock_ask(mock_ask_marker_trade(
                "marker_addr",
                "denom",
                0,
                &coins(100, "nhash"),
            )),
            marker_trade_error("must refer to a marker with at least one of its coins held"),
        );
    }

    #[test]
    fn test_marker_trade_empty_quote() {
        assert_validation_failure(
            "ask order did not specify a quote per share for its marker",
            &mock_ask(mock_ask_marker_trade("marker_addr", "denom", 100, &[])),
            marker_trade_error("must have a quote per share"),
        );
    }

    #[test]
    fn test_marker_trade_quote_funds_include_invalid_coins() {
        assert_validation_failure(
            "ask order includes quote coin with zero amount",
            &mock_ask(mock_ask_marker_trade(
                "marker_addr",
                "denom",
                100,
                &coins(0, "nhash"),
            )),
            zero_coin_error("nhash", "AskCollateral Quote per Share Coin"),
        );
        assert_validation_failure(
            "ask order includes quote coin with invalid denom",
            &mock_ask(mock_ask_marker_trade(
                "marker_addr",
                "denom",
                100,
                &coins(100, ""),
            )),
            blank_denom_error(100, "AskCollateral Quote per Share Coin"),
        );
    }

    #[test]
    fn test_marker_trade_removed_permissions_do_not_include_owner_address() {
        assert_validation_failure(
            "ask order does not specify that it removed the owner's permissions",
            &mock_ask(AskCollateral::marker_trade(
                Addr::unchecked("marker_address"),
                "denom",
                100,
                &coins(150, "nhash"),
                &[AccessGrant {
                    permissions: vec![],
                    address: Addr::unchecked("some rando"),
                }],
            )),
            marker_trade_error("does not have a permission for owner [asker]"),
        );
    }

    #[test]
    fn test_marker_share_sale_invalid_marker_address() {
        assert_validation_failure(
            "ask order does not include a valid marker address",
            &mock_ask(mock_ask_marker_share_single("", "denom", 10, &[], 10)),
            marker_share_sale_error("must have a valid marker address"),
        );
    }

    #[test]
    fn test_marker_share_sale_invalid_marker_denom() {
        assert_validation_failure(
            "ask order does not include a valid marker denom",
            &mock_ask(mock_ask_marker_share_single("marker", "", 10, &[], 10)),
            marker_share_sale_error("must have a specified denom"),
        );
    }

    #[test]
    fn test_marker_share_sale_zero_remaining_shares() {
        assert_validation_failure(
            "ask order indicates that referenced marker has no remaining held shares",
            &mock_ask(mock_ask_marker_share_single("marker", "denom", 0, &[], 10)),
            marker_share_sale_error("must refer to a marker with at least one of its coins held"),
        );
    }

    #[test]
    fn test_marker_share_sale_empty_quote_per_share() {
        assert_validation_failure(
            "ask order includes an empty quote per share",
            &mock_ask(mock_ask_marker_share_single("marker", "denom", 10, &[], 10)),
            marker_share_sale_error("must have a quote per share"),
        );
    }

    #[test]
    fn test_marker_share_sale_quote_per_share_includes_invalid_coins() {
        assert_validation_failure(
            "ask order includes quote per share with zero amount in coin",
            &mock_ask(mock_ask_marker_share_single(
                "marker",
                "denom",
                10,
                &coins(0, "nhash"),
                10,
            )),
            zero_coin_error("nhash", "AskCollateral Quote per Share Coin"),
        );
        assert_validation_failure(
            "ask order includes quote per share with invalid denom in coin",
            &mock_ask(mock_ask_marker_share_single(
                "marker",
                "denom",
                10,
                &coins(100, ""),
                10,
            )),
            blank_denom_error(100, "AskCollateral Quote per Share Coin"),
        );
    }

    #[test]
    fn test_marker_share_sale_removed_permissions_do_not_include_owner_address() {
        assert_validation_failure(
            "ask order does not specify that it removed the owner's permissions",
            &mock_ask(AskCollateral::marker_share_sale(
                Addr::unchecked("marker_address"),
                "denom",
                100,
                &coins(150, "nhash"),
                &[AccessGrant {
                    permissions: vec![],
                    address: Addr::unchecked("some rando"),
                }],
                ShareSaleType::single(100),
            )),
            marker_share_sale_error("does not have a permission for owner [asker]"),
        );
    }

    #[test]
    fn test_marker_share_sale_single_tx_share_count_is_zero() {
        assert_validation_failure(
            "ask order specifies a single transaction sale but it wants to sell zero shares",
            &mock_ask(mock_ask_marker_share_single("marker", "denom", 100, &[], 0)),
            marker_share_sale_error(
                "specified share count for single transaction must be greater than zero",
            ),
        );
    }

    #[test]
    fn test_marker_share_sale_single_tx_share_count_greater_than_remaining_shares() {
        assert_validation_failure(
            "ask order specifies a single transaction that wants to sell more shares than the marker has",
            &mock_ask(mock_ask_marker_share_single("marker", "denom", 100, &[], 101)),
            marker_share_sale_error("specified share count [101] cannot be greater than marker remaining shares [100]"),
        );
    }

    #[test]
    fn test_marker_share_sale_multi_tx_threshold_greater_than_remaining_shares() {
        assert_validation_failure(
            "ask order specifies a multiple transaction with a removal threshold greater than remaining shares",
            &mock_ask(mock_ask_marker_share_multi("marker", "denom", 100, &[], Some(101))),
            marker_share_sale_error("specified ask removal threshold [101] cannot be greater than marker remaining shares [100]"),
        );
    }

    #[test]
    fn test_scope_trade_missing_scope_address() {
        assert_validation_failure(
            "ask order does not include a valid scope address",
            &mock_ask(mock_ask_scope_trade("", &coins(100, "nhash"))),
            scope_trade_error("must have a valid scope address"),
        );
    }

    #[test]
    fn test_scope_trade_empty_quote() {
        assert_validation_failure(
            "ask order does not include any quote coins",
            &mock_ask(mock_ask_scope_trade("scope", &[])),
            scope_trade_error("must have a valid quote specified"),
        );
    }

    #[test]
    fn test_scope_trade_quote_includes_invalid_coins() {
        assert_validation_failure(
            "ask order includes quote with zero amount in coin",
            &mock_ask(mock_ask_scope_trade("scope", &coins(0, "nhash"))),
            zero_coin_error("nhash", "AskCollateral Quote"),
        );
        assert_validation_failure(
            "ask order includes quote with invalid denom in coin",
            &mock_ask(mock_ask_scope_trade("scope", &coins(100, ""))),
            blank_denom_error(100, "AskCollateral Quote"),
        );
    }

    fn coin_trade_error<S: Into<String>>(suffix: S) -> String {
        format!("AskOrder [ask_id] of type coin trade {}", suffix.into())
    }

    fn marker_trade_error<S: Into<String>>(suffix: S) -> String {
        format!("AskOrder [ask_id] of type marker trade {}", suffix.into())
    }

    fn marker_share_sale_error<S: Into<String>>(suffix: S) -> String {
        format!(
            "AskOrder [ask_id] of type marker share sale {}",
            suffix.into()
        )
    }

    fn scope_trade_error<S: Into<String>>(suffix: S) -> String {
        format!("AskOrder [ask_id] of type scope trade {}", suffix.into())
    }

    fn zero_coin_error<S1: Into<String>, S2: Into<String>>(denom: S1, coin_type: S2) -> String {
        format!(
            "Zero amounts not allowed on coins. Coin denom [{}] and type [{}] for AskOrder [ask_id]",
            denom.into(),
            coin_type.into(),
        )
    }

    fn blank_denom_error<S: Into<String>>(coin_amount: u128, coin_type: S) -> String {
        format!(
            "Blank denoms not allowed on coins. Coin amount [{}] and type [{}] for AskOrder [ask_id]",
            coin_amount,
            coin_type.into(),
        )
    }

    fn assert_validation_failure<S1: Into<String>, S2: Into<String>>(
        test_name: S1,
        ask_order: &AskOrder,
        expected_error_message: S2,
    ) {
        let test_name = test_name.into();
        let message = expected_error_message.into();
        let messages = match validate_ask_order(&ask_order) {
            Err(e) => match e {
                ContractError::ValidationError { messages } => messages,
                e => panic!(
                    "{}: Expected message [{}], but got unexpected error type instead during validation: {:?}",
                    test_name, message, e,
                ),
            },
            Ok(_) => panic!(
                "{}: Expected message [{}] to be output for input values, but validation passed",
                test_name, message,
            ),
        };
        assert!(
            messages.contains(&message),
            "expected message [{}] to be in result list {:?} for ask order [{}]",
            &message,
            &messages,
            &ask_order.id,
        );
    }
}
