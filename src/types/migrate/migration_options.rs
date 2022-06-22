use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrationOptions {
    pub new_admin_address: Option<String>,
}
impl MigrationOptions {
    pub fn has_changes(&self) -> bool {
        self.new_admin_address.is_some()
    }
}
