use crate::types::constants::{
    ASK_TYPE_COIN, ASK_TYPE_MARKER, BID_TYPE_COIN, BID_TYPE_MARKER, UNKNOWN_TYPE,
};
use crate::types::error::ContractError;
use crate::util::extensions::ResultExtensions;
use crate::validation::bid_order_validation::validate_bid_order;
use cosmwasm_std::{Addr, Coin, Storage, Timestamp};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const NAMESPACE_BID_PK: &str = "bid";
const NAMESPACE_OWNER_IDX: &str = "bid__owner";
const NAMESPACE_TYPE_IDX: &str = "bid__type";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BidOrder {
    pub id: String,
    pub bid_type: String,
    pub owner: Addr,
    pub collateral: BidCollateral,
    pub effective_time: Option<Timestamp>,
}
impl BidOrder {
    pub fn new<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: BidCollateral,
        effective_time: Option<Timestamp>,
    ) -> Result<Self, ContractError> {
        let bid_order = Self::new_unchecked(id, owner, collateral, effective_time);
        validate_bid_order(&bid_order)?;
        bid_order.to_ok()
    }

    pub fn new_unchecked<S: Into<String>>(
        id: S,
        owner: Addr,
        collateral: BidCollateral,
        effective_time: Option<Timestamp>,
    ) -> Self {
        Self {
            id: id.into(),
            bid_type: match collateral {
                BidCollateral::Coin { .. } => BID_TYPE_COIN.to_string(),
                BidCollateral::Marker { .. } => BID_TYPE_MARKER.to_string(),
            },
            owner,
            collateral,
            effective_time,
        }
    }

    pub fn get_pk(&self) -> &[u8] {
        self.id.as_bytes()
    }

    pub fn get_matching_ask_type(&self) -> &str {
        match self.bid_type.as_str() {
            BID_TYPE_COIN => ASK_TYPE_COIN,
            BID_TYPE_MARKER => ASK_TYPE_MARKER,
            _ => UNKNOWN_TYPE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BidCollateral {
    Coin {
        base: Vec<Coin>,
        quote: Vec<Coin>,
    },
    Marker {
        address: Addr,
        denom: String,
        quote: Vec<Coin>,
    },
}
impl BidCollateral {
    pub fn coin(base: Vec<Coin>, quote: Vec<Coin>) -> Self {
        Self::Coin { base, quote }
    }

    pub fn marker<S: Into<String>>(address: Addr, denom: S, quote: Vec<Coin>) -> Self {
        Self::Marker {
            address,
            denom: denom.into(),
            quote,
        }
    }
}

pub struct BidOrderIndices<'a> {
    owner_index: MultiIndex<'a, String, BidOrder>,
    type_index: MultiIndex<'a, String, BidOrder>,
}
impl<'a> IndexList<BidOrder> for BidOrderIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<BidOrder>> + '_> {
        let v: Vec<&dyn Index<BidOrder>> = vec![&self.owner_index, &self.type_index];
        Box::new(v.into_iter())
    }
}

pub fn bid_orders<'a>() -> IndexedMap<'a, &'a [u8], BidOrder, BidOrderIndices<'a>> {
    let indices = BidOrderIndices {
        owner_index: MultiIndex::new(
            |bid: &BidOrder| bid.owner.clone().to_string(),
            NAMESPACE_BID_PK,
            NAMESPACE_OWNER_IDX,
        ),
        type_index: MultiIndex::new(
            |bid: &BidOrder| bid.bid_type.clone().to_string(),
            NAMESPACE_BID_PK,
            NAMESPACE_TYPE_IDX,
        ),
    };
    IndexedMap::new(NAMESPACE_BID_PK, indices)
}

pub fn insert_bid_order(
    storage: &mut dyn Storage,
    bid_order: &BidOrder,
) -> Result<(), ContractError> {
    let state = bid_orders();
    if let Ok(existing_bid) = state.load(storage, bid_order.get_pk()) {
        return ContractError::StorageError {
            message: format!(
                "a bid with id [{}] for owner [{}] already exists",
                existing_bid.id,
                existing_bid.owner.as_str(),
            ),
        }
        .to_err();
    }
    state
        .replace(storage, bid_order.get_pk(), Some(bid_order), None)?
        .to_ok()
}

pub fn get_bid_order_by_id<S: Into<String>>(
    storage: &dyn Storage,
    id: S,
) -> Result<BidOrder, ContractError> {
    let id = id.into();
    bid_orders()
        .load(storage, id.as_bytes())
        .map_err(|e| ContractError::StorageError {
            message: format!("failed to find BidOrder by id [{}]: {:?}", id, e),
        })
}

pub fn delete_bid_order_by_id<S: Into<String>>(
    storage: &mut dyn Storage,
    id: S,
) -> Result<(), ContractError> {
    let id = id.into();
    bid_orders()
        .remove(storage, id.as_bytes())
        .map_err(|e| ContractError::StorageError {
            message: format!("failed to remove BidOrder by id [{}]: {:?}", id, e),
        })?;
    ().to_ok()
}
