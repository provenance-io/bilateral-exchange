use crate::storage::ask_order_storage::{
    delete_ask_order_by_id, get_ask_order_by_id, update_ask_order,
};
use crate::storage::bid_order_storage::{delete_bid_order_by_id, get_bid_order_by_id};
use crate::storage::contract_info::get_contract_info;
use crate::types::core::error::ContractError;
use crate::types::request::ask_types::ask_collateral::{
    AskCollateral, CoinTradeAskCollateral, MarkerShareSaleAskCollateral, MarkerTradeAskCollateral,
    ScopeTradeAskCollateral,
};
use crate::types::request::ask_types::ask_order::AskOrder;
use crate::types::request::bid_types::bid_collateral::{
    CoinTradeBidCollateral, MarkerShareSaleBidCollateral, MarkerTradeBidCollateral,
    ScopeTradeBidCollateral,
};
use crate::types::request::bid_types::bid_order::BidOrder;
use crate::types::request::share_sale_type::ShareSaleType;
use crate::util::extensions::ResultExtensions;
use crate::util::provenance_utilities::{release_marker_from_contract, replace_scope_owner};
use crate::validation::execute_match_validation::validate_match;
use cosmwasm_std::{BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};
use provwasm_std::{
    withdraw_coins, write_scope, ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery,
};

// match and execute an ask and bid order
pub fn execute_match(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    ask_id: String,
    bid_id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // only the admin may execute matches
    if info.sender != get_contract_info(deps.storage)?.admin {
        return Err(ContractError::Unauthorized);
    }
    // return error if funds sent
    if !info.funds.is_empty() {
        return ContractError::InvalidFundsProvided {
            message: "funds should not be provided during match execution".to_string(),
        }
        .to_err();
    }
    let mut invalid_fields: Vec<String> = vec![];
    if ask_id.is_empty() {
        invalid_fields.push("ask id must not be empty".to_string());
    }
    if bid_id.is_empty() {
        invalid_fields.push("bid id must not be empty".to_string());
    }
    // return error if either ids are badly formed
    if !invalid_fields.is_empty() {
        return ContractError::ValidationError {
            messages: invalid_fields,
        }
        .to_err();
    }

    let ask_order = get_ask_order_by_id(deps.storage, ask_id)?;
    let bid_order = get_bid_order_by_id(deps.storage, bid_id)?;

    validate_match(&deps, &ask_order, &bid_order)?;

    let execute_result = match &ask_order.collateral {
        AskCollateral::CoinTrade(collateral) => execute_coin_trade(
            deps,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_coin_trade()?,
        )?,
        AskCollateral::MarkerTrade(collateral) => execute_marker_trade(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_marker_trade()?,
        )?,
        AskCollateral::MarkerShareSale(collateral) => execute_marker_share_sale(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_marker_share_sale()?,
        )?,
        AskCollateral::ScopeTrade(collateral) => execute_scope_trade(
            deps,
            &env,
            &ask_order,
            &bid_order,
            collateral,
            bid_order.collateral.get_scope_trade()?,
        )?,
    };

    Response::new()
        .add_messages(execute_result.messages)
        .add_attribute("action", "execute")
        .add_attribute("ask_id", &ask_order.id)
        .add_attribute("bid_id", &bid_order.id)
        .to_ok()
}

struct ExecuteResults {
    pub messages: Vec<CosmosMsg<ProvenanceMsg>>,
}
impl ExecuteResults {
    fn new(messages: Vec<CosmosMsg<ProvenanceMsg>>) -> Self {
        Self { messages }
    }
}

fn execute_coin_trade(
    deps: DepsMut<ProvenanceQuery>,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &CoinTradeAskCollateral,
    bid_collateral: &CoinTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    // Remove ask and bid - this transaction has concluded
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: ask_order.owner.to_string(),
            amount: ask_collateral.quote.to_owned(),
        }),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: bid_order.owner.to_string(),
            amount: bid_collateral.base.to_owned(),
        }),
    ])
    .to_ok()
}

fn execute_marker_trade(
    deps: DepsMut<ProvenanceQuery>,
    env: &Env,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &MarkerTradeAskCollateral,
    bid_collateral: &MarkerTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    let mut messages = vec![];
    if let Some(asker_permissions) = ask_collateral
        .removed_permissions
        .iter()
        .find(|perm| perm.address == ask_order.owner)
    {
        // Now that the match has been made, grant all permissions on the marker to the bidder that
        // the asker once had.  The validation code has already ensured that the asker was an admin
        // of the marker, so the bidder at very least has the permission on the marker to grant
        // themselves any remaining permissions they desire.
        let mut bidder_permissions = asker_permissions.to_owned();
        bidder_permissions.address = bid_order.owner.to_owned();
        messages.append(&mut release_marker_from_contract(
            &ask_collateral.denom,
            &env.contract.address,
            &[bidder_permissions],
        )?);
    } else {
        return ContractError::validation_error(&[
            "failed to find access permissions in the revoked permissions for the asker"
                .to_string(),
        ])
        .to_err();
    }
    // Send the entirety of the quote to the asker. They have just effectively sold their
    // marker to the bidder.
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: ask_order.owner.to_string(),
        amount: bid_collateral.quote.to_owned(),
    }));
    // Remove ask and bid - this transaction has concluded
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(messages).to_ok()
}

fn execute_marker_share_sale(
    deps: DepsMut<ProvenanceQuery>,
    env: &Env,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &MarkerShareSaleAskCollateral,
    bid_collateral: &MarkerShareSaleBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    // Asker gets the quote that the bidder provided from escrow
    // Bidder gets their X marker coins withdrawn to them from the contract-controlled marker
    let mut messages = vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: ask_order.owner.to_string(),
            amount: bid_collateral.quote.to_owned(),
        }),
        withdraw_coins(
            &ask_collateral.denom,
            bid_collateral.share_count.u128(),
            &ask_collateral.denom,
            bid_order.owner.to_owned(),
        )?,
    ];
    let mut terminate_sale = || -> Result<(), ContractError> {
        // Marker gets released to the asker.  The sale is effectively over.
        messages.append(&mut release_marker_from_contract(
            &ask_collateral.denom,
            &env.contract.address,
            &ask_collateral.removed_permissions,
        )?);
        delete_ask_order_by_id(deps.storage, &ask_order.id)?;
        ().to_ok()
    };
    match ask_collateral.sale_type {
        ShareSaleType::SingleTransaction { .. } => terminate_sale()?,
        ShareSaleType::MultipleTransactions {
            remove_sale_share_threshold,
        } => {
            let share_threshold = remove_sale_share_threshold.map(|t| t.u128()).unwrap_or(0);
            let shares_remaining_after_sale =
                ask_collateral.remaining_shares.u128() - bid_collateral.share_count.u128();
            // Validation will prevent this code from being executed if shares_remaining_after_sale
            // is ever less than share_threshold, so only an equality check is necessary
            if share_threshold == shares_remaining_after_sale {
                terminate_sale()?;
            } else {
                let mut ask_order = ask_order.to_owned();
                let mut ask_collateral = ask_collateral.to_owned();
                ask_collateral.remaining_shares = Uint128::new(shares_remaining_after_sale);
                ask_order.collateral = AskCollateral::MarkerShareSale(ask_collateral);
                // Replace the ask order in storage with an updated remaining_shares value
                update_ask_order(deps.storage, &ask_order)?;
            }
        }
    }
    // Regardless of sale type scenario, the bid is always deleted after successful execution
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(messages).to_ok()
}

fn execute_scope_trade(
    deps: DepsMut<ProvenanceQuery>,
    env: &Env,
    ask_order: &AskOrder,
    bid_order: &BidOrder,
    ask_collateral: &ScopeTradeAskCollateral,
    bid_collateral: &ScopeTradeBidCollateral,
) -> Result<ExecuteResults, ContractError> {
    // Asker gets the quote that the bidder provided from escrow
    let mut messages = vec![CosmosMsg::Bank(BankMsg::Send {
        to_address: ask_order.owner.to_string(),
        amount: ask_collateral.quote.to_owned(),
    })];
    let scope = ProvenanceQuerier::new(&deps.querier).get_scope(&bid_collateral.scope_address)?;
    // Bidder gets the scope transferred to them
    messages.push(write_scope(
        replace_scope_owner(scope, bid_order.owner.to_owned()),
        vec![env.contract.address.to_owned()],
    )?);
    // Remove the ask and bid orders now that the trade has been finalized
    delete_ask_order_by_id(deps.storage, &ask_order.id)?;
    delete_bid_order_by_id(deps.storage, &bid_order.id)?;
    ExecuteResults::new(messages).to_ok()
}

#[cfg(test)]
mod tests {}
