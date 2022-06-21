use crate::test::type_helpers::decimal;
use cosmwasm_std::{coins, Addr, Coin, Decimal};
use provwasm_std::{AccessGrant, Marker, MarkerAccess, MarkerStatus, MarkerType};

pub const DEFAULT_MARKER_ADDRESS: &str = "marker_address";
pub const DEFAULT_MARKER_HOLDINGS: u128 = 100;
pub const DEFAULT_MARKER_DENOM: &str = "markerdenom";

pub struct MockMarker {
    pub address: Addr,
    pub coins: Vec<Coin>,
    pub account_number: u64,
    pub sequence: u64,
    pub manager: String,
    pub permissions: Vec<AccessGrant>,
    pub status: MarkerStatus,
    pub denom: String,
    pub total_supply: Decimal,
    pub marker_type: MarkerType,
    pub supply_fixed: bool,
}
impl Default for MockMarker {
    fn default() -> Self {
        Self {
            address: Addr::unchecked(DEFAULT_MARKER_ADDRESS),
            coins: coins(DEFAULT_MARKER_HOLDINGS, DEFAULT_MARKER_DENOM),
            account_number: 50,
            sequence: 0,
            manager: "".to_string(),
            permissions: vec![AccessGrant {
                address: Addr::unchecked("cosmos2contract"),
                permissions: vec![
                    MarkerAccess::Admin,
                    MarkerAccess::Burn,
                    MarkerAccess::Delete,
                    MarkerAccess::Deposit,
                    MarkerAccess::Mint,
                    MarkerAccess::Withdraw,
                ],
            }],
            status: MarkerStatus::Active,
            denom: DEFAULT_MARKER_DENOM.to_string(),
            total_supply: decimal(DEFAULT_MARKER_HOLDINGS),
            marker_type: MarkerType::Coin,
            supply_fixed: true,
        }
    }
}
impl MockMarker {
    pub fn new_marker() -> Marker {
        Self::default().to_marker()
    }

    pub fn new_owned_marker<S: Into<String>>(owner_address: S) -> Marker {
        Self {
            permissions: vec![
                AccessGrant {
                    address: Addr::unchecked(owner_address),
                    permissions: vec![
                        MarkerAccess::Admin,
                        MarkerAccess::Burn,
                        MarkerAccess::Delete,
                        MarkerAccess::Deposit,
                        MarkerAccess::Mint,
                        MarkerAccess::Withdraw,
                    ],
                },
                AccessGrant {
                    address: Addr::unchecked("cosmos2contract"),
                    permissions: vec![MarkerAccess::Admin, MarkerAccess::Withdraw],
                },
            ],
            ..Self::default()
        }
        .to_marker()
    }

    pub fn to_marker(self) -> Marker {
        Marker {
            address: self.address,
            coins: self.coins,
            account_number: self.account_number,
            sequence: self.sequence,
            manager: self.manager,
            permissions: self.permissions,
            status: self.status,
            denom: self.denom,
            total_supply: self.total_supply,
            marker_type: self.marker_type,
            supply_fixed: self.supply_fixed,
        }
    }
}
