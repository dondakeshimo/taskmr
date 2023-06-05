use anyhow::Result;

use crate::domain::es_task::{IESTaskRepository, IESTaskRepositoryComponent};

use super::error::UseCaseError;

/// DTO for input of AddTaskUseCase.
#[derive(Debug)]
pub struct ListTaskUseCaseInput {}

/// DTO of task
#[derive(Debug, PartialEq, Eq)]
pub struct TaskDTO {
    pub id: i64,
    pub title: String,
    pub priority: i32,
    pub cost: i32,
}

/// Usecase to list tasks.
pub trait ListTaskUseCase: IESTaskRepositoryComponent {
    /// execute listing tasks.
    /// TODO: CQRS accelerates performance.
    fn execute(&self, _: ListTaskUseCaseInput) -> Result<Vec<TaskDTO>> {
        let sequential_ids = self.repository().load_all_sequential_ids()?;

        let mut tasks = Vec::new();
        for sequential_id in sequential_ids {
            let task = self
                .repository()
                .load_by_sequential_id(sequential_id)?
                .ok_or(UseCaseError::NotFound(sequential_id.to_i64()))?;

            if task.is_closed() {
                continue;
            }

            tasks.push(task);
        }

        let mut dto_tasks: Vec<TaskDTO> = Vec::new();
        for task in tasks {
            dto_tasks.push(TaskDTO {
                id: task.sequential_id().to_i64(),
                title: task.title().to_owned(),
                priority: task.priority().to_i32(),
                cost: task.cost().to_i32(),
            })
        }

        Ok(dto_tasks)
    }
}

impl<T: IESTaskRepositoryComponent> ListTaskUseCase for T {}

/// CloseTaskUseCaseComponent returns CloseTaskUseCase.
pub trait ListTaskUseCaseComponent {
    type ListTaskUseCase: ListTaskUseCase;
    fn list_task_usecase(&self) -> &Self::ListTaskUseCase;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sqlite::es_task_repository::TaskRepository;
    use crate::usecase::es_add_task_usecase::{
        AddTaskUseCase, AddTaskUseCaseComponent, AddTaskUseCaseInput,
    };
    use crate::usecase::es_close_task_usecase::{
        CloseTaskUseCase, CloseTaskUseCaseComponent, CloseTaskUseCaseInput,
    };
    use rusqlite::Connection;

    fn make_task_dto(seed: u64) -> TaskDTO {
        TaskDTO {
            id: seed as i64,
            title: seed.to_string(),
            priority: 10,
            cost: 10,
        }
    }

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            input: ListTaskUseCaseInput,
        }

        #[derive(Debug)]
        struct TaskSource {
            seed: u64,
            is_closed: bool,
        }

        #[derive(Debug)]
        struct TestCase {
            given: Vec<TaskSource>,
            args: Args,
            want: Vec<TaskDTO>,
            name: String,
        }

        struct ListTaskUseCaseComponentImpl {
            task_repository: TaskRepository,
        }

        impl IESTaskRepositoryComponent for ListTaskUseCaseComponentImpl {
            type Repository = TaskRepository;
            fn repository(&self) -> &Self::Repository {
                &self.task_repository
            }
        }

        impl ListTaskUseCaseComponent for ListTaskUseCaseComponentImpl {
            type ListTaskUseCase = Self;
            fn list_task_usecase(&self) -> &Self::ListTaskUseCase {
                self
            }
        }

        // for creating a new task
        impl AddTaskUseCaseComponent for ListTaskUseCaseComponentImpl {
            type AddTaskUseCase = Self;
            fn add_task_usecase(&self) -> &Self::AddTaskUseCase {
                self
            }
        }

        // for creating a new task
        impl CloseTaskUseCaseComponent for ListTaskUseCaseComponentImpl {
            type CloseTaskUseCase = Self;
            fn close_task_usecase(&self) -> &Self::CloseTaskUseCase {
                self
            }
        }

        let table = [
            TestCase {
                name: String::from("normal: with priority and cost"),
                given: vec![
                    TaskSource {
                        seed: 1,
                        is_closed: false,
                    },
                    TaskSource {
                        seed: 2,
                        is_closed: false,
                    },
                    TaskSource {
                        seed: 3,
                        is_closed: true,
                    },
                    TaskSource {
                        seed: 4,
                        is_closed: false,
                    },
                ],
                args: Args {
                    input: ListTaskUseCaseInput {},
                },
                want: vec![make_task_dto(1), make_task_dto(2), make_task_dto(4)],
            },
            TestCase {
                name: String::from("normal: empty"),
                given: vec![
                    TaskSource {
                        seed: 1,
                        is_closed: true,
                    },
                    TaskSource {
                        seed: 2,
                        is_closed: true,
                    },
                ],
                args: Args {
                    input: ListTaskUseCaseInput {},
                },
                want: vec![],
            },
            TestCase {
                name: String::from("normal: empty2"),
                given: vec![],
                args: Args {
                    input: ListTaskUseCaseInput {},
                },
                want: vec![],
            },
        ];

        for test_case in table {
            let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
            task_repository.create_table_if_not_exists().unwrap();
            let list_task_usecase_component_impl = ListTaskUseCaseComponentImpl { task_repository };

            for gt in test_case.given {
                let add_task_usecase = list_task_usecase_component_impl.add_task_usecase();
                let sequential_id = <ListTaskUseCaseComponentImpl as AddTaskUseCase>::execute(
                    &add_task_usecase,
                    AddTaskUseCaseInput {
                        title: gt.seed.to_string(),
                        priority: None,
                        cost: None,
                    },
                )
                .unwrap();

                if gt.is_closed {
                    let close_task_usecase = list_task_usecase_component_impl.close_task_usecase();
                    <ListTaskUseCaseComponentImpl as CloseTaskUseCase>::execute(
                        &close_task_usecase,
                        CloseTaskUseCaseInput { sequential_id },
                    )
                    .unwrap();
                }
            }

            let list_task_usecase = list_task_usecase_component_impl.list_task_usecase();
            let got = <ListTaskUseCaseComponentImpl as ListTaskUseCase>::execute(
                &list_task_usecase,
                test_case.args.input,
            )
            .unwrap();

            assert_eq!(got, test_case.want, "Failed in the \"{}\".", test_case.name,);
        }
    }
}
