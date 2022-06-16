use crate::storage::ask_order_storage::ask_orders;
use crate::storage::order_search_repository::OrderSearchRepository;
use crate::types::error::ContractError;
use crate::types::search::Search;
use crate::util::extensions::ResultExtensions;
use cosmwasm_std::{to_binary, Binary, Deps};
use provwasm_std::ProvenanceQuery;

pub fn search_asks(deps: Deps<ProvenanceQuery>, search: Search) -> Result<Binary, ContractError> {
    let repository = OrderSearchRepository::new(ask_orders());
    to_binary(&repository.search(deps.storage, search))?.to_ok()
}

#[cfg(test)]
mod tests {
    use crate::query::search_asks::search_asks;
    use crate::storage::ask_order_storage::insert_ask_order;
    use crate::types::ask_collateral::AskCollateral;
    use crate::types::ask_order::AskOrder;
    use crate::types::request_descriptor::RequestDescriptor;
    use crate::types::search::{Search, SearchResult};
    use cosmwasm_std::{from_binary, Addr, Deps, Storage, Timestamp};
    use provwasm_mocks::mock_dependencies;
    use provwasm_std::ProvenanceQuery;

    #[test]
    fn test_search_all() {
        let mut deps = mock_dependencies(&[]);
        for index in 0..21 {
            insert_ask_order(
                deps.as_mut().storage,
                &AskOrder::new_unchecked(
                    format!("ask_id_{}", index),
                    Addr::unchecked(format!("asker{}", index)),
                    // Swap between coin and marker for some variety
                    if index % 2 == 0 {
                        AskCollateral::coin(&[], &[])
                    } else {
                        AskCollateral::marker(
                            Addr::unchecked(format!("marker{}", index)),
                            format!("denom{}", index),
                            index as u128,
                            &[],
                            &[],
                        )
                    },
                    Some(RequestDescriptor {
                        description: Some(format!("Some ask {}", index)),
                        effective_time: Some(Timestamp::default()),
                    }),
                ),
            )
            .expect(&format!(
                "expected ask order {} to be inserted correctly",
                index
            ));
        }
        // Search for everything with a page size of 10 and no offset key
        let first_search = Search::all(Some(10), None);
        let first_page = search(deps.as_ref(), first_search);
        assert_eq!(
            10,
            first_page.results.len(),
            "there are 21 items in storage, so all results should be returned in the 0-10 range",
        );
        assert_eq!(
            1, first_page.page_number,
            "with no page number input, the fist page should be returned"
        );
        assert_eq!(
            10, first_page.page_size,
            "the provided page size should always be returned",
        );
        assert_eq!(
            3, first_page.total_pages,
            "the total page value should be equal to the total number of items divided by results",
        );
        let second_search = Search::all(Some(10), Some(2));
        let second_page = search(deps.as_ref(), second_search);
        assert_eq!(
            10,
            second_page.results.len(),
            "there are 21 items in storage, so all results should be returned in the 10-20 range",
        );
        assert_eq!(
            2, second_page.page_number,
            "the requested page number should be returned in the result",
        );
        assert_eq!(
            3, second_page.total_pages,
            "the total page number should not change unless new values are added",
        );
        let second_page_ids = second_page
            .results
            .iter()
            .map(|ask| &ask.id)
            .collect::<Vec<&String>>();
        assert!(
            !first_page
                .results
                .iter()
                .any(|ask| second_page_ids.contains(&&ask.id)),
            "no ids from the first search should be contained in the second search's results"
        );
        let third_page = search(deps.as_ref(), Search::all(Some(10), Some(3)));
        assert_eq!(
            1,
            third_page.results.len(),
            "the final, 21st result should be returned on the third page",
        );
        assert_eq!(
            3, third_page.page_number,
            "the requested page number should be returned in the result",
        );
        assert_eq!(
            3, third_page.total_pages,
            "the total page number should be unchanging",
        );
        let final_id = third_page.results.first().unwrap().id.to_owned();
        assert!(
            !first_page.results.iter().any(|ask| ask.id == final_id),
            "the third page's resulting id should not be included in the first page's results",
        );
        assert!(
            !second_page_ids.iter().any(|ask_id| ask_id == &&final_id),
            "the third page's resulting id should not be included in the second page's results",
        );
        let fourth_page = search(deps.as_ref(), Search::all(Some(10), Some(4)));
        assert!(
            fourth_page.results.is_empty(),
            "no results should be contained because there are not enough items for a fourth page",
        );
        assert_eq!(
            4, fourth_page.page_number,
            "the requested page number should be returned in the result",
        );
        assert_eq!(
            3, fourth_page.total_pages,
            "the result should indicate that there are only three pages, revealing how ridiculous the search was in the first place",
        );
        let max_page = search(deps.as_ref(), Search::all(Some(25), None));
        assert_eq!(
            21,
            max_page.results.len(),
            "all items should be returned in a search that requests more items than are available",
        );
        assert_eq!(
            25, max_page.page_size,
            "the provided page size should be returned",
        );
        assert_eq!(
            1, max_page.page_number,
            "the default first page number should be used",
        );
        assert_eq!(
            1,
            max_page.total_pages,
            "due to there being less results than will fit on a single page, there should be one page",
        );
    }

    fn search(deps: Deps<ProvenanceQuery>, search: Search) -> SearchResult<AskOrder> {
        let bin = search_asks(deps, search).expect("expected the result to succeed");
        from_binary(&bin)
            .expect("expected binary deserialization to the appropriate type to succeed")
    }
}
