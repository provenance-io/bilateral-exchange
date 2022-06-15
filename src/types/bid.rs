use crate::types::constants::{BID_TYPE_COIN, BID_TYPE_MARKER};
use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bid {
    Coin(CoinBid),
    Marker(MarkerBid),
}
impl Bid {
    pub fn new_coin<S: Into<String>>(id: S, base: &[Coin]) -> Self {
        Self::Coin(CoinBid::new(id, base))
    }

    pub fn new_marker<S1: Into<String>, S2: Into<String>>(id: S1, denom: S2) -> Self {
        Self::Marker(MarkerBid::new(id, denom))
    }

    pub fn get_id(&self) -> &str {
        match self {
            Self::Coin(base) => &base.id,
            Self::Marker(base) => &base.id,
        }
    }

    pub fn get_storage_key(&self) -> &[u8] {
        self.get_id().as_bytes()
    }

    pub fn get_bid_type(&self) -> &str {
        match self {
            Self::Coin(_) => BID_TYPE_COIN,
            Self::Marker(_) => BID_TYPE_MARKER,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinBid {
    pub id: String,
    pub base: Vec<Coin>,
}
impl CoinBid {
    pub fn new<S: Into<String>>(id: S, base: &[Coin]) -> Self {
        Self {
            id: id.into(),
            base: base.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerBid {
    pub id: String,
    pub denom: String,
}
impl MarkerBid {
    pub fn new<S1: Into<String>, S2: Into<String>>(id: S1, denom: S2) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
        }
    }
}
