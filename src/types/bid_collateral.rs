use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BidCollateral {
    Coin(CoinBidCollateral),
    Marker(MarkerBidCollateral),
}
impl BidCollateral {
    pub fn coin(base: &[Coin], quote: &[Coin]) -> Self {
        Self::Coin(CoinBidCollateral::new(base, quote))
    }

    pub fn marker<S: Into<String>>(address: Addr, denom: S, quote: &[Coin]) -> Self {
        Self::Marker(MarkerBidCollateral::new(address, denom, quote))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinBidCollateral {
    pub base: Vec<Coin>,
    pub quote: Vec<Coin>,
}
impl CoinBidCollateral {
    fn new(base: &[Coin], quote: &[Coin]) -> Self {
        Self {
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerBidCollateral {
    pub address: Addr,
    pub denom: String,
    pub quote: Vec<Coin>,
}
impl MarkerBidCollateral {
    fn new<S: Into<String>>(address: Addr, denom: S, quote: &[Coin]) -> Self {
        Self {
            address,
            denom: denom.into(),
            quote: quote.to_owned(),
        }
    }
}
