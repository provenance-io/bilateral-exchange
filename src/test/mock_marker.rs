use crate::test::type_helpers::decimal;
use cosmwasm_std::{coins, Addr, Coin, Decimal};
use provwasm_std::{AccessGrant, Marker, MarkerAccess, MarkerStatus, MarkerType};

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
            address: Addr::unchecked("marker_address"),
            coins: coins(100, "defaultdenom"),
            account_number: 50,
            sequence: 0,
            manager: "".to_string(),
            permissions: vec![AccessGrant {
                address: Addr::unchecked("marker_admin"),
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
            denom: "defaultdenom".to_string(),
            total_supply: decimal(100),
            marker_type: MarkerType::Coin,
            supply_fixed: true,
        }
    }
}
impl MockMarker {
    pub fn new_marker() -> Marker {
        Self::default().to_marker()
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
