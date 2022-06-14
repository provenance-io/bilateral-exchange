use crate::types::ask_base::{ASK_TYPES, COIN_ASK_TYPE, MARKER_ASK_TYPE};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{Addr, Coin, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use provwasm_std::{AccessGrant, MarkerAccess, MarkerStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const NAMESPACE_ASK_PK: &str = "ask";
const NAMESPACE_OWNER_IDX: &str = "ask__owner";
const NAMESPACE_TYPE_IDX: &str = "ask__type";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AskCollateral {
    Coin {
        base: Vec<Coin>,
    },
    Marker {
        denom: String,
        removed_permissions: Vec<AccessGrant>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AskOrder {
    pub id: String,
    pub ask_type: String,
    pub owner: Addr,
    pub collateral: AskCollateral,
}
impl AskOrder {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        id: S1,
        ask_type: S2,
        owner: Addr,
        collateral: AskCollateral,
    ) -> Result<Self, ContractError> {
        let ask_type = ask_type.into();
        let valid_structure = match &ask_type {
            COIN_ASK_TYPE => matches!(collateral, AskCollateral::Coin { .. }),
            MARKER_ASK_TYPE => matches!(collateral, AskCollateral::Marker { .. }),
            _ => false,
        };
        if !valid_structure {
            return ContractError::InvalidField {
                message: format!(
                    "Invalid storage structure provided for ask type [{}]",
                    ask_type
                ),
            }
            .to_err();
        }
        Self {
            id: id.into(),
            ask_type: ask_type.into(),
            owner,
            collateral,
        }
        .to_ok()
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }
}

fn validate_ask_order(ask_order: &AskOrder) -> Result<(), ContractError> {
    if ask_order.id.is_empty() {
        return ContractError::InvalidField {
            message: "id for AskOrder must not be empty".to_string(),
        }
        .to_err();
    }
    if !ASK_TYPES.contains(&&**&ask_order.ask_type) {
        return ContractError::InvalidField {
            message: format!(
                "ask type [{}] for AskOrder is invalid. Must be one of: {:?}",
                ask_order.ask_type, ASK_TYPES
            ),
        }
        .to_err();
    }
    ().to_ok()
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

pub fn insert_ask(storage: &mut dyn Storage, ask_order: &AskOrder) -> Result<(), ContractError> {
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
