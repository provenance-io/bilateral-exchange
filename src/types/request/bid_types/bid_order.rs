use crate::types::core::error::ContractError;
use crate::types::request::bid_types::bid_collateral::BidCollateral;
use crate::types::request::request_descriptor::RequestDescriptor;
use crate::types::request::request_type::RequestType;
use crate::types::request::search::Searchable;
use crate::util::extensions::ResultExtensions;
use crate::validation::bid_order_validation::validate_bid_order;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BidOrder {
    pub id: String,
    pub bid_type: RequestType,
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
            bid_type: RequestType::from_bid_collateral(&collateral),
            owner,
            collateral,
            descriptor,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }
}
impl Searchable for BidOrder {}
