use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::types::request::bid_types::bid_collateral::BidCollateral;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::slice::Iter;

const REQUEST_TYPE_COIN_TRADE: &str = "coin_trade";
const REQUEST_TYPE_MARKER_TRADE: &str = "marker_trade";
const REQUEST_TYPE_MARKER_SHARE_SALE: &str = "marker_share_sale";
const REQUEST_TYPE_SCOPE_TRADE: &str = "scope_trade";

static REQUEST_TYPES: [RequestType; 4] = [
    RequestType::CoinTrade,
    RequestType::MarkerTrade,
    RequestType::MarkerShareSale,
    RequestType::ScopeTrade,
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    CoinTrade,
    MarkerTrade,
    MarkerShareSale,
    ScopeTrade,
}
impl RequestType {
    pub fn from_ask_collateral(collateral: &AskCollateral) -> Self {
        match collateral {
            AskCollateral::CoinTrade(_) => Self::CoinTrade,
            AskCollateral::MarkerTrade(_) => Self::MarkerTrade,
            AskCollateral::MarkerShareSale(_) => Self::MarkerShareSale,
            AskCollateral::ScopeTrade(_) => Self::ScopeTrade,
        }
    }

    pub fn from_bid_collateral(collateral: &BidCollateral) -> Self {
        match collateral {
            BidCollateral::CoinTrade(_) => Self::CoinTrade,
            BidCollateral::MarkerTrade(_) => Self::MarkerTrade,
            BidCollateral::MarkerShareSale(_) => Self::MarkerShareSale,
            BidCollateral::ScopeTrade(_) => Self::ScopeTrade,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Self::CoinTrade => REQUEST_TYPE_COIN_TRADE,
            Self::MarkerTrade => REQUEST_TYPE_MARKER_TRADE,
            Self::MarkerShareSale => REQUEST_TYPE_MARKER_SHARE_SALE,
            Self::ScopeTrade => REQUEST_TYPE_SCOPE_TRADE,
        }
    }

    pub fn iterator() -> Iter<'static, RequestType> {
        REQUEST_TYPES.iter()
    }
}
