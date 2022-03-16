use rusqlite;

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
                name TEXT NOT NULL
            )",
            [],
        )
    }

    /// Find Task by id.
    pub fn find_by_id(&self, id: i32) -> rusqlite::Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM tasks where id = ?")?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => return Ok(Some(row.get(1)?)),
            None => return Ok(None),
        }
    }

    /// Add Task
    pub fn add(&self, name: &str) -> rusqlite::Result<usize> {
        self.conn.execute(
            "INSERT INTO tasks (name) VALUES (?1)",
            rusqlite::params![name],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        args: i32,
        expected: rusqlite::Result<Option<String>>,
        name: String,
    }

    #[test]
    fn find_by_id() {
        let table = [
            TestCase {
                name: String::from("nominal"),
                args: 1,
                expected: Ok(Some(String::from("hoge"))),
            },
            TestCase {
                name: String::from("anominal: not found task"),
                args: 2,
                expected: Ok(None),
            },
        ];

        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table().unwrap();
        task_repository.add("hoge").unwrap();

        for test_case in table {
            assert_eq!(
                task_repository.find_by_id(test_case.args),
                test_case.expected,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
