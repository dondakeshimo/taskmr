use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;

use crate::domain::task::{Cost, ITaskRepository, Priority, Task, ID};

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

    fn find_opening(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,
                    title,
                    is_closed,
                    priority,
                    cost,
                    elapsed_time_sec,
                    created_at,
                    updated_at
             FROM tasks where is_closed = 0",
        )?;

        let task_iter = stmt.query_map([], |row| {
            Ok(Task::from_repository(
                ID::new(row.get(0)?),
                row.get(1)?,
                row.get(2)?,
                Priority::new(row.get(3)?),
                Cost::new(row.get(4)?),
                Duration::from_secs(row.get(5)?),
            ))
        })?;

        let mut tv = Vec::new();
        for t in task_iter {
            tv.push(t?);
        }

        Ok(tv)
    }

    fn fetch_all(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,
                    title,
                    is_closed,
                    priority,
                    cost,
                    elapsed_time_sec,
                    created_at,
                    updated_at
             FROM tasks",
        )?;

        let task_iter = stmt.query_map([], |row| {
            Ok(Task::from_repository(
                ID::new(row.get(0)?),
                row.get(1)?,
                row.get(2)?,
                Priority::new(row.get(3)?),
                Cost::new(row.get(4)?),
                Duration::from_secs(row.get(5)?),
            ))
        })?;

        let mut tv = Vec::new();
        for t in task_iter {
            tv.push(t?);
        }

        Ok(tv)
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

    /// Update a Task.
    fn update(&self, a_task: Task) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "UPDATE tasks SET
                title = ?1,
                is_closed = ?2,
                priority = ?3,
                cost = ?4,
                elapsed_time_sec = ?5
             where id = ?6",
        )?;

        stmt.insert(rusqlite::params![
            a_task.title(),
            a_task.is_closed(),
            a_task.priority().get(),
            a_task.cost().get(),
            a_task.elapsed_time().as_secs(),
            a_task.id().get(),
        ])?;

        Ok(())
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
    fn test_update() {
        #[derive(Debug)]
        struct Args {
            task: Task,
        }

        #[derive(Debug)]
        struct TestCase {
            given: Task,
            args: Args,
            want: Option<Task>,
            name: String,
        }

        let table = [TestCase {
            name: String::from("nominal: close"),
            given: Task::new(
                "hoge".to_owned(),
                Some(Priority::new(2)),
                Some(Cost::new(3)),
            ),
            args: Args {
                task: Task::from_repository(
                    ID::new(1),
                    String::from("fuga"),
                    true,
                    Priority::new(3),
                    Cost::new(4),
                    Duration::from_secs(1),
                ),
            },
            want: Some(Task::from_repository(
                ID::new(1),
                String::from("fuga"),
                true,
                Priority::new(3),
                Cost::new(4),
                Duration::from_secs(1),
            )),
        }];

        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        for test_case in table {
            let id = task_repository.add(test_case.given).unwrap();
            task_repository.update(test_case.args.task).unwrap();
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

    #[test]
    fn test_find_opening() {
        #[derive(Debug)]
        struct TestCase {
            given: Vec<Task>,
            want: Vec<Task>,
            name: String,
        }

        let table = [
            TestCase {
                name: String::from("nominal"),
                given: vec![
                    make_task(1, false),
                    make_task(2, false),
                    make_task(3, true),
                    make_task(4, false),
                ],
                want: vec![
                    make_task(1, false),
                    make_task(2, false),
                    make_task(4, false),
                ],
            },
            TestCase {
                name: String::from("nominal: empty table"),
                given: Vec::new(),
                want: Vec::new(),
            },
            TestCase {
                name: String::from("nominal: all closed"),
                given: vec![
                    make_task(1, true),
                    make_task(2, true),
                    make_task(3, true),
                    make_task(4, true),
                ],
                want: Vec::new(),
            },
        ];

        for test_case in table {
            let task_repository =
                TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
            task_repository.create_table_if_not_exists().unwrap();

            for gt in test_case.given {
                task_repository.add(gt).unwrap();
            }

            assert_eq!(
                task_repository.find_opening().unwrap(),
                test_case.want,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }

    #[test]
    fn test_fetch_all() {
        #[derive(Debug)]
        struct TestCase {
            given: Vec<Task>,
            want: Vec<Task>,
            name: String,
        }

        let table = [
            TestCase {
                name: String::from("nominal"),
                given: vec![
                    make_task(1, false),
                    make_task(2, false),
                    make_task(3, true),
                    make_task(4, false),
                ],
                want: vec![
                    make_task(1, false),
                    make_task(2, false),
                    make_task(3, true),
                    make_task(4, false),
                ],
            },
            TestCase {
                name: String::from("nominal: empty table"),
                given: Vec::new(),
                want: Vec::new(),
            },
        ];

        for test_case in table {
            let task_repository =
                TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
            task_repository.create_table_if_not_exists().unwrap();

            for gt in test_case.given {
                task_repository.add(gt).unwrap();
            }

            assert_eq!(
                task_repository.fetch_all().unwrap(),
                test_case.want,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
