/// Allows any Sized type to functionally move itself into a Result<T, U>
pub trait ResultExtensions
where
    Self: Sized,
{
    /// Converts the caller into an Ok (left-hand-side) result
    fn to_ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }

    /// Converts the caller into an Err (right-hand-side) result
    fn to_err<T>(self) -> Result<T, Self> {
        Err(self)
    }
}
// Implement for EVERYTHING IN THE UNIVERSE
impl<T> ResultExtensions for T {}

#[cfg(test)]
mod tests {
    use crate::types::core::error::ContractError;

    use super::ResultExtensions;

    #[test]
    fn test_to_ok() {
        let value: Result<String, ContractError> = "hello".to_string().to_ok();
        assert_eq!(
            "hello".to_string(),
            value.unwrap(),
            "expected the value to serialize correctly",
        );
    }

    #[test]
    fn test_to_err() {
        let result: Result<(), ContractError> = ContractError::Unauthorized.to_err();
        let error = result.unwrap_err();
        assert!(
            matches!(error, ContractError::Unauthorized),
            "the error should unwrap correctly, but got incorrect error: {:?}",
            error,
        );
    }
}
