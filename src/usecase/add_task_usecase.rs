use anyhow::Result;

use crate::domain::task::{Cost, ITaskRepository, Priority, Task, ID};

/// DTO for input of AddTaskUseCase.
#[derive(Debug)]
pub struct AddTaskUseCaseInput {
    title: String,
    priority: Option<i32>,
    cost: Option<i32>,
}

/// Usecase to add a task.
pub struct AddTaskUseCase {
    task_repository: Box<dyn ITaskRepository>,
}

impl AddTaskUseCase {
    pub fn new(task_repository: Box<dyn ITaskRepository>) -> Self {
        AddTaskUseCase { task_repository }
    }

    /// execute addition a task.
    pub fn execute(&self, input: AddTaskUseCaseInput) -> Result<ID> {
        let p: Option<Priority> = input.priority.map(Priority::new);
        let c: Option<Cost> = input.cost.map(Cost::new);
        let t = Task::new(input.title, p, c);
        self.task_repository.add(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sqlite::task_repository::TaskRepository;
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

        let table = [
            TestCase {
                name: String::from("nominal: with priority and cost"),
                args: Args {
                    input: AddTaskUseCaseInput {
                        title: String::from("title1"),
                        priority: Some(100),
                        cost: Some(200),
                    },
                },
                want: Task::new(
                    "title1".to_owned(),
                    Some(Priority::new(100)),
                    Some(Cost::new(200)),
                ),
            },
            TestCase {
                name: String::from("nominal: without priority and cost"),
                args: Args {
                    input: AddTaskUseCaseInput {
                        title: String::from("title2"),
                        priority: None,
                        cost: None,
                    },
                },
                want: Task::new(
                    "title2".to_owned(),
                    Some(Priority::new(10)),
                    Some(Cost::new(10)),
                ),
            },
        ];

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        let add_task_usecase = AddTaskUseCase::new(Box::new(task_repository));

        for test_case in table {
            let id = add_task_usecase.execute(test_case.args.input).unwrap();
            let got = add_task_usecase
                .task_repository
                .find_by_id(id)
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
