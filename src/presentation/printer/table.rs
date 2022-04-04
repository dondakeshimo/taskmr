use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

use crate::usecase::list_task_usecase::TaskDTO;

/// Printer to transrate tasks into table style string.
pub struct TablePrinter<W: Write> {
    tab_writer: TabWriter<W>,
}

impl<W: Write> TablePrinter<W> {
    /// construct TablePrinter.
    pub fn new(w: W) -> Self {
        TablePrinter {
            tab_writer: TabWriter::new(w),
        }
    }

    /// print out with given writer.
    pub fn print(&mut self, tasks: Vec<TaskDTO>) -> Result<()> {
        writeln!(&mut self.tab_writer, "ID\tTitle\tPriority\tCost")?;

        for t in tasks {
            writeln!(
                &mut self.tab_writer,
                "{}\t{}\t{}\t{}",
                t.id, t.title, t.priority, t.cost
            )?;
        }

        self.tab_writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute() {
        #[derive(Debug)]
        struct Args {
            tasks: Vec<TaskDTO>,
        }

        #[derive(Debug)]
        struct TestCase {
            args: Args,
            want: String,
            name: String,
        }

        let table = [
            TestCase {
                name: String::from("normal: with priority and cost"),
                args: Args { tasks: vec![] },
                want: String::from("ID  Title  Priority  Cost\n"),
            },
            TestCase {
                name: String::from("normal: with priority and cost"),
                args: Args {
                    tasks: vec![
                        TaskDTO {
                            id: 1,
                            title: "title1".to_owned(),
                            priority: 1,
                            cost: 1,
                        },
                        TaskDTO {
                            id: 2,
                            title: "title2".to_owned(),
                            priority: 2,
                            cost: 2,
                        },
                        TaskDTO {
                            id: 3,
                            title: "title3".to_owned(),
                            priority: 3,
                            cost: 3,
                        },
                    ],
                },
                want: String::from("ID  Title   Priority  Cost\n1   title1  1         1\n2   title2  2         2\n3   title3  3         3\n"),
            },
        ];

        for test_case in table {
            let mut table_printer = TablePrinter::new(vec![]);
            table_printer.print(test_case.args.tasks).unwrap();
            let got = String::from_utf8(table_printer.tab_writer.into_inner().unwrap()).unwrap();

            assert_eq!(
                &*got, test_case.want,
                "Failed in the \"{}\".",
                test_case.name,
            );
        }
    }
}
