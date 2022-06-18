use crate::storage::order_indices::OrderIndices;
use crate::types::ask_order::AskOrder;
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::Storage;
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

const NAMESPACE_ASK_PK: &str = "ask";
const NAMESPACE_OWNER_IDX: &str = "ask__owner";
const NAMESPACE_TYPE_IDX: &str = "ask__type";

pub struct AskOrderIndices<'a> {
    pub owner_index: MultiIndex<'a, String, AskOrder, String>,
    pub type_index: MultiIndex<'a, String, AskOrder, String>,
}
impl<'a> IndexList<AskOrder> for AskOrderIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AskOrder>> + '_> {
        let v: Vec<&dyn Index<AskOrder>> = vec![&self.owner_index, &self.type_index];
        Box::new(v.into_iter())
    }
}
impl<'a> OrderIndices<'a, AskOrder> for AskOrderIndices<'a> {
    fn owner_index(&self) -> &MultiIndex<'a, String, AskOrder, String> {
        &self.owner_index
    }

    fn type_index(&self) -> &MultiIndex<'a, String, AskOrder, String> {
        &self.type_index
    }
}

pub fn ask_orders<'a>() -> IndexedMap<'a, &'a [u8], AskOrder, AskOrderIndices<'a>> {
    let indices = AskOrderIndices {
        owner_index: MultiIndex::new(
            |ask: &AskOrder| ask.owner.clone().to_string(),
            NAMESPACE_ASK_PK,
            NAMESPACE_OWNER_IDX,
        ),
        type_index: MultiIndex::new(
            |ask: &AskOrder| ask.ask_type.get_name().to_string(),
            NAMESPACE_ASK_PK,
            NAMESPACE_TYPE_IDX,
        ),
    };
    IndexedMap::new(NAMESPACE_ASK_PK, indices)
}

pub fn insert_ask_order(
    storage: &mut dyn Storage,
    ask_order: &AskOrder,
) -> Result<(), ContractError> {
    let state = ask_orders();
    if let Ok(existing_ask) = state.load(storage, ask_order.get_pk()) {
        return ContractError::StorageError {
            message: format!(
                "an ask with id [{}] for owner [{}] already exists",
                existing_ask.id,
                existing_ask.owner.as_str(),
            ),
        }
        .to_err();
    }
    store_ask_order(storage, ask_order)
}

pub fn update_ask_order(
    storage: &mut dyn Storage,
    ask_order: &AskOrder,
) -> Result<(), ContractError> {
    let state = ask_orders();
    if let Ok(_) = state.load(storage, ask_order.get_pk()) {
        store_ask_order(storage, ask_order)
    } else {
        ContractError::StorageError {
            message: format!(
                "attempted to replace ask with id [{}] in storage, but no ask with that id existed",
                &ask_order.id
            ),
        }
        .to_err()
    }
}

fn store_ask_order(storage: &mut dyn Storage, ask_order: &AskOrder) -> Result<(), ContractError> {
    ask_orders()
        .replace(storage, ask_order.get_pk(), Some(ask_order), None)?
        .to_ok()
}

pub fn get_ask_order_by_id<S: Into<String>>(
    storage: &dyn Storage,
    id: S,
) -> Result<AskOrder, ContractError> {
    let id = id.into();
    ask_orders()
        .load(storage, id.as_bytes())
        .map_err(|e| ContractError::StorageError {
            message: format!("failed to find AskOrder by id [{}]: {:?}", id, e),
        })
}

pub fn delete_ask_order_by_id<S: Into<String>>(
    storage: &mut dyn Storage,
    id: S,
) -> Result<(), ContractError> {
    let id = id.into();
    ask_orders()
        .remove(storage, id.as_bytes())
        .map_err(|e| ContractError::StorageError {
            message: format!("failed to remove AskOrder by id [{}]: {:?}", id, e),
        })?;
    ().to_ok()
}
