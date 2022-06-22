use crate::storage::bid_order_storage::bid_orders;
use crate::storage::order_search_repository::OrderSearchRepository;
use crate::types::core::error::ContractError;
use crate::types::request::search::Search;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

pub fn search_bids(deps: Deps<ProvenanceQuery>, search: Search) -> Result<Binary, ContractError> {
    let repository = OrderSearchRepository::new(bid_orders());
    to_binary(&repository.search(deps.storage, search))?.to_ok()
}
