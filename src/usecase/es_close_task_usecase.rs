use anyhow::Result;

use crate::ddd::component::{AggregateRoot, Repository};
use crate::domain::es_task::{
    IESTaskRepository, IESTaskRepositoryComponent, SequentialID, TaskCommand,
};
use crate::usecase::error::UseCaseError;

/// DTO for input of CloseTaskUseCase.
#[derive(Debug)]
pub struct CloseTaskUseCaseInput {
    pub sequential_id: SequentialID,
}

/// Usecase to close a task.
pub trait CloseTaskUseCase: IESTaskRepositoryComponent {
    /// execute closing a task.
    fn execute(&self, input: CloseTaskUseCaseInput) -> Result<SequentialID> {
        let mut task = self
            .repository()
            .load_by_sequential_id(input.sequential_id)?
            .ok_or(UseCaseError::NotFound(input.sequential_id.to_i64()))?;

        if task.is_closed() {
            return Err(UseCaseError::AlreadyClosed(task.sequential_id().to_i64()).into());
        }

        task.execute(TaskCommand::Close)?;

        self.repository().save(&mut task)?;
        Ok(task.sequential_id())
    }
}

impl<T: IESTaskRepositoryComponent> CloseTaskUseCase for T {}

/// CloseTaskUseCaseComponent returns CloseTaskUseCase.
pub trait CloseTaskUseCaseComponent {
    type CloseTaskUseCase: CloseTaskUseCase;
    fn close_task_usecase(&self) -> &Self::CloseTaskUseCase;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sqlite::es_task_repository::TaskRepository;
    use crate::usecase::es_add_task_usecase::{
        AddTaskUseCase, AddTaskUseCaseComponent, AddTaskUseCaseInput,
    };
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

        struct CloseTaskUseCaseComponentImpl {
            task_repository: TaskRepository,
        }

        impl IESTaskRepositoryComponent for CloseTaskUseCaseComponentImpl {
            type Repository = TaskRepository;
            fn repository(&self) -> &Self::Repository {
                &self.task_repository
            }
        }

        impl CloseTaskUseCaseComponent for CloseTaskUseCaseComponentImpl {
            type CloseTaskUseCase = Self;
            fn close_task_usecase(&self) -> &Self::CloseTaskUseCase {
                self
            }
        }

        // for creating a new task
        impl AddTaskUseCaseComponent for CloseTaskUseCaseComponentImpl {
            type AddTaskUseCase = Self;
            fn add_task_usecase(&self) -> &Self::AddTaskUseCase {
                self
            }
        }

        let table = [
            TestCase {
                name: String::from("normal: close a task"),
                args: Args {
                    input: CloseTaskUseCaseInput {
                        sequential_id: SequentialID::new(1),
                    },
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
                    input: CloseTaskUseCaseInput {
                        sequential_id: SequentialID::new(1),
                    },
                },
                want: None,
                want_error: Some(UseCaseError::AlreadyClosed(1)),
            },
            TestCase {
                name: String::from("abnormal: not found"),
                args: Args {
                    input: CloseTaskUseCaseInput {
                        sequential_id: SequentialID::new(2),
                    },
                },
                want: None,
                want_error: Some(UseCaseError::NotFound(2)),
            },
        ];

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        let close_task_usecase_component_impl = CloseTaskUseCaseComponentImpl { task_repository };

        let add_task_usecase = close_task_usecase_component_impl.add_task_usecase();

        <CloseTaskUseCaseComponentImpl as AddTaskUseCase>::execute(
            &add_task_usecase,
            AddTaskUseCaseInput {
                title: "title".to_owned(),
                priority: None,
                cost: None,
            },
        )
        .unwrap();

        let close_task_usecase = close_task_usecase_component_impl.close_task_usecase();
        for test_case in table {
            match <CloseTaskUseCaseComponentImpl as CloseTaskUseCase>::execute(
                &close_task_usecase,
                test_case.args.input,
            ) {
                Ok(sequential_id) => {
                    let want = test_case.want.unwrap();

                    let got = close_task_usecase_component_impl
                        .task_repository
                        .load_by_sequential_id(sequential_id)
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
