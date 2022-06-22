use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::types::request::ask_types::ask_order::AskOrder;
use crate::types::request::bid_types::bid_collateral::BidCollateral;
use crate::types::request::bid_types::bid_order::BidOrder;
use crate::types::request::request_descriptor::RequestDescriptor;
use crate::types::request::share_sale_type::ShareSaleType;
use cosmwasm_std::{Addr, Coin};
use provwasm_std::{AccessGrant, MarkerAccess};

pub fn replace_ask_quote(ask_order: &mut AskOrder, quote: &[Coin]) {
    match ask_order.collateral {
        AskCollateral::CoinTrade(ref mut collateral) => collateral.quote = quote.to_vec(),
        AskCollateral::MarkerTrade(ref mut collateral) => {
            collateral.quote_per_share = quote.to_vec()
        }
        AskCollateral::MarkerShareSale(ref mut collateral) => {
            collateral.quote_per_share = quote.to_vec()
        }
        AskCollateral::ScopeTrade(ref mut collateral) => collateral.quote = quote.to_vec(),
    };
}

pub fn replace_bid_quote(bid_order: &mut BidOrder, quote: &[Coin]) {
    match bid_order.collateral {
        BidCollateral::CoinTrade(ref mut collateral) => collateral.quote = quote.to_vec(),
        BidCollateral::MarkerTrade(ref mut collateral) => collateral.quote = quote.to_vec(),
        BidCollateral::MarkerShareSale(ref mut collateral) => collateral.quote = quote.to_vec(),
        BidCollateral::ScopeTrade(ref mut collateral) => collateral.quote = quote.to_vec(),
    };
}

pub fn mock_ask(collateral: AskCollateral) -> AskOrder {
    AskOrder::new_unchecked("ask_id", Addr::unchecked("asker"), collateral, None)
}

pub fn mock_ask_with_descriptor(
    collateral: AskCollateral,
    descriptor: RequestDescriptor,
) -> AskOrder {
    AskOrder::new_unchecked(
        "ask_id",
        Addr::unchecked("asker"),
        collateral,
        Some(descriptor),
    )
}

pub fn mock_ask_marker_trade<S1: Into<String>, S2: Into<String>>(
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

pub fn mock_ask_marker_share_single<S1: Into<String>, S2: Into<String>>(
    addr: S1,
    denom: S2,
    remaining_shares: u128,
    share_quote: &[Coin],
    share_sale_amount: u128,
) -> AskCollateral {
    AskCollateral::marker_share_sale(
        Addr::unchecked(addr),
        denom,
        remaining_shares,
        share_quote,
        &[AccessGrant {
            address: Addr::unchecked("asker"),
            permissions: vec![MarkerAccess::Admin],
        }],
        ShareSaleType::single(share_sale_amount),
    )
}

pub fn mock_ask_marker_share_multi<S1: Into<String>, S2: Into<String>>(
    addr: S1,
    denom: S2,
    remaining_shares: u128,
    share_quote: &[Coin],
    sale_cutoff: Option<u128>,
) -> AskCollateral {
    AskCollateral::marker_share_sale(
        Addr::unchecked(addr),
        denom,
        remaining_shares,
        share_quote,
        &[AccessGrant {
            address: Addr::unchecked("asker"),
            permissions: vec![MarkerAccess::Admin],
        }],
        ShareSaleType::multiple(sale_cutoff),
    )
}

pub fn mock_ask_scope_trade<S: Into<String>>(scope_address: S, quote: &[Coin]) -> AskCollateral {
    AskCollateral::scope_trade(scope_address, quote)
}

pub fn mock_bid(collateral: BidCollateral) -> BidOrder {
    BidOrder::new_unchecked("bid_id", Addr::unchecked("bidder"), collateral, None)
}

pub fn mock_bid_with_descriptor(
    collateral: BidCollateral,
    descriptor: RequestDescriptor,
) -> BidOrder {
    BidOrder::new_unchecked(
        "bid_id",
        Addr::unchecked("bidder"),
        collateral,
        Some(descriptor),
    )
}

pub fn mock_bid_marker_trade<S1: Into<String>, S2: Into<String>>(
    addr: S1,
    denom: S2,
    quote: &[Coin],
) -> BidCollateral {
    BidCollateral::marker_trade(Addr::unchecked(addr), denom, quote)
}

pub fn mock_bid_marker_share<S1: Into<String>, S2: Into<String>>(
    addr: S1,
    denom: S2,
    share_count: u128,
    quote: &[Coin],
) -> BidCollateral {
    BidCollateral::marker_share_sale(Addr::unchecked(addr), denom, share_count, quote)
}

pub fn mock_bid_scope_trade<S: Into<String>>(scope_address: S, quote: &[Coin]) -> BidCollateral {
    BidCollateral::scope_trade(scope_address, quote)
}
