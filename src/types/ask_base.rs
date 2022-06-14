use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const COIN_ASK_TYPE: &str = "coin";
pub const MARKER_ASK_TYPE: &str = "marker";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AskBase {
    Coin(CoinAskBase),
    Marker(MarkerAskBase),
}
impl AskBase {
    pub fn new_coin<S: Into<String>>(id: S, quote: Vec<Coin>) -> Self {
        Self::Coin(CoinAskBase::new(id, quote))
    }

    pub fn new_marker<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        price_per_share: Coin,
    ) -> Self {
        Self::Marker(MarkerAskBase::new(id, denom, price_per_share))
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
            Self::Coin(_) => COIN_ASK_TYPE,
            Self::Marker(_) => MARKER_ASK_TYPE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinAskBase {
    pub id: String,
    pub quote: Vec<Coin>,
}
impl CoinAskBase {
    pub fn new<S: Into<String>>(id: S, quote: Vec<Coin>) -> Self {
        Self {
            id: id.into(),
            quote,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerAskBase {
    pub id: String,
    pub denom: String,
    pub price_per_share: Coin,
}
impl MarkerAskBase {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        denom: S2,
        price_per_share: Coin,
    ) -> Self {
        Self {
            id: id.into(),
            denom: denom.into(),
            price_per_share,
        }
    }
}
