use anyhow::Result;

use crate::ddd::component::{AggregateID, Repository};
use crate::domain::es_task::{
    Cost, IESTaskRepository, IESTaskRepositoryComponent, Priority, SequentialID, Task, TaskSource,
};

use super::es_repository::{TransactionableRepository, TransactionableRepositoryComponent};

/// DTO for input of AddTaskUseCase.
#[derive(Debug)]
pub struct AddTaskUseCaseInput {
    pub title: String,
    pub priority: Option<i32>,
    pub cost: Option<i32>,
}

/// Usecase to add a task.
pub trait AddTaskUseCase<'a>: IESTaskRepositoryComponent + TransactionableRepositoryComponent<'a, Task> {
    /// execute addition a task.
    fn execute(&'a mut self, input: AddTaskUseCaseInput) -> Result<SequentialID> {
        let p: Option<Priority> = input.priority.map(Priority::new);
        let c: Option<Cost> = input.cost.map(Cost::new);

        self.transactionable_repository().begin();

        let aggregate_id = AggregateID::new();
        let sequential_id = self.repository().issue_sequential_id(aggregate_id)?;

        let mut t = Task::create(TaskSource {
            aggregate_id,
            sequential_id,
            title: input.title,
            priority: p,
            cost: c,
        });

        self.repository().save(&mut t)?;

        self.transactionable_repository().commit();

        Ok(t.sequential_id())
    }
}

impl<'a, T: IESTaskRepositoryComponent + TransactionableRepositoryComponent<'a, Task>> AddTaskUseCase<'a> for T {}

/// AddTaskUseCaseComponent returns AddTaskUseCase.
pub trait AddTaskUseCaseComponent<'a> {
    type AddTaskUseCase: AddTaskUseCase<'a>;
    fn add_task_usecase(&self) -> &Self::AddTaskUseCase;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sqlite::es_task_repository::TaskRepository;
    use rusqlite::Connection;

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            input: AddTaskUseCaseInput,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            want: Task,
            name: String,
        }

        struct AddTaskUseCaseComponentImpl<'a> {
            task_repository: TaskRepository<'a>,
        }

        impl<'a> IESTaskRepositoryComponent for AddTaskUseCaseComponentImpl<'a> {
            type Repository = TaskRepository<'a>;
            fn repository(&self) -> &Self::Repository {
                &self.task_repository
            }
        }

        impl<'a> TransactionableRepositoryComponent<'a, Task> for AddTaskUseCaseComponentImpl<'a> {
            type TransactionableRepository = TaskRepository<'a>;
            fn transactionable_repository(&mut self) -> &mut Self::TransactionableRepository {
                &mut self.task_repository
            }
        }

        impl<'a> AddTaskUseCaseComponent<'a> for AddTaskUseCaseComponentImpl<'a> {
            type AddTaskUseCase = Self;
            fn add_task_usecase(&self) -> &Self::AddTaskUseCase {
                self
            }
        }

        let table = [
            TestCase {
                name: String::from("normal: with priority and cost"),
                args: Args {
                    input: AddTaskUseCaseInput {
                        title: String::from("title1"),
                        priority: Some(100),
                        cost: Some(200),
                    },
                },
                want: Task::create(TaskSource {
                    aggregate_id: AggregateID::new(),
                    sequential_id: SequentialID::new(10),
                    title: "title1".to_owned(),
                    priority: Some(Priority::new(100)),
                    cost: Some(Cost::new(200)),
                }),
            },
            TestCase {
                name: String::from("normal: without priority and cost"),
                args: Args {
                    input: AddTaskUseCaseInput {
                        title: String::from("title2"),
                        priority: None,
                        cost: None,
                    },
                },
                want: Task::create(TaskSource {
                    aggregate_id: AggregateID::new(),
                    sequential_id: SequentialID::new(10),
                    title: "title2".to_owned(),
                    priority: Some(Priority::new(10)),
                    cost: Some(Cost::new(10)),
                }),
            },
        ];

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        let add_task_usecase_component_impl = AddTaskUseCaseComponentImpl { task_repository };

        for test_case in table {
            let id = add_task_usecase_component_impl
                .add_task_usecase()
                .execute(test_case.args.input)
                .unwrap();
            let got = add_task_usecase_component_impl
                .task_repository
                .load_by_sequential_id(id)
                .unwrap()
                .unwrap();

            assert_eq!(
                got.title(),
                test_case.want.title(),
                "Failed in the \"{}\".",
                test_case.name,
            );

            assert_eq!(
                got.priority(),
                test_case.want.priority(),
                "Failed in the \"{}\".",
                test_case.name,
            );

            assert_eq!(
                got.cost(),
                test_case.want.cost(),
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
