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

    #[error("Cannot create [{id_type}] with id [{id}]. One with that id already exists")]
    ExistingId { id_type: String, id: String },

    #[error("Validation failed with messages: {messages:?}")]
    ValidationError { messages: Vec<String> },

    #[error("Invalid funds provided: {message}")]
    InvalidFundsProvided { message: String },

    #[error("Invalid marker: {message}")]
    InvalidMarker { message: String },

    #[error("Scope at address [{scope_address}] has invalid owner: {explanation}")]
    InvalidScopeOwner {
        scope_address: String,
        explanation: String,
    },

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
