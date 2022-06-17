use crate::types::bid_collateral::BidCollateral;
use crate::types::constants::{
    ASK_TYPE_COIN_TRADE, ASK_TYPE_MARKER_TRADE, BID_TYPE_COIN_TRADE, BID_TYPE_MARKER_SHARE_SALE,
    BID_TYPE_MARKER_TRADE, BID_TYPE_SCOPE_TRADE, UNKNOWN_TYPE,
};
use crate::types::error::ContractError;
use crate::types::request_descriptor::RequestDescriptor;
use crate::util::extensions::ResultExtensions;
use crate::validation::bid_order_validation::validate_bid_order;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BidOrder {
    pub id: String,
    pub bid_type: String,
    pub owner: Addr,
    pub collateral: BidCollateral,
    pub descriptor: Option<RequestDescriptor>,
}
impl BidOrder {
    pub fn new<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: BidCollateral,
        descriptor: Option<RequestDescriptor>,
    ) -> Result<Self, ContractError> {
        let bid_order = Self::new_unchecked(id, owner, collateral, descriptor);
        validate_bid_order(&bid_order)?;
        bid_order.to_ok()
    }

    pub fn new_unchecked<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: BidCollateral,
        descriptor: Option<RequestDescriptor>,
    ) -> Self {
        Self {
            id: id.into(),
            bid_type: match collateral {
                BidCollateral::CoinTrade { .. } => BID_TYPE_COIN_TRADE.to_string(),
                BidCollateral::MarkerTrade { .. } => BID_TYPE_MARKER_TRADE.to_string(),
                BidCollateral::MarkerShareSale { .. } => BID_TYPE_MARKER_SHARE_SALE.to_string(),
                BidCollateral::ScopeTrade { .. } => BID_TYPE_SCOPE_TRADE.to_string(),
            },
            owner,
            collateral,
            descriptor,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }

    pub fn get_matching_ask_type(&self) -> &str {
        match self.bid_type.as_str() {
            BID_TYPE_COIN_TRADE => ASK_TYPE_COIN_TRADE,
            BID_TYPE_MARKER_TRADE => ASK_TYPE_MARKER_TRADE,
            _ => UNKNOWN_TYPE,
        }
    }
}
