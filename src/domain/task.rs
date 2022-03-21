use std::time;

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
    pub fn new(title: String, a_priority: Option<i32>, a_cost: Option<i32>) -> Task {
        let default_priorty = 10;
        let priority;
        match a_priority {
            Some(p) => priority = p,
            _ => priority = default_priorty,
        }

        let default_cost = 10;
        let cost;
        match a_cost {
            Some(c) => cost = c,
            _ => cost = default_cost,
        }

        Task {
            id: 0,
            title,
            is_closed: false,
            priority,
            cost,
            elapsed_time: time::Duration::from_secs(0),
        }
    }

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

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn cost(&self) -> i32 {
        self.cost
    }

    pub fn elapsed_time(&self) -> time::Duration {
        self.elapsed_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_new() {
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
}
