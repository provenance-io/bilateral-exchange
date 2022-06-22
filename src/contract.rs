use crate::execute::cancel_ask::cancel_ask;
use crate::execute::cancel_bid::cancel_bid;
use crate::execute::create_ask::create_ask;
use crate::execute::create_bid::create_bid;
use crate::execute::execute_match::execute_match;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::migrate::migrate_contract::migrate_contract;
use crate::query::get_ask::query_ask;
use crate::query::get_bid::query_bid;
use crate::query::get_contract_info::query_contract_info;
use crate::query::search_asks::search_asks;
use crate::query::search_bids::search_bids;
use crate::types::core::error::ContractError;
use crate::types::core::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

// smart contract initialization entrypoint
#[entry_point]
pub fn instantiate(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    instantiate_contract(deps, env, info, msg)
}

// smart contract execute entrypoint
#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateAsk { ask, descriptor } => create_ask(deps, env, info, ask, descriptor),
        ExecuteMsg::CreateBid { bid, descriptor } => create_bid(deps, info, bid, descriptor),
        ExecuteMsg::CancelAsk { id } => cancel_ask(deps, env, info, id),
        ExecuteMsg::CancelBid { id } => cancel_bid(deps, env, info, id),
        ExecuteMsg::ExecuteMatch { ask_id, bid_id } => {
            execute_match(deps, env, info, ask_id, bid_id)
        }
    }
}

// smart contract query entrypoint
#[entry_point]
pub fn query(
    deps: Deps<ProvenanceQuery>,
    _env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetAsk { id } => query_ask(deps, id),
        QueryMsg::GetBid { id } => query_bid(deps, id),
        QueryMsg::GetContractInfo {} => query_contract_info(deps),
        QueryMsg::SearchAsks { search } => search_asks(deps, search),
        QueryMsg::SearchBids { search } => search_bids(deps, search),
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    match msg {
        MigrateMsg::ContractUpgrade { options } => migrate_contract(deps, options),
    }
}
