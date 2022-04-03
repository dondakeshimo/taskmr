use anyhow::Result;
use std::rc::Rc;

use crate::domain::task::{ITaskRepository, ID};
use crate::usecase::error::UseCaseError;

/// DTO for input of CloseTaskUseCase.
#[derive(Debug)]
pub struct CloseTaskUseCaseInput {
    pub id: i64,
}

/// Usecase to close a task.
pub struct CloseTaskUseCase {
    task_repository: Rc<dyn ITaskRepository>,
}

impl CloseTaskUseCase {
    pub fn new(task_repository: Rc<dyn ITaskRepository>) -> Self {
        CloseTaskUseCase { task_repository }
    }

    /// execute closing a task.
    pub fn execute(&self, input: CloseTaskUseCaseInput) -> Result<ID> {
        let mut t = self
            .task_repository
            .find_by_id(ID::new(input.id))?
            .ok_or(UseCaseError::NotFound(input.id))?;
        let id = t.id();

        if t.is_closed() {
            Err(UseCaseError::AlreadyClosed(id.get().to_owned()))?;
        }

        t.close();
        self.task_repository.update(t)?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;
    use crate::infra::sqlite::task_repository::TaskRepository;
    use rusqlite::Connection;

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            input: CloseTaskUseCaseInput,
        }

        #[derive(Debug)]
        struct Want {
            title: String,
            is_closed: bool,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            want: Option<Want>,
            want_error: Option<UseCaseError>,
            name: String,
        }

        let given = Task::new("title".to_owned(), None, None);

        let table = [
            TestCase {
                name: String::from("normal: close a task"),
                args: Args {
                    input: CloseTaskUseCaseInput { id: 1 },
                },
                want: Some(Want {
                    title: "title".to_owned(),
                    is_closed: true,
                }),
                want_error: None,
            },
            TestCase {
                name: String::from("abnormal: already closed"),
                args: Args {
                    input: CloseTaskUseCaseInput { id: 1 },
                },
                want: None,
                want_error: Some(UseCaseError::AlreadyClosed(1)),
            },
            TestCase {
                name: String::from("abnormal: not found"),
                args: Args {
                    input: CloseTaskUseCaseInput { id: 2 },
                },
                want: None,
                want_error: Some(UseCaseError::NotFound(2)),
            },
        ];

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        task_repository.add(given).unwrap();
        let close_task_usecase = CloseTaskUseCase::new(Rc::new(task_repository));

        for test_case in table {
            match close_task_usecase.execute(test_case.args.input) {
                Ok(id) => {
                    let want = test_case.want.unwrap();

                    let got = close_task_usecase
                        .task_repository
                        .find_by_id(id)
                        .unwrap()
                        .unwrap();

                    assert_eq!(
                        got.title(),
                        want.title,
                        "failed in the \"{}\".",
                        test_case.name,
                    );

                    assert_eq!(
                        got.is_closed(),
                        want.is_closed,
                        "Failed in the \"{}\".",
                        test_case.name,
                    );
                }
                Err(err) => {
                    assert_eq!(
                        err.to_string(),
                        test_case.want_error.unwrap().to_string(),
                        "Failed in the \"{}\".",
                        test_case.name,
                    );
                }
            };
        }
    }
}
