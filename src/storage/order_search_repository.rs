use crate::storage::order_indices::OrderIndices;
use crate::types::constants::{
    DEFAULT_SEARCH_ORDER, DEFAULT_SEARCH_PAGE_SIZE, MAX_SEARCH_PAGE_SIZE,
};
use crate::types::search::{Search, SearchResult, SearchType};
use cosmwasm_std::Storage;
use cw_storage_plus::{IndexList, IndexedMap, MultiIndex};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct OrderSearchRepository<'a, T, O>
where
    T: IndexList<O> + OrderIndices<'a, O>,
    O: Clone + Serialize + DeserializeOwned,
{
    index_map: IndexedMap<'a, &'a [u8], O, T>,
}
impl<'a, T, O> OrderSearchRepository<'a, T, O>
where
    T: IndexList<O> + OrderIndices<'a, O>,
    O: Clone + Serialize + DeserializeOwned,
{
    pub fn new(index_map: IndexedMap<'a, &'a [u8], O, T>) -> Self {
        Self { index_map }
    }

    pub fn search(&self, storage: &dyn Storage, search: Search) -> SearchResult<O> {
        let page_size = search
            .page_size
            .unwrap_or(DEFAULT_SEARCH_PAGE_SIZE)
            .min(MAX_SEARCH_PAGE_SIZE)
            // Never allow page sizes < 1, which would cause division panics
            .max(1);
        // Never allow lower page numbers than 1, which would cause subtraction panics
        let page_number = search.page_number.unwrap_or(1).max(1);
        let response = match search.search_type {
            SearchType::All => self.get_all_response(storage, page_size, page_number),
            SearchType::ValueType { value_type } => {
                self.get_value_type_response(storage, &value_type, page_size, page_number)
            }
            SearchType::Id { id } => self.get_id_response(storage, &id),
            SearchType::Owner { owner } => {
                self.get_owner_response(storage, &owner, page_size, page_number)
            }
        };
        // Multiply total pages by 100 to determine if there is some lossy remainder during division,
        // effectively allowing a ceiling value to be established on the quotient.  This ensures that
        // a total result count of 101 with a page size will return a page count of 11 instead of 10,
        // etc.
        let mut total_pages = (response.total_results * 100) / page_size;
        total_pages = total_pages / 100 + if total_pages % 100 != 0 { 1 } else { 0 };
        SearchResult {
            results: response.query_results,
            page_number,
            page_size,
            total_pages,
        }
    }

    fn get_all_response(
        &self,
        storage: &dyn Storage,
        page_size: usize,
        page_number: usize,
    ) -> SearchResponse<O> {
        let query = || {
            self.index_map
                .range(storage, None, None, DEFAULT_SEARCH_ORDER)
        };
        SearchResponse {
            total_results: query().count(),
            query_results: query()
                .skip(page_size * (page_number - 1))
                .take(page_size)
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap().1)
                .collect(),
        }
    }

    fn get_value_type_response(
        &self,
        storage: &dyn Storage,
        value_type: &str,
        page_size: usize,
        page_number: usize,
    ) -> SearchResponse<O> {
        let query = || {
            self.index_map
                .idx
                .type_index()
                .prefix(value_type.to_owned())
                .range(storage, None, None, DEFAULT_SEARCH_ORDER)
        };
        SearchResponse {
            total_results: query().count(),
            query_results: query()
                .skip(page_size * (page_number - 1))
                .take(page_size)
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap().1)
                .collect(),
        }
    }

    fn get_id_response(&self, storage: &dyn Storage, id: &str) -> SearchResponse<O> {
        SearchResponse {
            total_results: 1,
            query_results: self
                .index_map
                .load(storage, id.as_bytes())
                .map(|value| vec![value])
                .unwrap_or(vec![]),
        }
    }

    fn get_owner_response(
        &self,
        storage: &dyn Storage,
        owner: &str,
        page_size: usize,
        page_number: usize,
    ) -> SearchResponse<O> {
        let query = || {
            self.index_map
                .idx
                .owner_index()
                .prefix(owner.to_owned())
                .range(storage, None, None, DEFAULT_SEARCH_ORDER)
        };
        SearchResponse {
            total_results: query().count(),
            query_results: query()
                .skip(page_size * (page_number - 1))
                .take(page_size)
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap().1)
                .collect(),
        }
    }
}

struct SearchResponse<T> {
    pub total_results: usize,
    pub query_results: Vec<T>,
}
