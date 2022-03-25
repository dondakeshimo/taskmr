use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;

use crate::domain::task::{Cost, Priority, Task, ID, ITaskRepository};

/// Implementation of TaskRepository.
pub struct TaskRepository {
    conn: rusqlite::Connection,
}

impl TaskRepository {
    /// Construct a TaskRepository.
    pub fn new(conn: Connection) -> TaskRepository {
        TaskRepository { conn }
    }

    /// Create table tasks.
    /// This function is to be called at first time.
    ///
    /// FIXME: This function includes magic number about default values.
    /// These values should sync default values of task::Task::new.
    pub fn create_table_if_not_exists(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE if not exists tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                is_closed INTEGER DEFAULT 0,
                priority INTEGER NOT NULL DEFAULT 10,
                cost INTEGER NOT NULL DEFAULT 10,
                elapsed_time_sec INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime'))
            )",
            [],
        )?;

        Ok(())
    }
}

impl ITaskRepository for TaskRepository {
    /// Find a Task by id.
    fn find_by_id(&self, id: ID) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,
                    title,
                    is_closed,
                    priority,
                    cost,
                    elapsed_time_sec,
                    created_at,
                    updated_at
             FROM tasks where id = ?",
        )?;

        let mut rows = stmt.query([id.get()])?;

        match rows.next()? {
            Some(row) => Ok(Some(Task::from_repository(
                ID::new(row.get(0)?),
                row.get(1)?,
                row.get(2)?,
                Priority::new(row.get(3)?),
                Cost::new(row.get(4)?),
                Duration::from_secs(row.get(5)?),
            ))),
            None => Ok(None),
        }
    }

    /// Add a Task.
     fn add(&self, a_task: Task) -> Result<ID> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO tasks (
                title,
                is_closed,
                priority,
                cost,
                elapsed_time_sec
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        let rowid = stmt.insert(rusqlite::params![
            a_task.title(),
            a_task.is_closed(),
            a_task.priority().get(),
            a_task.cost().get(),
            a_task.elapsed_time().as_secs()
        ])?;

        Ok(ID::new(rowid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_panic_create_table_twice() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        task_repository.create_table_if_not_exists().unwrap();
    }

    #[test]
    fn test_add() {
        #[derive(Debug)]
        struct Args {
            task: Task,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            want: Option<Task>,
            name: String,
        }

        let table = [TestCase {
            name: String::from("nominal"),
            args: Args {
                task: Task::new(
                    String::from("hoge"),
                    Some(Priority::new(2)),
                    Some(Cost::new(3)),
                ),
            },
            want: Some(Task::from_repository(
                ID::new(1),
                String::from("hoge"),
                false,
                Priority::new(2),
                Cost::new(3),
                Duration::from_secs(0),
            )),
        }];

        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        for test_case in table {
            let id = task_repository.add(test_case.args.task).unwrap();
            assert_eq!(
                task_repository.find_by_id(id).unwrap(),
                test_case.want,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }

    #[test]
    fn test_find_by_id() {
        #[derive(Debug)]
        struct Args {
            make_id: fn(id: ID) -> ID,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            make_want: fn(id: ID) -> Option<Task>,
            name: String,
        }

        let table = [
            TestCase {
                name: String::from("nominal"),
                args: Args { make_id: |id| id },
                make_want: |id| {
                    Some(Task::from_repository(
                        id,
                        String::from("fuga"),
                        false,
                        Priority::new(10),
                        Cost::new(10),
                        Duration::from_secs(0),
                    ))
                },
            },
            TestCase {
                name: String::from("anominal: not found task"),
                args: Args {
                    make_id: |id| ID::new(id.get() + 100),
                },
                make_want: |_| None,
            },
        ];

        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        let inserted_id = task_repository
            .add(Task::new(String::from("fuga"), None, None))
            .unwrap();

        for test_case in table {
            assert_eq!(
                task_repository
                    .find_by_id((test_case.args.make_id)(inserted_id))
                    .unwrap(),
                (test_case.make_want)(inserted_id),
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
