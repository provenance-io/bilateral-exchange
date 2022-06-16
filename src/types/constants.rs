use cosmwasm_std::Order;

pub const ASK_TYPE_COIN: &str = "coin";
pub const ASK_TYPE_MARKER: &str = "marker";

pub const BID_TYPE_COIN: &str = "coin";
pub const BID_TYPE_MARKER: &str = "marker";

// For matching against ask/bid type strings directly when hitting an unknown value
pub const UNKNOWN_TYPE: &str = "unknown_type";

pub const DEFAULT_SEARCH_PAGE_SIZE: usize = 10;
pub const MAX_SEARCH_PAGE_SIZE: usize = 25;
pub const DEFAULT_SEARCH_ORDER: Order = Order::Ascending;
