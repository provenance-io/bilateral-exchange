use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Ask Order does not match Bid Order")]
    AskBidMismatch {},

    #[error("Cannot send funds when canceling order")]
    CancelWithFunds {},

    #[error("Cannot send funds when executing match")]
    ExecuteWithFunds {},

    #[error("Ask base was not sent")]
    MissingAskBase,

    #[error("Scope ask base cannot also be sent funds")]
    ScopeAskBaseWithFunds,

    #[error("Coin ask base can only be created from funds provided")]
    CoinAskBaseWithoutFunds,

    #[error("Missing field: {field:?}")]
    MissingField { field: String },

    #[error("Bid quote was not sent")]
    MissingBidQuote,

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}
