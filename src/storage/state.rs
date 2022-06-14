use cosmwasm_std::{Addr, Coin, Storage, Timestamp};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static NAMESPACE_ORDER_ASK: &[u8] = b"ask";
pub static NAMESPACE_ORDER_BID: &[u8] = b"bid";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOrder {
    pub id: String,
    pub ask_type: String,
    pub owner: Addr,
    pub 
    
    pub base: Vec<Coin>,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidOrder {
    pub base: Vec<Coin>,
    pub effective_time: Option<Timestamp>,
    pub id: String,
    pub owner: Addr,
    pub quote: Vec<Coin>,
}

pub fn get_ask_storage(storage: &mut dyn Storage) -> Bucket<AskOrder> {
    bucket(storage, NAMESPACE_ORDER_ASK)
}

pub fn get_ask_storage_read(storage: &dyn Storage) -> ReadonlyBucket<AskOrder> {
    bucket_read(storage, NAMESPACE_ORDER_ASK)
}
pub fn get_bid_storage(storage: &mut dyn Storage) -> Bucket<BidOrder> {
    bucket(storage, NAMESPACE_ORDER_BID)
}

pub fn get_bid_storage_read(storage: &dyn Storage) -> ReadonlyBucket<BidOrder> {
    bucket_read(storage, NAMESPACE_ORDER_BID)
}
