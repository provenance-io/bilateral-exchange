use cosmwasm_std::Order;

pub const ASK_TYPE_COIN_TRADE: &str = "coin_trade";
pub const ASK_TYPE_MARKER_TRADE: &str = "marker_trade";
pub const ASK_TYPE_MARKER_SHARE_SALE: &str = "marker_share_sale";
pub const ASK_TYPE_SCOPE_TRADE: &str = "scope_trade";

pub const BID_TYPE_COIN_TRADE: &str = "coin_trade";
pub const BID_TYPE_MARKER_TRADE: &str = "marker_trade";
pub const BID_TYPE_MARKER_SHARE_SALE: &str = "marker_share_sale";
pub const BID_TYPE_SCOPE_TRADE: &str = "scope_trade";

// For matching against ask/bid type strings directly when hitting an unknown value
pub const UNKNOWN_TYPE: &str = "unknown_type";

pub const DEFAULT_SEARCH_PAGE_SIZE: usize = 10;
pub const MAX_SEARCH_PAGE_SIZE: usize = 25;
pub const MIN_SEARCH_PAGE_SIZE: usize = 1;
pub const DEFAULT_SEARCH_PAGE_NUMBER: usize = 1;
pub const MIN_SEARCH_PAGE_NUMBER: usize = 1;
pub const DEFAULT_SEARCH_ORDER: Order = Order::Ascending;
