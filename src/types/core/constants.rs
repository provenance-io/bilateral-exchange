use cosmwasm_std::Order;

pub const DEFAULT_SEARCH_PAGE_SIZE: usize = 10;
pub const MAX_SEARCH_PAGE_SIZE: usize = 25;
pub const MIN_SEARCH_PAGE_SIZE: usize = 1;
pub const DEFAULT_SEARCH_PAGE_NUMBER: usize = 1;
pub const MIN_SEARCH_PAGE_NUMBER: usize = 1;
pub const DEFAULT_SEARCH_ORDER: Order = Order::Ascending;
