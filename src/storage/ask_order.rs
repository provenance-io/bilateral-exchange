use crate::types::constants::{
    ASK_TYPE_COIN, ASK_TYPE_MARKER, BID_TYPE_COIN, BID_TYPE_MARKER, UNKNOWN_TYPE,
};
use crate::types::error::ContractError;
use crate::types::request_descriptor::RequestDescriptor;
use crate::util::extensions::ResultExtensions;
use crate::validation::ask_order_validation::validate_ask_order;
use cosmwasm_std::{Addr, Coin, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use provwasm_std::AccessGrant;
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
            ask_type: match collateral {
                AskCollateral::Coin { .. } => ASK_TYPE_COIN.to_string(),
                AskCollateral::Marker { .. } => ASK_TYPE_MARKER.to_string(),
            },
            owner,
            collateral,
            descriptor,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }

    pub fn get_matching_bid_type(&self) -> &str {
        match self.ask_type.as_str() {
            ASK_TYPE_COIN => BID_TYPE_COIN,
            ASK_TYPE_MARKER => BID_TYPE_MARKER,
            _ => UNKNOWN_TYPE,
        }
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
        quote_per_share: Vec<Coin>,
        removed_permissions: Vec<AccessGrant>,
    },
}
impl AskCollateral {
    pub fn coin(base: &[Coin], quote: &[Coin]) -> Self {
        Self::Coin {
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }

    pub fn marker<S: Into<String>>(
        address: Addr,
        denom: S,
        quote_per_share: &[Coin],
        removed_permissions: &[AccessGrant],
    ) -> Self {
        Self::Marker {
            address,
            denom: denom.into(),
            quote_per_share: quote_per_share.to_owned(),
            removed_permissions: removed_permissions.to_owned(),
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
                existing_ask.owner.as_str(),
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
