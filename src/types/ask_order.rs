use crate::types::ask_collateral::AskCollateral;
use crate::types::constants::{
    ASK_TYPE_COIN_TRADE, ASK_TYPE_MARKER_SHARE_SALE, ASK_TYPE_MARKER_TRADE, ASK_TYPE_SCOPE_TRADE,
    BID_TYPE_COIN_TRADE, BID_TYPE_MARKER_TRADE, UNKNOWN_TYPE,
};
use crate::types::error::ContractError;
use crate::types::request_descriptor::RequestDescriptor;
use crate::util::extensions::ResultExtensions;
use crate::validation::ask_order_validation::validate_ask_order;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AskOrder {
    pub id: String,
    pub ask_type: String,
    pub owner: Addr,
    pub collateral: AskCollateral,
    pub descriptor: Option<RequestDescriptor>,
}
impl AskOrder {
    pub fn new<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: AskCollateral,
        descriptor: Option<RequestDescriptor>,
    ) -> Result<Self, ContractError> {
        let ask_order = Self::new_unchecked(id, owner, collateral, descriptor);
        validate_ask_order(&ask_order)?;
        ask_order.to_ok()
    }

    pub fn new_unchecked<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: AskCollateral,
        descriptor: Option<RequestDescriptor>,
    ) -> Self {
        Self {
            id: id.into(),
            ask_type: match collateral {
                AskCollateral::CoinTrade { .. } => ASK_TYPE_COIN_TRADE.to_string(),
                AskCollateral::MarkerTrade { .. } => ASK_TYPE_MARKER_TRADE.to_string(),
                AskCollateral::MarkerShareSale { .. } => ASK_TYPE_MARKER_SHARE_SALE.to_string(),
                AskCollateral::ScopeTrade { .. } => ASK_TYPE_SCOPE_TRADE.to_string(),
            },
            owner,
            collateral,
            descriptor,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }

    pub fn get_matching_bid_type(&self) -> &str {
        match self.ask_type.as_str() {
            ASK_TYPE_COIN_TRADE => BID_TYPE_COIN_TRADE,
            ASK_TYPE_MARKER_TRADE => BID_TYPE_MARKER_TRADE,
            _ => UNKNOWN_TYPE,
        }
    }
}
