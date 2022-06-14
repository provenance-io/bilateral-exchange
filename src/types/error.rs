use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Ask Order does not match Bid Order")]
    AskBidMismatch,

    #[error("Cannot send funds when canceling order")]
    CancelWithFunds,

    #[error("Cannot send funds when executing match")]
    ExecuteWithFunds,

    #[error("Cannot create ask with id: {id}. An ask with that id already exists")]
    ExistingAskId { id: String },

    #[error("Field issues encountered: {messages:?}")]
    InvalidFields { messages: Vec<String> },

    #[error("Invalid funds provided: {message}")]
    InvalidFundsProvided { message: String },

    #[error("Invalid marker: {message}")]
    InvalidMarker { message: String },

    #[error("Missing field: {field:?}")]
    MissingField { field: String },

    #[error("Bid quote was not sent")]
    MissingBidQuote,

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Contact storage error occurred: {message}")]
    StorageError { message: String },

    #[error("Unauthorized")]
    Unauthorized,
}
