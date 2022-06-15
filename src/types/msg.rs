use crate::types::ask_base::AskBase;
use crate::types::bid_base::BidBase;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bind_name: String,
    pub contract_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CancelAsk { id: String },
    CancelBid { id: String },
    CreateAsk { ask: AskBase },
    CreateBid { base: BidBase },
    ExecuteMatch { ask_id: String, bid_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAsk { id: String },
    GetBid { id: String },
    GetContractInfo {},
}
