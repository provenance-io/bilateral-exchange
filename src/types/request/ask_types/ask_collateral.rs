use crate::types::request::share_sale_type::ShareSaleType;
use cosmwasm_std::{Addr, Coin, Uint128};
use provwasm_std::AccessGrant;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AskCollateral {
    CoinTrade(CoinTradeAskCollateral),
    MarkerTrade(MarkerTradeAskCollateral),
    MarkerShareSale(MarkerShareSaleAskCollateral),
    ScopeTrade(ScopeTradeAskCollateral),
}
impl AskCollateral {
    pub fn coin_trade(base: &[Coin], quote: &[Coin]) -> Self {
        Self::CoinTrade(CoinTradeAskCollateral::new(base, quote))
    }

    pub fn marker_trade<S: Into<String>>(
        address: Addr,
        denom: S,
        share_count: u128,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
    ) -> Self {
        Self::MarkerTrade(MarkerTradeAskCollateral::new(
            address,
            denom,
            share_count,
            quote_per_share,
            removed_permissions,
        ))
    }

    pub fn marker_share_sale<S: Into<String>>(
        address: Addr,
        denom: S,
        remaining_shares: u128,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
        sale_type: ShareSaleType,
    ) -> Self {
        Self::MarkerShareSale(MarkerShareSaleAskCollateral::new(
            address,
            denom,
            remaining_shares,
            quote_per_share,
            removed_permissions,
            sale_type,
        ))
    }

    pub fn scope_trade<S: Into<String>>(scope_address: S, quote: &[Coin]) -> Self {
        Self::ScopeTrade(ScopeTradeAskCollateral::new(scope_address, quote))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinTradeAskCollateral {
    pub base: Vec<Coin>,
    pub quote: Vec<Coin>,
}
impl CoinTradeAskCollateral {
    fn new(base: &[Coin], quote: &[Coin]) -> Self {
        Self {
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerTradeAskCollateral {
    pub address: Addr,
    pub denom: String,
    pub share_count: Uint128,
    pub quote_per_share: Vec<Coin>,
    pub removed_permissions: Vec<AccessGrant>,
}
impl MarkerTradeAskCollateral {
    fn new<S: Into<String>>(
        address: Addr,
        denom: S,
        share_count: u128,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
    ) -> Self {
        Self {
            address,
            denom: denom.into(),
            share_count: Uint128::new(share_count),
            quote_per_share: quote_per_share.to_owned(),
            removed_permissions: removed_permissions.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerShareSaleAskCollateral {
    pub address: Addr,
    pub denom: String,
    pub remaining_shares: Uint128,
    pub quote_per_share: Vec<Coin>,
    pub removed_permissions: Vec<AccessGrant>,
    pub sale_type: ShareSaleType,
}
impl MarkerShareSaleAskCollateral {
    fn new<S: Into<String>>(
        address: Addr,
        denom: S,
        remaining_shares: u128,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
        sale_type: ShareSaleType,
    ) -> Self {
        Self {
            address,
            denom: denom.into(),
            remaining_shares: Uint128::new(remaining_shares),
            quote_per_share: quote_per_share.to_owned(),
            removed_permissions: removed_permissions.to_owned(),
            sale_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeTradeAskCollateral {
    pub scope_address: String,
    pub quote: Vec<Coin>,
}
impl ScopeTradeAskCollateral {
    fn new<S: Into<String>>(scope_address: S, quote: &[Coin]) -> Self {
        Self {
            scope_address: scope_address.into(),
            quote: quote.to_owned(),
        }
    }
}
