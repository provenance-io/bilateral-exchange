use crate::types::constants::{ASK_TYPE_COIN, ASK_TYPE_MARKER};
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ask {
    Coin(CoinAsk),
    Marker(MarkerAsk),
}
impl Ask {
    pub fn new_coin<S: Into<String>>(id: S, quote: &[Coin]) -> Self {
        Self::Coin(CoinAsk::new(id, quote))
    }

    pub fn new_marker<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        quote_per_share: &[Coin],
    ) -> Self {
        Self::Marker(MarkerAsk::new(id, denom, quote_per_share))
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

    pub fn get_ask_type(&self) -> &str {
        match self {
            Self::Coin(_) => ASK_TYPE_COIN,
            Self::Marker(_) => ASK_TYPE_MARKER,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinAsk {
    pub id: String,
    pub quote: Vec<Coin>,
}
impl CoinAsk {
    pub fn new<S: Into<String>>(id: S, quote: &[Coin]) -> Self {
        Self {
            id: id.into(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerAsk {
    pub id: String,
    pub denom: String,
    pub quote_per_share: Vec<Coin>,
}
impl MarkerAsk {
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
