use crate::types::core::error::ContractError;
use crate::types::request::ask_types::ask_collateral::AskCollateral;
use crate::types::request::request_descriptor::RequestDescriptor;
use crate::types::request::request_type::RequestType;
use crate::types::request::search::Searchable;
use crate::util::extensions::ResultExtensions;
use crate::validation::ask_order_validation::validate_ask_order;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AskOrder {
    pub id: String,
    pub ask_type: RequestType,
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
            ask_type: RequestType::from_ask_collateral(&collateral),
            owner,
            collateral,
            descriptor,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }
}
impl Searchable for AskOrder {}
