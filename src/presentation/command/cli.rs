use clap::{Parser, Subcommand};

use crate::usecase::add_task_usecase::{AddTaskUseCase, AddTaskUseCaseInput};
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
    /// list tasks.
    List {},
}

pub struct Cli {
    add_task_usecase: AddTaskUseCase,
    list_task_usecase: ListTaskUseCase,
}

impl Cli {
    pub fn new(add_task_usecase: AddTaskUseCase, list_task_usecase: ListTaskUseCase) -> Self {
        Cli {
            add_task_usecase,
            list_task_usecase,
        }
    }

    pub fn handle(&self) {
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
            SubCommands::List {} => {
                let task_dto = self
                    .list_task_usecase
                    .execute(ListTaskUseCaseInput {})
                    .unwrap();
                println!("{:?}", task_dto);
            }
        }
    }
}
