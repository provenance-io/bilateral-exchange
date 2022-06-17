use crate::storage::order_indices::OrderIndices;
use crate::types::constants::{
    DEFAULT_SEARCH_ORDER, DEFAULT_SEARCH_PAGE_NUMBER, DEFAULT_SEARCH_PAGE_SIZE,
    MAX_SEARCH_PAGE_SIZE, MIN_SEARCH_PAGE_NUMBER, MIN_SEARCH_PAGE_SIZE,
};
use crate::types::search::{Search, SearchResult, SearchType, Searchable};
use cosmwasm_std::{Storage, Uint128};
use cw_storage_plus::{IndexList, IndexedMap};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct OrderSearchRepository<'a, T, O>
where
    T: IndexList<O> + OrderIndices<'a, O>,
    O: Clone + Serialize + DeserializeOwned + Searchable,
{
    index_map: IndexedMap<'a, &'a [u8], O, T>,
}
impl<'a, T, O> OrderSearchRepository<'a, T, O>
where
    T: IndexList<O> + OrderIndices<'a, O>,
    O: Clone + Serialize + DeserializeOwned + Searchable,
{
    pub fn new(index_map: IndexedMap<'a, &'a [u8], O, T>) -> Self {
        Self { index_map }
    }

    pub fn search(&self, storage: &dyn Storage, search: Search) -> SearchResult<O> {
        let page_size = self.get_page_size(&search);
        let page_number = self.get_page_number(&search);
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
        // etc.  This also maxes with 1 to ensure that searches that return no results still show a
        // single page for an easier UI integration.
        let mut total_pages = (response.total_results * 100) / page_size;
        total_pages = (total_pages / 100 + if total_pages % 100 != 0 { 1 } else { 0 }).max(1);
        SearchResult {
            results: response.query_results,
            page_number: Uint128::new(page_number as u128),
            page_size: Uint128::new(page_size as u128),
            total_pages: Uint128::new(total_pages as u128),
        }
    }

    fn get_page_size(&self, search: &Search) -> usize {
        search
            .page_size
            .map(|u| u.u128() as usize)
            .unwrap_or(DEFAULT_SEARCH_PAGE_SIZE)
            // Limit page size to ensure overloads do not occur
            .min(MAX_SEARCH_PAGE_SIZE)
            // Never allow page sizes < 1, which would cause division panics
            .max(MIN_SEARCH_PAGE_SIZE)
    }

    fn get_page_number(&self, search: &Search) -> usize {
        search
            .page_number
            .map(|u| u.u128() as usize)
            .unwrap_or(DEFAULT_SEARCH_PAGE_NUMBER)
            // Never allow page sizes < 1, which would cause division panics
            .max(MIN_SEARCH_PAGE_NUMBER)
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

#[cfg(test)]
mod tests {
    use crate::storage::ask_order_storage::ask_orders;
    use crate::storage::bid_order_storage::bid_orders;
    use crate::storage::order_search_repository::OrderSearchRepository;
    use crate::types::constants::{
        DEFAULT_SEARCH_PAGE_NUMBER, DEFAULT_SEARCH_PAGE_SIZE, MAX_SEARCH_PAGE_SIZE,
        MIN_SEARCH_PAGE_NUMBER, MIN_SEARCH_PAGE_SIZE,
    };
    use crate::types::search::Search;
    use cosmwasm_std::Uint128;

    #[test]
    fn test_get_page_size() {
        let repository = OrderSearchRepository::new(ask_orders());
        let mut search = Search::all(None, None);
        assert_eq!(
            DEFAULT_SEARCH_PAGE_SIZE,
            repository.get_page_size(&search),
            "expected the default page size to be used when no page size is provided to the search",
        );
        search.page_size = Some(Uint128::new(MAX_SEARCH_PAGE_SIZE as u128 + 1));
        assert_eq!(
            MAX_SEARCH_PAGE_SIZE,
            repository.get_page_size(&search),
            "expected the max page size to always be favored if a value greater than it is used",
        );
        search.page_size = Some(Uint128::new(MIN_SEARCH_PAGE_SIZE as u128 - 1));
        assert_eq!(
            MIN_SEARCH_PAGE_SIZE,
            repository.get_page_size(&search),
            "expected the min page size to always be favored if a value less than it is used",
        );
        search.page_size = Some(Uint128::new(MIN_SEARCH_PAGE_SIZE as u128 + 1));
        assert_eq!(
            MIN_SEARCH_PAGE_SIZE + 1,
            repository.get_page_size(&search),
            "expected a value within the bounds of min and max to always be favored if provided",
        );
    }

    #[test]
    fn test_get_page_number() {
        let repository = OrderSearchRepository::new(bid_orders());
        let mut search = Search::all(None, None);
        assert_eq!(
            DEFAULT_SEARCH_PAGE_NUMBER,
            repository.get_page_number(&search),
            "expected the default page number to be used when no page number is provided to the search",
        );
        search.page_number = Some(Uint128::new(MIN_SEARCH_PAGE_NUMBER as u128 - 1));
        assert_eq!(
            MIN_SEARCH_PAGE_NUMBER,
            repository.get_page_number(&search),
            "expected the min page number to always be favored if a value less than it is used",
        );
        search.page_number = Some(Uint128::new(MIN_SEARCH_PAGE_NUMBER as u128 + 10));
        assert_eq!(
            MIN_SEARCH_PAGE_NUMBER + 10,
            repository.get_page_number(&search),
            "expected a value above the min page number to always be favored if it is provided",
        );
    }
}
