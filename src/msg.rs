use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::BaseType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bind_name: String,
    pub contract_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CancelAsk {
        id: String,
    },
    CancelBid {
        id: String,
    },
    CreateAsk {
        id: String,
        quote: Vec<Coin>,
        base: Option<BaseType>,
    },
    CreateBid {
        id: String,
        base: BaseType,
        effective_time: Option<Timestamp>,
    },
    ExecuteMatch {
        ask_id: String,
        bid_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAsk { id: String },
    GetBid { id: String },
    GetContractInfo {},
}
