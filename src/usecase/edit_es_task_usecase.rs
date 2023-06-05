use anyhow::Result;

use crate::ddd::component::{AggregateRoot, Repository};
use crate::domain::es_task::{
    Cost, IESTaskRepository, IESTaskRepositoryComponent, Priority, SequentialID, TaskCommand,
};
use crate::usecase::error::UseCaseError;

/// DTO for input of EditTaskUseCase.
#[derive(Debug)]
pub struct EditTaskUseCaseInput {
    pub sequential_id: SequentialID,
    pub title: Option<String>,
    pub priority: Option<i32>,
    pub cost: Option<i32>,
}

/// Usecase to edit a task.
pub trait EditTaskUseCase: IESTaskRepositoryComponent {
    /// execute editing a task.
    fn execute(&self, input: EditTaskUseCaseInput) -> Result<SequentialID> {
        let mut task = self
            .repository()
            .load_by_sequential_id(input.sequential_id)?
            .ok_or(UseCaseError::NotFound(input.sequential_id.to_i64()))?;

        if task.is_closed() {
            return Err(
                UseCaseError::AlreadyClosed(task.sequential_id().to_i64().to_owned()).into(),
            );
        }

        if let Some(title) = input.title {
            task.execute(TaskCommand::EditTitle { title })?;
        }

        if let Some(priority) = input.priority {
            task.execute(TaskCommand::RescorePriority {
                priority: Priority::new(priority),
            })?;
        }

        if let Some(cost) = input.cost {
            task.execute(TaskCommand::RescoreCost {
                cost: Cost::new(cost),
            })?;
        }

        self.repository().save(&mut task)?;
        Ok(task.sequential_id())
    }
}

impl<T: IESTaskRepositoryComponent> EditTaskUseCase for T {}

/// EditTaskUseCaseComponent returns EditTaskUseCase.
pub trait EditTaskUseCaseComponent {
    type EditTaskUseCase: EditTaskUseCase;
    fn edit_task_usecase(&self) -> &Self::EditTaskUseCase;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ddd::component::AggregateID;
    use crate::domain::es_task::{Task, TaskSource};
    use crate::infra::sqlite::es_task_repository::TaskRepository;
    use crate::usecase::add_es_task_usecase::{
        AddTaskUseCase, AddTaskUseCaseComponent, AddTaskUseCaseInput,
    };
    use crate::usecase::close_es_task_usecase::{
        CloseTaskUseCase, CloseTaskUseCaseComponent, CloseTaskUseCaseInput,
    };
    use rusqlite::Connection;

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            input: EditTaskUseCaseInput,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            want: Option<Task>,
            want_error: Option<UseCaseError>,
            name: String,
        }

        struct EditTaskUseCaseComponentImpl {
            task_repository: TaskRepository,
        }

        impl IESTaskRepositoryComponent for EditTaskUseCaseComponentImpl {
            type Repository = TaskRepository;
            fn repository(&self) -> &Self::Repository {
                &self.task_repository
            }
        }

        impl EditTaskUseCaseComponent for EditTaskUseCaseComponentImpl {
            type EditTaskUseCase = Self;
            fn edit_task_usecase(&self) -> &Self::EditTaskUseCase {
                self
            }
        }

        // for creating a new task
        impl AddTaskUseCaseComponent for EditTaskUseCaseComponentImpl {
            type AddTaskUseCase = Self;
            fn add_task_usecase(&self) -> &Self::AddTaskUseCase {
                self
            }
        }

        // for creating a new task
        impl CloseTaskUseCaseComponent for EditTaskUseCaseComponentImpl {
            type CloseTaskUseCase = Self;
            fn close_task_usecase(&self) -> &Self::CloseTaskUseCase {
                self
            }
        }

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        let edit_task_usecase_component_impl = EditTaskUseCaseComponentImpl { task_repository };

        let add_task_usecase = edit_task_usecase_component_impl.add_task_usecase();

        <EditTaskUseCaseComponentImpl as AddTaskUseCase>::execute(
            &add_task_usecase,
            AddTaskUseCaseInput {
                title: "title".to_owned(),
                priority: None,
                cost: None,
            },
        )
        .unwrap();

        <EditTaskUseCaseComponentImpl as AddTaskUseCase>::execute(
            &add_task_usecase,
            AddTaskUseCaseInput {
                title: "closed".to_owned(),
                priority: None,
                cost: None,
            },
        )
        .unwrap();

        let close_task_usecase = edit_task_usecase_component_impl.close_task_usecase();

        <EditTaskUseCaseComponentImpl as CloseTaskUseCase>::execute(
            &close_task_usecase,
            CloseTaskUseCaseInput {
                sequential_id: SequentialID::new(2),
            },
        )
        .unwrap();

        let table = [
            TestCase {
                name: String::from("normal: with title, priority and cost"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        sequential_id: SequentialID::new(1),
                        title: Some(String::from("title1")),
                        priority: Some(100),
                        cost: Some(200),
                    },
                },
                want: Some(Task::create(TaskSource {
                    aggregate_id: AggregateID::new(),
                    sequential_id: SequentialID::new(1),
                    title: "title1".to_owned(),
                    priority: Some(Priority::new(100)),
                    cost: Some(Cost::new(200)),
                })),
                want_error: None,
            },
            TestCase {
                name: String::from("normal: without title, priority and cost"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        sequential_id: SequentialID::new(1),
                        title: None,
                        priority: None,
                        cost: None,
                    },
                },
                want: Some(Task::create(TaskSource {
                    aggregate_id: AggregateID::new(),
                    sequential_id: SequentialID::new(1),
                    title: "title1".to_owned(),
                    priority: Some(Priority::new(100)),
                    cost: Some(Cost::new(200)),
                })),
                want_error: None,
            },
            TestCase {
                name: String::from("abnormal: not found"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        sequential_id: SequentialID::new(3),
                        title: None,
                        priority: None,
                        cost: None,
                    },
                },
                want: None,
                want_error: Some(UseCaseError::NotFound(3)),
            },
            TestCase {
                name: String::from("abnormal: already closed"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        sequential_id: SequentialID::new(2),
                        title: None,
                        priority: None,
                        cost: None,
                    },
                },
                want: None,
                want_error: Some(UseCaseError::AlreadyClosed(2)),
            },
        ];

        for test_case in table {
            let edit_task_usecase = edit_task_usecase_component_impl.edit_task_usecase();
            match <EditTaskUseCaseComponentImpl as EditTaskUseCase>::execute(
                &edit_task_usecase,
                test_case.args.input,
            ) {
                Ok(id) => {
                    let want = test_case.want.unwrap();

                    let got = edit_task_usecase_component_impl
                        .repository()
                        .load_by_sequential_id(id)
                        .unwrap()
                        .unwrap();

                    assert_eq!(
                        got.title(),
                        want.title(),
                        "failed in the \"{}\".",
                        test_case.name,
                    );

                    assert_eq!(
                        got.priority(),
                        want.priority(),
                        "Failed in the \"{}\".",
                        test_case.name,
                    );

                    assert_eq!(
                        got.cost(),
                        want.cost(),
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
