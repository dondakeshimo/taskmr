use std::time::Duration;

use anyhow::Result;

/// Task ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ID(i64);

impl ID {
    /// construct a Task ID.
    pub fn new(id: i64) -> Self {
        ID(id)
    }

    /// get a Task ID as primitive type.
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// Task Priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Priority(i32);

impl Priority {
    /// construct a task priority.
    pub fn new(priority: i32) -> Self {
        Priority(priority)
    }

    /// get a task priority as primitive type.
    pub fn get(&self) -> i32 {
        self.0
    }
}

/// Task Cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cost(i32);

impl Cost {
    /// construct a task cost.
    pub fn new(cost: i32) -> Self {
        Cost(cost)
    }

    /// get a task cost as primitive type.
    pub fn get(&self) -> i32 {
        self.0
    }
}

/// Task is a entity representing what you should do.
#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    id: ID,
    title: String,
    is_closed: bool,
    priority: Priority,
    cost: Cost,
    elapsed_time: Duration,
}

impl Task {
    /// construct new Task.
    pub fn new(title: String, a_priority: Option<Priority>, a_cost: Option<Cost>) -> Task {
        let default_priorty = Priority(10);
        let priority = match a_priority {
            Some(p) => p,
            _ => default_priorty,
        };

        let default_cost = Cost(10);
        let cost = match a_cost {
            Some(c) => c,
            _ => default_cost,
        };

        Task {
            id: ID(0),
            title,
            is_closed: false,
            priority,
            cost,
            elapsed_time: Duration::from_secs(0),
        }
    }

    /// construct new Task from repository.
    /// WARNING: don't use this function any layer other than repository.
    pub fn from_repository(
        id: ID,
        title: String,
        is_closed: bool,
        priority: Priority,
        cost: Cost,
        elapsed_time: Duration,
    ) -> Task {
        Task {
            id,
            title,
            is_closed,
            priority,
            cost,
            elapsed_time,
        }
    }

    /// get id.
    pub fn id(&self) -> ID {
        self.id
    }

    /// get title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// get is_closed.
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// get priority.
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// get cost.
    pub fn cost(&self) -> Cost {
        self.cost
    }

    /// get elapsed_time.
    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        #[derive(Debug)]
        struct Args {
            title: String,
            priority: Option<Priority>,
            cost: Option<Cost>,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            expected: Task,
            name: String,
        }

        let table = [
            TestCase {
                name: String::from("nominal: with priority and cost"),
                args: Args {
                    title: String::from("title1"),
                    priority: Some(Priority(100)),
                    cost: Some(Cost(100)),
                },
                expected: Task {
                    id: ID(0),
                    title: String::from("title1"),
                    is_closed: false,
                    priority: Priority(100),
                    cost: Cost(100),
                    elapsed_time: Duration::from_secs(0),
                },
            },
            TestCase {
                name: String::from("nominal: withtout priority and cost"),
                args: Args {
                    title: String::from("title2"),
                    priority: None,
                    cost: None,
                },
                expected: Task {
                    id: ID(0),
                    title: String::from("title2"),
                    is_closed: false,
                    priority: Priority(10),
                    cost: Cost(10),
                    elapsed_time: Duration::from_secs(0),
                },
            },
        ];

        for test_case in table {
            assert_eq!(
                Task::new(
                    test_case.args.title,
                    test_case.args.priority,
                    test_case.args.cost
                ),
                test_case.expected,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }

    #[test]
    fn test_from_repository_and_getter() {
        #[derive(Debug)]
        struct Args {
            id: ID,
            title: String,
            is_closed: bool,
            priority: Priority,
            cost: Cost,
            elapsed_time: Duration,
        }

        #[derive(Debug)]
        struct Wants<'w> {
            id: ID,
            title: &'w str,
            is_closed: bool,
            priority: Priority,
            cost: Cost,
            elapsed_time: Duration,
        }

        #[derive(Debug)]
        struct TestCase<'tc> {
            args: Args,
            want: Wants<'tc>,
            name: String,
        }

        let table = [TestCase {
            name: String::from("nominal: with priority and cost"),
            args: Args {
                id: ID(1),
                title: String::from("title1"),
                is_closed: true,
                priority: Priority(2),
                cost: Cost(3),
                elapsed_time: Duration::from_secs(4),
            },
            want: Wants {
                id: ID(1),
                title: "title1",
                is_closed: true,
                priority: Priority(2),
                cost: Cost(3),
                elapsed_time: Duration::from_secs(4),
            },
        }];

        for test_case in table {
            let got = Task::from_repository(
                test_case.args.id,
                test_case.args.title,
                test_case.args.is_closed,
                test_case.args.priority,
                test_case.args.cost,
                test_case.args.elapsed_time,
            );
            assert_eq!(
                got.id(),
                test_case.want.id,
                "Failed in the \"{}\".",
                test_case.name
            );
            assert_eq!(
                got.title(),
                test_case.want.title,
                "Failed in the \"{}\".",
                test_case.name
            );
            assert_eq!(
                got.is_closed(),
                test_case.want.is_closed,
                "Failed in the \"{}\".",
                test_case.name
            );
            assert_eq!(
                got.priority(),
                test_case.want.priority,
                "Failed in the \"{}\".",
                test_case.name
            );
            assert_eq!(
                got.cost(),
                test_case.want.cost,
                "Failed in the \"{}\".",
                test_case.name
            );
            assert_eq!(
                got.elapsed_time(),
                test_case.want.elapsed_time,
                "Failed in the \"{}\".",
                test_case.name
            );
        }
    }
}

/// ITaskRepository define interface of task repository.
pub trait ITaskRepository {
    fn find_by_id(&self, id: ID) -> Result<Option<Task>>;
    fn add(&self, a_task: Task) -> Result<ID>;
}
