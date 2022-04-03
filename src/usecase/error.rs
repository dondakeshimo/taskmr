use thiserror::Error;

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
            "the task for id `3` has already be closed".to_owned()
        );
    }
}
