use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ddd::component::{
    AggregateID, AggregateRoot, Command, DomainEvent, DomainEventEnvelope, Entity, Repository,
    ValueObject,
};

/// Sequential ID.
/// This ID is for shortcut to specifying the task.
/// It is assigned lazily because it is a serial number which is generated after query latest
/// number at the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequentialID(i64);

impl SequentialID {
    /// construct a SequentialID.
    pub fn new(id: i64) -> Self {
        SequentialID(id)
    }

    /// get a Task ID as primitive type.
    pub fn to_i64(&self) -> i64 {
        self.0
    }
}

impl ValueObject for SequentialID {}

/// Task Priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Priority(i32);

impl Priority {
    /// construct a task priority.
    pub fn new(priority: i32) -> Self {
        Priority(priority)
    }

    /// get a task priority as primitive type.
    pub fn to_i32(&self) -> i32 {
        self.0
    }
}

impl ValueObject for Priority {}

const DEFAULT_PRIORITY: Priority = Priority(10);

/// Task Cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost(i32);

impl Cost {
    /// construct a task cost.
    pub fn new(cost: i32) -> Self {
        Cost(cost)
    }

    /// get a task cost as primitive type.
    pub fn to_i32(&self) -> i32 {
        self.0
    }
}

impl ValueObject for Cost {}

const DEFAULT_COST: Cost = Cost(10);

/// TaskCommand is a command set to mutate the Task.
#[derive(Debug, PartialEq, Eq)]
pub enum TaskCommand {
    Close,
    EditTitle { title: String },
    RescoreCost { cost: Cost },
    RescorePriority { priority: Priority },
}

impl Command for TaskCommand {}

const TASK_DOMAIN_EVENT_VERSION: i32 = 1;

/// TaskDomainEvent is a event issued when the task mutated.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskDomainEvent {
    Created {
        aggregate_id: AggregateID,
        sequential_id: SequentialID,
    },
    Closed,
    TitleEdited {
        title: String,
    },
    CostRescored {
        cost: Cost,
    },
    PriorityRescored {
        priority: Priority,
    },
}

impl DomainEvent for TaskDomainEvent {}

/// Task is a entity representing what you should do.
#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    aggregate_id: AggregateID,
    version: i32,
    sequential_id: SequentialID,
    events: Vec<DomainEventEnvelope<TaskDomainEvent>>,
    title: String,
    is_closed: bool,
    priority: Priority,
    cost: Cost,
    elapsed_time: Duration,
}

#[derive(Debug)]
pub struct TaskSource {
    pub aggregate_id: AggregateID,
    pub sequential_id: SequentialID,
    pub title: String,
    pub priority: Option<Priority>,
    pub cost: Option<Cost>,
}

impl Task {
    /// create a Task.
    pub fn create(task_source: TaskSource) -> Task {
        let mut task = Task::new(task_source.aggregate_id, task_source.sequential_id);
        task.record_event(TaskDomainEvent::Created {
            aggregate_id: task.aggregate_id(),
            sequential_id: task.sequential_id(),
        });

        task.edit_title(task_source.title);

        if let Some(p) = task_source.priority {
            task.rescore_priority(p);
        }

        if let Some(c) = task_source.cost {
            task.rescore_cost(c);
        }

        task
    }

    /// construct new default Task.
    fn new(aggregate_id: AggregateID, sequential_id: SequentialID) -> Task {
        Task {
            aggregate_id,
            version: 0,
            sequential_id,
            events: vec![],
            title: "".into(),
            is_closed: false,
            priority: DEFAULT_PRIORITY,
            cost: DEFAULT_COST,
            elapsed_time: Duration::from_secs(0),
        }
    }

    /// reconstruct the Task from events.
    pub fn recreate(
        aggregate_id: AggregateID,
        sequential_id: SequentialID,
        events: Vec<DomainEventEnvelope<TaskDomainEvent>>,
    ) -> Task {
        let mut task = Task::new(aggregate_id, sequential_id);

        for event in events {
            task.apply(event.event());
            task.increment_version();
        }

        task
    }

    /// get aggregate id.
    pub fn aggregate_id(&self) -> AggregateID {
        self.aggregate_id
    }

    /// increment version.
    /// This function is invoked every time when TaskDomainEvent is issued.
    fn increment_version(&mut self) {
        self.version += 1;
    }

    /// get sequential id.
    pub fn sequential_id(&self) -> SequentialID {
        self.sequential_id
    }

    /// get title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// edit title.
    fn edit_title(&mut self, title: String) {
        self.record_event(TaskDomainEvent::TitleEdited { title });
    }

    /// get is_closed flag.
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// close the task.
    fn close(&mut self) {
        self.record_event(TaskDomainEvent::Closed);
    }

    /// get priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// rescore priority.
    pub fn rescore_priority(&mut self, priority: Priority) {
        self.record_event(TaskDomainEvent::PriorityRescored { priority });
    }

    /// get cost.
    pub fn cost(&self) -> Cost {
        self.cost
    }

    /// rescore cost.
    pub fn rescore_cost(&mut self, cost: Cost) {
        self.record_event(TaskDomainEvent::CostRescored { cost });
    }

    /// get elapsed_time.
    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }
}

impl Entity for Task {
    type Id = AggregateID;

    fn id(&self) -> Self::Id {
        self.aggregate_id()
    }
}

/// Error is used in or outer Application Service layer.
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("the event cannot apply")]
    InvalidEvent,
}

impl AggregateRoot for Task {
    type Command = TaskCommand;
    type DomainEvent = TaskDomainEvent;

    fn execute(&mut self, command: Self::Command) -> Result<()> {
        match command {
            TaskCommand::Close => self.close(),
            TaskCommand::EditTitle { title } => self.edit_title(title),
            TaskCommand::RescoreCost { cost } => self.rescore_cost(cost),
            TaskCommand::RescorePriority { priority } => self.rescore_priority(priority),
        }
        Ok(())
    }

    fn apply(&mut self, event: &Self::DomainEvent) {
        match event {
            TaskDomainEvent::Created { aggregate_id, .. } => self.aggregate_id = *aggregate_id,
            TaskDomainEvent::Closed { .. } => self.is_closed = true,
            TaskDomainEvent::TitleEdited { title, .. } => self.title = title.to_owned(),
            TaskDomainEvent::CostRescored { cost, .. } => self.cost = *cost,
            TaskDomainEvent::PriorityRescored { priority, .. } => self.priority = *priority,
        }
    }

    fn events(&self) -> &Vec<DomainEventEnvelope<Self::DomainEvent>> {
        &self.events
    }

    fn clear_events(&mut self) {
        self.events.clear();
    }

    fn record_event(&mut self, event: Self::DomainEvent) {
        self.apply(&event);
        let ee = DomainEventEnvelope::new(event, self.version, TASK_DOMAIN_EVENT_VERSION);
        self.events.push(ee);
        self.increment_version();
    }
}

/// IESTaskRepository define interface of task repository.
pub trait IESTaskRepository: Repository<Task> {
    /// issue_sequential_id issue SequentialID incremented from latest serial number.
    fn issue_sequential_id(&self, aggregate_id: AggregateID) -> Result<SequentialID>;

    /// load_by_sequential_id loads Task by sequential_id.
    fn load_by_sequential_id(&self, sequential_id: SequentialID) -> Result<Option<Task>>;

    /// load_all_sequential_ids loads all sequential_ids.
    fn load_all_sequential_ids(&self) -> Result<Vec<SequentialID>>;
}

/// RepositoryComponent returns Repository.
/// This is CakePattern.
/// SEE: http://eed3si9n.com/ja/real-world-scala-dependency-injection-di/
pub trait IESTaskRepositoryComponent {
    type Repository: IESTaskRepository;

    /// repository returns Repository.
    fn repository(&self) -> &Self::Repository;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_events(got: &Vec<DomainEventEnvelope<TaskDomainEvent>>, want: &Vec<TaskDomainEvent>) {
        let mut counter: i32 = 0;
        for (g, w) in got.iter().zip(want.iter()) {
            assert_eq!(g.aggregate_version(), counter);
            assert_eq!(g.event(), w);
            counter += 1;
        }
    }

    #[test]
    fn test_create() {
        #[derive(Debug, PartialEq, Eq)]
        struct TargetState {
            title: String,
            priority: Priority,
            cost: Cost,
        }

        #[derive(Debug)]
        struct TestCase {
            args: TaskSource,
            want_state: TargetState,
            want_events: Vec<TaskDomainEvent>,
            name: String,
        }

        let aggregate_id = AggregateID::new();

        let table = [
            TestCase {
                name: String::from("with priority and cost"),
                args: TaskSource {
                    aggregate_id: aggregate_id.clone(),
                    sequential_id: SequentialID::new(10),
                    title: String::from("title1"),
                    priority: Some(Priority(100)),
                    cost: Some(Cost(100)),
                },
                want_state: TargetState {
                    title: "title1".into(),
                    priority: Priority::new(100),
                    cost: Cost::new(100),
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: "title1".into(),
                    },
                    TaskDomainEvent::PriorityRescored {
                        priority: Priority::new(100),
                    },
                ],
            },
            TestCase {
                name: String::from("withtout priority and cost"),
                args: TaskSource {
                    aggregate_id: aggregate_id.clone(),
                    sequential_id: SequentialID::new(10),
                    title: String::from("title2"),
                    priority: None,
                    cost: None,
                },
                want_state: TargetState {
                    title: "title2".into(),
                    priority: DEFAULT_PRIORITY,
                    cost: DEFAULT_COST,
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: "title2".into(),
                    },
                ],
            },
        ];

        for test_case in table {
            let task = Task::create(test_case.args);
            let got_state = TargetState {
                title: task.title().into(),
                priority: task.priority(),
                cost: task.cost(),
            };

            assert_eq!(
                got_state, test_case.want_state,
                "Failed in the \"{}\".",
                test_case.name,
            );

            assert_events(task.events(), &test_case.want_events);
        }
    }

    #[test]
    fn test_execute() {
        const TITLE: &str = "title";

        #[derive(Debug, PartialEq, Eq)]
        struct TargetState {
            title: String,
            priority: Priority,
            cost: Cost,
            is_closed: bool,
            sequential_id: SequentialID,
        }

        #[derive(Debug)]
        struct TestCase {
            command: TaskCommand,
            want_state: TargetState,
            want_events: Vec<TaskDomainEvent>,
            name: String,
        }

        let aggregate_id = AggregateID::new();

        let table = [
            TestCase {
                name: String::from("close"),
                command: TaskCommand::Close,
                want_state: TargetState {
                    title: TITLE.to_owned(),
                    priority: DEFAULT_PRIORITY,
                    cost: DEFAULT_COST,
                    is_closed: true,
                    sequential_id: SequentialID::new(10),
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: TITLE.to_owned(),
                    },
                    TaskDomainEvent::Closed,
                ],
            },
            TestCase {
                name: String::from("rescore priority"),
                command: TaskCommand::RescorePriority {
                    priority: Priority::new(100),
                },
                want_state: TargetState {
                    title: TITLE.to_owned(),
                    priority: Priority::new(100),
                    cost: DEFAULT_COST,
                    is_closed: false,
                    sequential_id: SequentialID::new(10),
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: TITLE.to_owned(),
                    },
                    TaskDomainEvent::PriorityRescored {
                        priority: Priority::new(100),
                    },
                ],
            },
            TestCase {
                name: String::from("rescore cost"),
                command: TaskCommand::RescoreCost {
                    cost: Cost::new(100),
                },
                want_state: TargetState {
                    title: TITLE.to_owned(),
                    priority: DEFAULT_PRIORITY,
                    cost: Cost::new(100),
                    is_closed: false,
                    sequential_id: SequentialID::new(10),
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: TITLE.to_owned(),
                    },
                    TaskDomainEvent::CostRescored {
                        cost: Cost::new(100),
                    },
                ],
            },
            TestCase {
                name: String::from("edit title"),
                command: TaskCommand::EditTitle {
                    title: "edited title".to_owned(),
                },
                want_state: TargetState {
                    title: "edited title".to_owned(),
                    priority: DEFAULT_PRIORITY,
                    cost: DEFAULT_COST,
                    is_closed: false,
                    sequential_id: SequentialID::new(10),
                },
                want_events: vec![
                    TaskDomainEvent::Created {
                        aggregate_id: aggregate_id.clone(),
                        sequential_id: SequentialID::new(10),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: TITLE.to_owned(),
                    },
                    TaskDomainEvent::TitleEdited {
                        title: "edited title".to_owned(),
                    },
                ],
            },
        ];

        for test_case in table {
            let mut task = Task::create(TaskSource {
                aggregate_id: aggregate_id.clone(),
                sequential_id: SequentialID::new(10),
                title: TITLE.to_owned(),
                priority: None,
                cost: None,
            });
            task.execute(test_case.command).unwrap();
            let got_state = TargetState {
                title: task.title().into(),
                priority: task.priority(),
                cost: task.cost(),
                is_closed: task.is_closed(),
                sequential_id: task.sequential_id(),
            };

            assert_eq!(
                got_state, test_case.want_state,
                "Failed in the \"{}\".",
                test_case.name,
            );

            assert_events(task.events(), &test_case.want_events);
        }
    }
}
