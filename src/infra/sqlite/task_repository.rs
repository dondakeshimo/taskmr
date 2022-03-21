use std::time;

use rusqlite;

use crate::domain::task;

/// Implementation of TaskRepository.
pub struct TaskRepository {
    conn: rusqlite::Connection,
}

impl TaskRepository {
    /// Construct a TaskRepository.
    pub fn new(conn: rusqlite::Connection) -> TaskRepository {
        TaskRepository { conn }
    }

    /// Create table tasks.
    /// This function is to be called at first time.
    pub fn create_table(&self) -> rusqlite::Result<usize> {
        self.conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                is_closed INTEGER DEFAULT 0,
                priority INTEGER NOT NULL DEFAULT 10,
                cost INTEGER NOT NULL DEFAULT 10,
                elapsed_time_sec INTEGER NOT NULL DEFAULT 10,
                created_at TEXT NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime'))
            )",
            [],
        )
    }

    /// Find Task by id.
    pub fn find_by_id(&self, id: i32) -> rusqlite::Result<Option<task::Task>> {
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

        let mut rows = stmt.query([id])?;

        match rows.next()? {
            Some(row) => {
                Ok(Some(task::Task::from_repository(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    time::Duration::from_secs(row.get(5)?),
                )))
            }
            None => Ok(None),
        }
    }

    /// Add Task
    pub fn add(&self, a_task: task::Task) -> rusqlite::Result<usize> {
        self.conn.execute(
            "INSERT INTO tasks (title, is_closed, priority, cost, elapsed_time_sec) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![a_task.title(), a_task.is_closed(), a_task.priority(), a_task.cost(), a_task.elapsed_time().as_secs()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Args {
        task: task::Task,
        id: i32,
    }

    #[derive(Debug)]
    struct TestCase {
        args: Args,
        expected: rusqlite::Result<Option<task::Task>>,
        name: String,
    }

    #[test]
    fn test_add_and_find_by_id() {
        let table = [
            TestCase {
                name: String::from("nominal"),
                args: Args {
                    task: task::Task::from_repository(
                        1,
                        String::from("hoge"),
                        true,
                        2,
                        3,
                        time::Duration::from_secs(4),
                    ),
                    id: 1,
                },
                expected: Ok(Some(task::Task::from_repository(
                    1,
                    String::from("hoge"),
                    true,
                    2,
                    3,
                    time::Duration::from_secs(4),
                ))),
            },
            TestCase {
                name: String::from("anominal: not found task"),
                args: Args {
                    task: task::Task::from_repository(
                        2,
                        String::from("hoge"),
                        true,
                        2,
                        3,
                        time::Duration::from_secs(4),
                    ),
                    id: 3,
                },
                expected: Ok(None),
            },
        ];

        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table().unwrap();

        for test_case in table {
            task_repository.add(test_case.args.task).unwrap();
            assert_eq!(
                task_repository.find_by_id(test_case.args.id),
                test_case.expected,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
