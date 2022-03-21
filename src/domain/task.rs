use std::time;

/// Task is a entity representing what you should do.
#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    id: i32,
    title: String,
    is_closed: bool,
    priority: i32,
    cost: i32,
    elapsed_time: time::Duration,
}

impl Task {
    /// construct new Task.
    pub fn new(title: String, a_priority: Option<i32>, a_cost: Option<i32>) -> Task {
        let default_priorty = 10;
        let priority = match a_priority {
            Some(p) => p,
            _ => default_priorty,
        };

        let default_cost = 10;
        let cost = match a_cost {
            Some(c) => c,
            _ => default_cost,
        };

        Task {
            id: 0,
            title,
            is_closed: false,
            priority,
            cost,
            elapsed_time: time::Duration::from_secs(0),
        }
    }

    /// construct new Task from repository.
    /// WARNING: don't use this function any layer other than repository.
    pub fn from_repository(
        id: i32,
        title: String,
        is_closed: bool,
        priority: i32,
        cost: i32,
        elapsed_time: time::Duration,
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
    pub fn id(&self) -> i32 {
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
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// get cost.
    pub fn cost(&self) -> i32 {
        self.cost
    }

    /// get elapsed_time.
    pub fn elapsed_time(&self) -> time::Duration {
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
            priority: Option<i32>,
            cost: Option<i32>,
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
                    priority: Some(100),
                    cost: Some(100),
                },
                expected: Task {
                    id: 0,
                    title: String::from("title1"),
                    is_closed: false,
                    priority: 100,
                    cost: 100,
                    elapsed_time: time::Duration::from_secs(0),
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
                    id: 0,
                    title: String::from("title2"),
                    is_closed: false,
                    priority: 10,
                    cost: 10,
                    elapsed_time: time::Duration::from_secs(0),
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
            id: i32,
            title: String,
            is_closed: bool,
            priority: i32,
            cost: i32,
            elapsed_time: time::Duration,
        }

        #[derive(Debug)]
        struct Wants<'w> {
            id: i32,
            title: &'w str,
            is_closed: bool,
            priority: i32,
            cost: i32,
            elapsed_time: time::Duration,
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
                id: 1,
                title: String::from("title1"),
                is_closed: true,
                priority: 2,
                cost: 3,
                elapsed_time: time::Duration::from_secs(4),
            },
            want: Wants {
                id: 1,
                title: "title1",
                is_closed: true,
                priority: 2,
                cost: 3,
                elapsed_time: time::Duration::from_secs(4),
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
