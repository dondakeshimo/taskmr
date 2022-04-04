//! # UseCase Error
//!
//! This module define an error to use in or outer Application Service layer.

use thiserror::Error;

/// Error is used in or outer Application Service layer.
#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("the task for id `{0}` is not found")]
    NotFound(i64),
    #[error("the task for id `{0}` has already been closed")]
    AlreadyClosed(i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found() {
        assert_eq!(
            UseCaseError::NotFound(2).to_string(),
            "the task for id `2` is not found".to_owned()
        );
    }

    #[test]
    fn test_already_closed() {
        assert_eq!(
            UseCaseError::AlreadyClosed(3).to_string(),
            "the task for id `3` has already been closed".to_owned()
        );
    }
}
