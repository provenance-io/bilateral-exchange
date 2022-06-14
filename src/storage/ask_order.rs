use crate::types::ask_base::{COIN_ASK_TYPE, MARKER_ASK_TYPE};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::validation::ask_order_validation::validate_ask_order;
use cosmwasm_std::{Addr, Coin, StdError, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use provwasm_std::{AccessGrant, MarkerAccess, MarkerStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const NAMESPACE_ASK_PK: &str = "ask";
const NAMESPACE_OWNER_IDX: &str = "ask__owner";
const NAMESPACE_TYPE_IDX: &str = "ask__type";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AskOrder {
    pub id: String,
    pub ask_type: String,
    pub owner: Addr,
    pub collateral: AskCollateral,
}
impl AskOrder {
    pub fn new<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: AskCollateral,
    ) -> Result<Self, ContractError> {
        let ask_order = Self::new_unchecked(id, owner, collateral);
        validate_ask_order(&ask_order)?;
        ask_order.to_ok()
    }

    pub fn new_unchecked<S: Into<String>>(id: S, owner: Addr, collateral: AskCollateral) -> Self {
        Self {
            id: id.into(),
            ask_type: match collateral {
                AskCollateral::Coin { .. } => COIN_ASK_TYPE.to_string(),
                AskCollateral::Marker { .. } => MARKER_ASK_TYPE.to_string(),
            },
            owner,
            collateral,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AskCollateral {
    Coin {
        base: Vec<Coin>,
        quote: Vec<Coin>,
    },
    Marker {
        address: Addr,
        denom: String,
        removed_permissions: Vec<AccessGrant>,
    },
}
impl AskCollateral {
    pub fn coin(base: Vec<Coin>, quote: Vec<Coin>) -> Self {
        Self::Coin { base, quote }
    }

    pub fn marker<S: Into<String>>(
        address: Addr,
        denom: S,
        removed_permissions: Vec<AccessGrant>,
    ) -> Self {
        Self::Marker {
            address,
            denom: denom.into(),
            removed_permissions,
        }
    }
}

pub struct AskOrderIndices<'a> {
    owner_index: MultiIndex<'a, String, AskOrder>,
    type_index: MultiIndex<'a, String, AskOrder>,
}
impl<'a> IndexList<AskOrder> for AskOrderIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AskOrder>> + '_> {
        let v: Vec<&dyn Index<AskOrder>> = vec![&self.owner_index, &self.type_index];
        Box::new(v.into_iter())
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
            |ask: &AskOrder| ask.ask_type.clone(),
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
                existing_ask.owner.to_string(),
            ),
        }
        .to_err();
    }
    state
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
