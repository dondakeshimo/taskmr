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
    pub fn new(a_title: String, a_priority: Option<i32>, a_cost: Option<i32>) -> Task {
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
            title: a_title,
            is_closed: false,
            priority,
            cost,
            elapsed_time: time::Duration::from_secs(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Args {
        a_title: String,
        a_priority: Option<i32>,
        a_cost: Option<i32>,
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
                args: Args{a_title: String::from("title1"), a_priority: Some(100), a_cost: Some(100)},
                expected: Task{id: 0, title: String::from("title1"), is_closed: false, priority: 100, cost: 100, elapsed_time: time::Duration::from_secs(0)},
            },
            TestCase {
                name: String::from("nominal: withtout priority and cost"),
                args: Args{a_title: String::from("title2"), a_priority: None, a_cost: None},
                expected: Task{id: 0, title: String::from("title2"), is_closed: false, priority: 10, cost: 10, elapsed_time: time::Duration::from_secs(0)},
            },
        ];

        for test_case in table {
            assert_eq!(
                Task::new(test_case.args.a_title, test_case.args.a_priority, test_case.args.a_cost),
                test_case.expected,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
