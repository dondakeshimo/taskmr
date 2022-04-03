use anyhow::Result;
use std::rc::Rc;

use crate::domain::task::{Cost, ITaskRepository, Priority, ID};
use crate::usecase::error::UseCaseError;

/// DTO for input of EditTaskUseCase.
#[derive(Debug)]
pub struct EditTaskUseCaseInput {
    pub id: i64,
    pub title: Option<String>,
    pub priority: Option<i32>,
    pub cost: Option<i32>,
}

/// Usecase to edit a task.
pub struct EditTaskUseCase {
    task_repository: Rc<dyn ITaskRepository>,
}

impl EditTaskUseCase {
    pub fn new(task_repository: Rc<dyn ITaskRepository>) -> Self {
        EditTaskUseCase { task_repository }
    }

    /// execute editing a task.
    pub fn execute(&self, input: EditTaskUseCaseInput) -> Result<ID> {
        let mut t = self
            .task_repository
            .find_by_id(ID::new(input.id))?
            .ok_or(UseCaseError::NotFound(input.id))?;
        let id = t.id();

        if t.is_closed() {
            return Err(UseCaseError::AlreadyClosed(id.get().to_owned()).into());
        }

        if let Some(title) = input.title {
            t.edit_title(title);
        }

        if let Some(priority) = input.priority {
            t.rescore_priority(Priority::new(priority));
        }

        if let Some(cost) = input.cost {
            t.rescore_cost(Cost::new(cost));
        }

        self.task_repository.update(t)?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::domain::task::Task;
    use crate::infra::sqlite::task_repository::TaskRepository;
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

        let given = vec![
            Task::new("title".to_owned(), None, None),
            Task::from_repository(
                ID::new(2),
                "closed".to_owned(),
                true,
                Priority::new(10),
                Cost::new(10),
                Duration::from_secs(0),
            ),
        ];

        let table = [
            TestCase {
                name: String::from("normal: with title, priority and cost"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        id: 1,
                        title: Some(String::from("title1")),
                        priority: Some(100),
                        cost: Some(200),
                    },
                },
                want: Some(Task::new(
                    "title1".to_owned(),
                    Some(Priority::new(100)),
                    Some(Cost::new(200)),
                )),
                want_error: None,
            },
            TestCase {
                name: String::from("normal: without title, priority and cost"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        id: 1,
                        title: None,
                        priority: None,
                        cost: None,
                    },
                },
                want: Some(Task::new(
                    "title1".to_owned(),
                    Some(Priority::new(100)),
                    Some(Cost::new(200)),
                )),
                want_error: None,
            },
            TestCase {
                name: String::from("abnormal: not found"),
                args: Args {
                    input: EditTaskUseCaseInput {
                        id: 3,
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
                        id: 2,
                        title: None,
                        priority: None,
                        cost: None,
                    },
                },
                want: None,
                want_error: Some(UseCaseError::AlreadyClosed(2)),
            },
        ];

        let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        given.into_iter().for_each(|g| {
            task_repository.add(g).unwrap();
        });
        let edit_task_usecase = EditTaskUseCase::new(Rc::new(task_repository));

        for test_case in table {
            match edit_task_usecase.execute(test_case.args.input) {
                Ok(id) => {
                    let want = test_case.want.unwrap();

                    let got = edit_task_usecase
                        .task_repository
                        .find_by_id(id)
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
