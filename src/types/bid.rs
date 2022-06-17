use crate::types::constants::{
    BID_TYPE_COIN_TRADE, BID_TYPE_MARKER_SHARE_SALE, BID_TYPE_MARKER_TRADE, BID_TYPE_SCOPE_TRADE,
};
use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bid {
    CoinTrade(CoinTradeBid),
    MarkerTrade(MarkerTradeBid),
    MarkerShareSale(MarkerShareSaleBid),
    ScopeTrade(ScopeTradeBid),
}
impl Bid {
    pub fn new_coin<S: Into<String>>(id: S, base: &[Coin]) -> Self {
        Self::CoinTrade(CoinTradeBid::new(id, base))
    }

    pub fn new_marker<S1: Into<String>, S2: Into<String>>(id: S1, denom: S2) -> Self {
        Self::MarkerTrade(MarkerTradeBid::new(id, denom))
    }

    pub fn new_marker_share_sale<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        share_count: u128,
    ) -> Self {
        Self::MarkerShareSale(MarkerShareSaleBid::new(id, denom, share_count))
    }

    pub fn get_id(&self) -> &str {
        match self {
            Self::CoinTrade(trade) => &trade.id,
            Self::MarkerTrade(trade) => &trade.id,
            Self::MarkerShareSale(sale) => &sale.id,
            Self::ScopeTrade(trade) => &trade.id,
        }
    }

    pub fn get_storage_key(&self) -> &[u8] {
        self.get_id().as_bytes()
    }

    pub fn get_bid_type(&self) -> &str {
        match self {
            Self::CoinTrade(_) => BID_TYPE_COIN_TRADE,
            Self::MarkerTrade(_) => BID_TYPE_MARKER_TRADE,
            Self::MarkerShareSale(_) => BID_TYPE_MARKER_SHARE_SALE,
            Self::ScopeTrade(_) => BID_TYPE_SCOPE_TRADE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinTradeBid {
    pub id: String,
    pub base: Vec<Coin>,
}
impl CoinTradeBid {
    pub fn new<S: Into<String>>(id: S, base: &[Coin]) -> Self {
        Self {
            id: id.into(),
            base: base.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerTradeBid {
    pub id: String,
    pub denom: String,
}
impl MarkerTradeBid {
    pub fn new<S1: Into<String>, S2: Into<String>>(id: S1, denom: S2) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerShareSaleBid {
    pub id: String,
    pub denom: String,
    pub share_count: Uint128,
}
impl MarkerShareSaleBid {
    pub fn new<S1: Into<String>, S2: Into<String>>(id: S1, denom: S2, share_count: u128) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
            share_count: Uint128::new(share_count),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeTradeBid {
    pub id: String,
    pub scope_address: String,
}
impl ScopeTradeBid {
    pub fn new<S1: Into<String>, S2: Into<String>>(id: S1, scope_address: S2) -> Self {
        Self {
            id: id.into(),
            scope_address: scope_address.into(),
        }
    }
}
