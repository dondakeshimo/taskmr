use anyhow::Result;
use std::rc::Rc;

use crate::domain::task::ITaskRepository;

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
pub struct ListTaskUseCase {
    task_repository: Rc<dyn ITaskRepository>,
}

impl ListTaskUseCase {
    pub fn new(task_repository: Rc<dyn ITaskRepository>) -> Self {
        ListTaskUseCase { task_repository }
    }

    /// execute addition a task.
    pub fn execute(&self, _: ListTaskUseCaseInput) -> Result<Vec<TaskDTO>> {
        let tasks = self.task_repository.find_opening()?;

        let mut dto_tasks: Vec<TaskDTO> = Vec::new();
        for t in tasks {
            dto_tasks.push(TaskDTO {
                id: t.id().get(),
                title: t.title().to_owned(),
                priority: t.priority().get(),
                cost: t.cost().get(),
            })
        }

        Ok(dto_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Cost, Priority, Task, ID};
    use crate::infra::sqlite::task_repository::TaskRepository;
    use rusqlite::Connection;
    use std::time::Duration;

    fn make_task(seed: u64, is_closed: bool) -> Task {
        Task::from_repository(
            ID::new(seed as i64),
            seed.to_string(),
            is_closed,
            Priority::new(seed as i32),
            Cost::new(seed as i32),
            Duration::from_secs(seed),
        )
    }

    fn make_task_dto(seed: u64) -> TaskDTO {
        TaskDTO {
            id: seed as i64,
            title: seed.to_string(),
            priority: seed as i32,
            cost: seed as i32,
        }
    }

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            input: ListTaskUseCaseInput,
        }

        #[derive(Debug)]
        struct TestCase {
            given: Vec<Task>,
            args: Args,
            want: Vec<TaskDTO>,
            name: String,
        }

        let table = [TestCase {
            name: String::from("nominal: with priority and cost"),
            given: vec![
                make_task(1, false),
                make_task(2, false),
                make_task(3, true),
                make_task(4, false),
            ],
            args: Args {
                input: ListTaskUseCaseInput {},
            },
            want: vec![make_task_dto(1), make_task_dto(2), make_task_dto(4)],
        }];

        for test_case in table {
            let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
            task_repository.create_table_if_not_exists().unwrap();

            for gt in test_case.given {
                task_repository.add(gt).unwrap();
            }

            let list_task_usecase = ListTaskUseCase::new(Rc::new(task_repository));
            let got = list_task_usecase.execute(test_case.args.input).unwrap();

            assert_eq!(got, test_case.want, "Failed in the \"{}\".", test_case.name,);
        }
    }
}
