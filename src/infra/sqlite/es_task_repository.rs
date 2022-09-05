use anyhow::Result;
use rusqlite::Connection;

use crate::ddd::component::{AggregateID, AggregateRoot, DomainEventEnvelope, Entity, Repository};
use crate::domain::es_task::{IESTaskRepository, SequentialID, Task, TaskDomainEvent};

/// Implementation of TaskRepository.
pub struct TaskRepository {
    conn: rusqlite::Connection,
}

impl TaskRepository {
    /// Construct a TaskRepository.
    pub fn new(conn: Connection) -> TaskRepository {
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        TaskRepository { conn }
    }

    /// Create table tasks.
    /// This function is to be called at first time.
    pub fn create_table_if_not_exists(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE if not exists task_events (
                aggregate_id TEXT NOT NULL,
                aggregate_version INTEGER NOT NULL,
                event TEXT NOT NULL,
                event_version INTEGER NOT NULL,
                occurred_on TEXT NOT NULL,
                PRIMARY KEY(aggregate_id, aggregate_version)
            )",
            [],
        )?;

        // NOTE: phantom_version is needed to define FOREIGN KEY.
        self.conn.execute(
            "CREATE TABLE if not exists task_sequential_ids (
                sequential_id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL UNIQUE,
                phantom_version INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (task_id, phantom_version) REFERENCES task_events(aggregate_id, aggregate_version)
            )",
            [],
        )?;

        Ok(())
    }
}

impl Repository<Task> for TaskRepository {
    /// load a Task by id.
    fn load(&self, id: AggregateID) -> Result<Task> {
        let mut stmt = self.conn.prepare(
            "SELECT aggregate_id,
                    aggregate_version,
                    event,
                    event_version,
                    occurred_on
             FROM task_events where aggregate_id = ?
             ORDER BY aggregate_version ASC",
        )?;

        let event_iter = stmt.query_map([id.to_string()], |row| row.get::<_, String>(2))?;

        let mut events = Vec::new();
        for e in event_iter {
            let event: DomainEventEnvelope<TaskDomainEvent> = serde_json::from_str(&e?)?;
            events.push(event);
        }

        let task = Task::recreate(id, events);

        Ok(task)
    }

    /// save the task events.
    /// The reason why an argument `task` as `mut` is to clear events associated to the task.
    fn save(&self, task: &mut Task) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO task_events (
                aggregate_id,
                aggregate_version,
                event,
                event_version,
                occurred_on
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for te in task.events() {
            stmt.insert(rusqlite::params![
                task.id().to_string(),
                te.aggregate_version(),
                serde_json::to_string(&te)?,
                te.event_version(),
                te.occurred_on().format("%Y-%m-%d %H:%m:%s").to_string(),
            ])?;
        }

        task.clear_events();

        Ok(())
    }
}

impl IESTaskRepository for TaskRepository {
    fn issue_sequential_id(&self, aggregate_id: AggregateID) -> Result<SequentialID> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO task_sequential_ids (
                task_id
             ) VALUES (?1)",
        )?;

        let rowid = stmt.insert(rusqlite::params![aggregate_id.to_string()])?;

        Ok(SequentialID::new(rowid))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ddd::component::Entity,
        domain::es_task::{Cost, Priority, TaskCommand},
    };

    use super::*;

    #[test]
    fn test_not_panic_create_table_twice() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();
        task_repository.create_table_if_not_exists().unwrap();
    }

    #[test]
    fn test_save_and_load() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        let mut task = Task::create(
            "test this task".into(),
            Some(Priority::new(11)),
            Some(Cost::new(12)),
        );
        task.execute(TaskCommand::EditTitle {
            title: "it is awesome task".into(),
        })
        .unwrap();

        task_repository.save(&mut task).unwrap();

        let loaded_task = task_repository.load(task.id()).unwrap();

        assert_eq!(
            task, loaded_task,
            "Failed in the \"{}\".",
            "test_save_and_load",
        );

        let seq_id = task_repository
            .issue_sequential_id(task.aggregate_id())
            .unwrap();
        assert_eq!(seq_id, SequentialID::new(1));
        task.execute(TaskCommand::AssignSequentialID {
            sequential_id: seq_id,
        })
        .unwrap();

        task_repository.save(&mut task).unwrap();

        let loaded_task = task_repository.load(task.id()).unwrap();

        assert_eq!(
            task, loaded_task,
            "Failed in the \"{}\".",
            "test_save_and_load",
        );
    }

    #[test]
    fn test_fail_issue_sequential_id_twice() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        let mut task = Task::create(
            "test this task".into(),
            Some(Priority::new(11)),
            Some(Cost::new(12)),
        );
        task_repository.save(&mut task).unwrap();

        let seq_id = task_repository
            .issue_sequential_id(task.aggregate_id())
            .unwrap();
        assert_eq!(seq_id, SequentialID::new(1));

        task_repository
            .issue_sequential_id(task.aggregate_id())
            .unwrap_err();
    }

    #[test]
    fn test_succeed_issue_sequential_id() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        let mut task1 = Task::create(
            "test this task".into(),
            Some(Priority::new(11)),
            Some(Cost::new(12)),
        );
        task_repository.save(&mut task1).unwrap();

        let seq_id = task_repository
            .issue_sequential_id(task1.aggregate_id())
            .unwrap();
        assert_eq!(seq_id, SequentialID::new(1));

        let mut task2 = Task::create(
            "test this task".into(),
            Some(Priority::new(21)),
            Some(Cost::new(22)),
        );
        task_repository.save(&mut task2).unwrap();

        let seq_id = task_repository
            .issue_sequential_id(task2.aggregate_id())
            .unwrap();
        assert_eq!(seq_id, SequentialID::new(2));
    }

    #[test]
    fn test_fail_issue_sequential_id_with_noexist_task_id() {
        let task_repository = TaskRepository::new(rusqlite::Connection::open_in_memory().unwrap());
        task_repository.create_table_if_not_exists().unwrap();

        task_repository
            .issue_sequential_id(AggregateID::new())
            .unwrap_err();
    }
}
