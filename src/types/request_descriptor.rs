use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RequestDescriptor {
    pub(crate) description: Option<String>,
    pub(crate) effective_time: Option<Timestamp>,
}
