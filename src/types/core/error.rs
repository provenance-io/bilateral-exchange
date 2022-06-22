use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Cannot send funds when canceling order")]
    CancelWithFunds,

    #[error("Cannot create [{id_type}] with id [{id}]. One with that id already exists")]
    ExistingId { id_type: String, id: String },

    #[error("Validation failed with messages: {messages:?}")]
    ValidationError { messages: Vec<String> },

    #[error("Invalid funds provided: {message}")]
    InvalidFundsProvided { message: String },

    #[error("Invalid marker: {message}")]
    InvalidMarker { message: String },

    #[error("Invalid migration: {message}")]
    InvalidMigration { message: String },

    #[error("Scope at address [{scope_address}] has invalid owner: {explanation}")]
    InvalidScopeOwner {
        scope_address: String,
        explanation: String,
    },

    #[error("Invalid type encountered: {explanation}")]
    InvalidType { explanation: String },

    #[error("Missing field: {field:?}")]
    MissingField { field: String },

    #[error("{0}")]
    SemVerError(#[from] semver::Error),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Contact storage error occurred: {message}")]
    StorageError { message: String },

    #[error("Unauthorized")]
    Unauthorized,
}
impl ContractError {
    pub fn cancel_with_funds() -> ContractError {
        ContractError::CancelWithFunds
    }

    pub fn existing_id<S1: Into<String>, S2: Into<String>>(id_type: S1, id: S2) -> ContractError {
        ContractError::ExistingId {
            id_type: id_type.into(),
            id: id.into(),
        }
    }

    pub fn validation_error(messages: &[String]) -> ContractError {
        ContractError::ValidationError {
            messages: messages.to_owned(),
        }
    }

    pub fn invalid_funds_provided<S: Into<String>>(message: S) -> ContractError {
        ContractError::InvalidFundsProvided {
            message: message.into(),
        }
    }

    pub fn invalid_marker<S: Into<String>>(message: S) -> ContractError {
        ContractError::InvalidMarker {
            message: message.into(),
        }
    }

    pub fn invalid_migration<S: Into<String>>(message: S) -> ContractError {
        ContractError::InvalidMigration {
            message: message.into(),
        }
    }

    pub fn invalid_scope_owner<S1: Into<String>, S2: Into<String>>(
        scope_address: S1,
        explanation: S2,
    ) -> ContractError {
        ContractError::InvalidScopeOwner {
            scope_address: scope_address.into(),
            explanation: explanation.into(),
        }
    }

    pub fn invalid_type<S: Into<String>>(explanation: S) -> ContractError {
        ContractError::InvalidType {
            explanation: explanation.into(),
        }
    }

    pub fn missing_field<S: Into<String>>(field: S) -> ContractError {
        ContractError::MissingField {
            field: field.into(),
        }
    }

    pub fn storage_error<S: Into<String>>(message: S) -> ContractError {
        ContractError::StorageError {
            message: message.into(),
        }
    }

    pub fn unauthorized() -> ContractError {
        ContractError::Unauthorized
    }
}
