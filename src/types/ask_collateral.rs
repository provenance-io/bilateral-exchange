use cosmwasm_std::{Addr, Coin, Uint128};
use provwasm_std::AccessGrant;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AskCollateral {
    Coin(CoinAskCollateral),
    Marker(MarkerAskCollateral),
}
impl AskCollateral {
    pub fn coin(base: &[Coin], quote: &[Coin]) -> Self {
        Self::Coin(CoinAskCollateral::new(base, quote))
    }

    pub fn marker<S: Into<String>>(
        address: Addr,
        denom: S,
        share_count: u128,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
    ) -> Self {
        Self::Marker(MarkerAskCollateral::new(
            address,
            denom,
            share_count,
            quote_per_share,
            removed_permissions,
        ))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinAskCollateral {
    pub base: Vec<Coin>,
    pub quote: Vec<Coin>,
}
impl CoinAskCollateral {
    fn new(base: &[Coin], quote: &[Coin]) -> Self {
        Self {
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerAskCollateral {
    pub address: Addr,
    pub denom: String,
    pub share_count: Uint128,
    pub quote_per_share: Vec<Coin>,
    pub removed_permissions: Vec<AccessGrant>,
}
impl MarkerAskCollateral {
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
