use clap::{Parser, Subcommand};
use std::{io, process};

use crate::presentation::printer::table::TablePrinter;
use crate::usecase::add_task_usecase::{AddTaskUseCase, AddTaskUseCaseInput};
use crate::usecase::close_task_usecase::{CloseTaskUseCase, CloseTaskUseCaseInput};
use crate::usecase::list_task_usecase::{ListTaskUseCase, ListTaskUseCaseInput};

/// A fictional versioning CLI
#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// add a task.
    #[clap(arg_required_else_help = true)]
    Add {
        /// Title of a task.
        title: String,
        /// Priority of a task.
        #[clap(short, long)]
        priority: Option<i32>,
        /// Cost of a task.
        #[clap(short, long)]
        cost: Option<i32>,
    },
    /// close a task.
    #[clap(arg_required_else_help = true)]
    Close {
        /// id of the task.
        ids: Vec<i64>,
    },
    /// list tasks.
    List {},
}

pub struct Cli {
    add_task_usecase: AddTaskUseCase,
    close_task_usecase: CloseTaskUseCase,
    list_task_usecase: ListTaskUseCase,
    table_printer: TablePrinter<io::Stdout>,
}

impl Cli {
    pub fn new(
        add_task_usecase: AddTaskUseCase,
        close_task_usecase: CloseTaskUseCase,
        list_task_usecase: ListTaskUseCase,
        table_printer: TablePrinter<io::Stdout>,
    ) -> Self {
        Cli {
            add_task_usecase,
            close_task_usecase,
            list_task_usecase,
            table_printer,
        }
    }

    pub fn handle(&mut self) {
        let args = Command::parse();

        match &args.command {
            SubCommands::Add {
                title,
                priority,
                cost,
            } => {
                let input = AddTaskUseCaseInput {
                    title: title.to_owned(),
                    priority: priority.to_owned(),
                    cost: cost.to_owned(),
                };
                self.add_task_usecase.execute(input).unwrap();
            }
            SubCommands::Close { ids } => {
                let mut is_all_success = true;
                for id in ids {
                    match self
                        .close_task_usecase
                        .execute(CloseTaskUseCaseInput { id: id.to_owned() })
                    {
                        Ok(r_id) => {
                            println!("Close the task for id `{}`.", r_id.get())
                        }
                        Err(err) => {
                            is_all_success = false;
                            eprintln!("Failed to closing the task: {}.", err)
                        }
                    }
                }

                if !is_all_success {
                    process::exit(1);
                }
            }
            SubCommands::List {} => {
                let task_dto = self
                    .list_task_usecase
                    .execute(ListTaskUseCaseInput {})
                    .unwrap();
                self.table_printer.print(task_dto).unwrap();
            }
        }
    }
}
