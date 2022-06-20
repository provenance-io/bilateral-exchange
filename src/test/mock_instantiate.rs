use crate::storage::contract_info::{set_contract_info, ContractInfo};
use cosmwasm_std::{Addr, Storage};

pub const DEFAULT_ADMIN_ADDRESS: &str = "contract_admin";
pub const DEFAULT_CONTRACT_BIND_NAME: &str = "contract_bind_name";
pub const DEFAULT_CONTRACT_NAME: &str = "contract_name";

pub struct TestInstantiate {
    pub admin_address: String,
    pub contract_bind_name: String,
    pub contract_name: String,
}
impl Default for TestInstantiate {
    fn default() -> Self {
        Self {
            admin_address: DEFAULT_ADMIN_ADDRESS.to_string(),
            contract_bind_name: DEFAULT_CONTRACT_BIND_NAME.to_string(),
            contract_name: DEFAULT_CONTRACT_NAME.to_string(),
        }
    }
}

pub fn default_instantiate(storage: &mut dyn Storage) {
    test_instantiate(storage, TestInstantiate::default())
}

pub fn test_instantiate(storage: &mut dyn Storage, instantiate: TestInstantiate) {
    set_contract_info(
        storage,
        &ContractInfo::new(
            Addr::unchecked(instantiate.admin_address),
            instantiate.contract_bind_name,
            instantiate.contract_name,
        ),
    )
    .expect("expected contract info to be created without issue")
}
