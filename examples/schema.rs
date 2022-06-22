use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use bilateral_exchange::storage::contract_info::ContractInfo;
use bilateral_exchange::types::core::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use bilateral_exchange::types::request::ask_types::ask_order::AskOrder;
use bilateral_exchange::types::request::bid_types::bid_order::BidOrder;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(AskOrder), &out_dir);
    export_schema(&schema_for!(BidOrder), &out_dir);
    export_schema(&schema_for!(ContractInfo), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
}
