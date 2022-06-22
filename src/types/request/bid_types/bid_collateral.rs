use crate::types::core::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{Addr, Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BidCollateral {
    CoinTrade(CoinTradeBidCollateral),
    MarkerTrade(MarkerTradeBidCollateral),
    MarkerShareSale(MarkerShareSaleBidCollateral),
    ScopeTrade(ScopeTradeBidCollateral),
}
impl BidCollateral {
    pub fn coin_trade(base: &[Coin], quote: &[Coin]) -> Self {
        Self::CoinTrade(CoinTradeBidCollateral::new(base, quote))
    }

    pub fn marker_trade<S: Into<String>>(address: Addr, denom: S, quote: &[Coin]) -> Self {
        Self::MarkerTrade(MarkerTradeBidCollateral::new(address, denom, quote))
    }

    pub fn marker_share_sale<S: Into<String>>(
        address: Addr,
        denom: S,
        share_count: u128,
        quote: &[Coin],
    ) -> Self {
        Self::MarkerShareSale(MarkerShareSaleBidCollateral::new(
            address,
            denom,
            share_count,
            quote,
        ))
    }

    pub fn scope_trade<S: Into<String>>(scope_address: S, quote: &[Coin]) -> Self {
        Self::ScopeTrade(ScopeTradeBidCollateral::new(scope_address, quote))
    }

    pub fn get_coin_trade(&self) -> Result<&CoinTradeBidCollateral, ContractError> {
        match self {
            Self::CoinTrade(collateral) => collateral.to_ok(),
            _ => ContractError::invalid_type("expected coin trade bid collateral").to_err(),
        }
    }

    pub fn get_marker_trade(&self) -> Result<&MarkerTradeBidCollateral, ContractError> {
        match self {
            Self::MarkerTrade(collateral) => collateral.to_ok(),
            _ => ContractError::invalid_type("expected marker trade bid collateral").to_err(),
        }
    }

    pub fn get_marker_share_sale(&self) -> Result<&MarkerShareSaleBidCollateral, ContractError> {
        match self {
            Self::MarkerShareSale(collateral) => collateral.to_ok(),
            _ => ContractError::invalid_type("expected marker share sale bid collateral").to_err(),
        }
    }

    pub fn get_scope_trade(&self) -> Result<&ScopeTradeBidCollateral, ContractError> {
        match self {
            Self::ScopeTrade(collateral) => collateral.to_ok(),
            _ => ContractError::invalid_type("expected scope trade bid collateral").to_err(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CoinTradeBidCollateral {
    pub base: Vec<Coin>,
    pub quote: Vec<Coin>,
}
impl CoinTradeBidCollateral {
    pub fn new(base: &[Coin], quote: &[Coin]) -> Self {
        Self {
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerTradeBidCollateral {
    pub address: Addr,
    pub denom: String,
    pub quote: Vec<Coin>,
}
impl MarkerTradeBidCollateral {
    pub fn new<S: Into<String>>(address: Addr, denom: S, quote: &[Coin]) -> Self {
        Self {
            address,
            denom: denom.into(),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarkerShareSaleBidCollateral {
    pub address: Addr,
    pub denom: String,
    pub share_count: Uint128,
    pub quote: Vec<Coin>,
}
impl MarkerShareSaleBidCollateral {
    pub fn new<S: Into<String>>(
        address: Addr,
        denom: S,
        share_count: u128,
        quote: &[Coin],
    ) -> Self {
        Self {
            address,
            denom: denom.into(),
            share_count: Uint128::new(share_count),
            quote: quote.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeTradeBidCollateral {
    pub scope_address: String,
    pub quote: Vec<Coin>,
}
impl ScopeTradeBidCollateral {
    pub fn new<S: Into<String>>(scope_address: S, quote: &[Coin]) -> Self {
        Self {
            scope_address: scope_address.into(),
            quote: quote.to_owned(),
        }
    }
}
