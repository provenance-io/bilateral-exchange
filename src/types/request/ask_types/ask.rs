use crate::types::request::share_sale_type::ShareSaleType;
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ask {
    CoinTrade(CoinTradeAsk),
    MarkerTrade(MarkerTradeAsk),
    MarkerShareSale(MarkerShareSaleAsk),
    ScopeTrade(ScopeTradeAsk),
}
impl Ask {
    pub fn new_coin_trade<S: Into<String>>(id: S, quote: &[Coin]) -> Self {
        Self::CoinTrade(CoinTradeAsk::new(id, quote))
    }

    pub fn new_marker_trade<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        quote_per_share: &[Coin],
    ) -> Self {
        Self::MarkerTrade(MarkerTradeAsk::new(id, denom, quote_per_share))
    }

    pub fn new_marker_share_sale<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        quote_per_share: &[Coin],
        share_sale_type: ShareSaleType,
    ) -> Self {
        Self::MarkerShareSale(MarkerShareSaleAsk::new(
            id,
            denom,
            quote_per_share,
            share_sale_type,
        ))
    }

    pub fn new_scope_trade<S1: Into<String>, S2: Into<String>>(
        id: S1,
        scope_address: S2,
        quote: &[Coin],
    ) -> Self {
        Self::ScopeTrade(ScopeTradeAsk::new(id, scope_address, quote))
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinTradeAsk {
    pub id: String,
    pub quote: Vec<Coin>,
}
impl CoinTradeAsk {
    pub fn new<S: Into<String>>(id: S, quote: &[Coin]) -> Self {
        Self {
            id: id.into(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerTradeAsk {
    pub id: String,
    pub denom: String,
    pub quote_per_share: Vec<Coin>,
}
impl MarkerTradeAsk {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        quote_per_share: &[Coin],
    ) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
            quote_per_share: quote_per_share.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerShareSaleAsk {
    pub id: String,
    pub denom: String,
    pub quote_per_share: Vec<Coin>,
    pub share_sale_type: ShareSaleType,
}
impl MarkerShareSaleAsk {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        quote_per_share: &[Coin],
        share_sale_type: ShareSaleType,
    ) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
            quote_per_share: quote_per_share.to_owned(),
            share_sale_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeTradeAsk {
    pub id: String,
    pub scope_address: String,
    pub quote: Vec<Coin>,
}
impl ScopeTradeAsk {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        scope_address: S2,
        quote: &[Coin],
    ) -> Self {
        Self {
            id: id.into(),
            scope_address: scope_address.into(),
            quote: quote.to_owned(),
        }
    }
}
