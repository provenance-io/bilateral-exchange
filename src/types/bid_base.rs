use crate::types::constants::{BID_TYPE_COIN, BID_TYPE_MARKER};
use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BidBase {
    Coin(CoinBidBase),
    Marker(MarkerBidBase),
}
impl BidBase {
    pub fn new_coin<S: Into<String>>(
        id: S,
        base: Vec<Coin>,
        effective_time: Option<Timestamp>,
    ) -> Self {
        Self::Coin(CoinBidBase::new(id, base, effective_time))
    }

    pub fn new_marker<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        base: Coin,
        effective_time: Option<Timestamp>,
    ) -> Self {
        Self::Marker(MarkerBidBase::new(id, denom, base, effective_time))
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
pub struct CoinBidBase {
    pub id: String,
    pub base: Vec<Coin>,
    pub effective_time: Option<Timestamp>,
}
impl CoinBidBase {
    pub fn new<S: Into<String>>(id: S, base: Vec<Coin>, effective_time: Option<Timestamp>) -> Self {
        Self {
            id: id.into(),
            base,
            effective_time,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerBidBase {
    pub id: String,
    pub denom: String,
    pub base: Coin,
    pub effective_time: Option<Timestamp>,
}
impl MarkerBidBase {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        base: Coin,
        effective_time: Option<Timestamp>,
    ) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
            base,
            effective_time,
        }
    }
}
