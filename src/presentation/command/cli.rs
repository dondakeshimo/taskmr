use clap::{Parser, Subcommand};
use std::{io, process};

use crate::domain::es_task::{IESTaskRepository, IESTaskRepositoryComponent};
use crate::presentation::printer::table::TablePrinter;
use crate::usecase::add_es_task_usecase::AddTaskUseCase as ESAddTaskUseCase;
use crate::usecase::add_es_task_usecase::AddTaskUseCaseComponent;
use crate::usecase::add_es_task_usecase::AddTaskUseCaseInput as ESAddTaskUseCaseInput;
use crate::usecase::add_task_usecase::{AddTaskUseCase, AddTaskUseCaseInput};
use crate::usecase::close_task_usecase::{CloseTaskUseCase, CloseTaskUseCaseInput};
use crate::usecase::edit_task_usecase::{EditTaskUseCase, EditTaskUseCaseInput};
use crate::usecase::list_task_usecase::{ListTaskUseCase, ListTaskUseCaseInput};

/// Task ManageR.
#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    command: SubCommands,
}

/// Subcommands define cli subcommands.
#[derive(Subcommand)]
enum SubCommands {
    /// Add a task.
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
    /// ESAdd add a task with event sourcing.
    #[clap(arg_required_else_help = true)]
    ESAdd {
        /// Title of a task.
        title: String,
        /// Priority of a task.
        #[clap(short, long)]
        priority: Option<i32>,
        /// Cost of a task.
        #[clap(short, long)]
        cost: Option<i32>,
    },
    /// Close tasks.
    #[clap(arg_required_else_help = true)]
    Close {
        /// ids of the tasks.
        ids: Vec<i64>,
    },
    /// Edit the task.
    #[clap(arg_required_else_help = true)]
    Edit {
        /// id of the task.
        id: i64,
        /// Title of the task.
        #[clap(short, long)]
        title: Option<String>,
        /// Priority of the task.
        #[clap(short, long)]
        priority: Option<i32>,
        /// Cost of the task.
        #[clap(short, long)]
        cost: Option<i32>,
    },
    /// List tasks.
    List {},
}

/// Cli has structs to execute usecases.
pub struct Cli<TR: IESTaskRepository> {
    add_task_usecase: AddTaskUseCase,
    close_task_usecase: CloseTaskUseCase,
    edit_task_usecase: EditTaskUseCase,
    list_task_usecase: ListTaskUseCase,
    table_printer: TablePrinter<io::Stdout>,
    es_task_repository: TR,
}

impl<TR: IESTaskRepository> IESTaskRepositoryComponent for Cli<TR> {
    type Repository = TR;
    fn repository(&self) -> &Self::Repository {
        &self.es_task_repository
    }
}

impl<TR: IESTaskRepository> AddTaskUseCaseComponent for Cli<TR> {
    type AddTaskUseCase = Self;
    fn add_task_usecase(&self) -> &Self::AddTaskUseCase {
        self
    }
}

impl<TR: IESTaskRepository> Cli<TR> {
    /// construct Cli.
    pub fn new(
        add_task_usecase: AddTaskUseCase,
        close_task_usecase: CloseTaskUseCase,
        edit_task_usecase: EditTaskUseCase,
        list_task_usecase: ListTaskUseCase,
        table_printer: TablePrinter<io::Stdout>,
        es_task_repository: TR,
    ) -> Self {
        Cli {
            add_task_usecase,
            close_task_usecase,
            edit_task_usecase,
            list_task_usecase,
            table_printer,
            es_task_repository,
        }
    }

    /// handle user input.
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
            SubCommands::ESAdd {
                title,
                priority,
                cost,
            } => {
                let input = ESAddTaskUseCaseInput {
                    title: title.to_owned(),
                    priority: priority.to_owned(),
                    cost: cost.to_owned(),
                };
                self.add_task_usecase().execute(input).unwrap();
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
                            eprintln!("Failed to close the task: {}.", err)
                        }
                    }
                }

                if !is_all_success {
                    process::exit(1);
                }
            }
            SubCommands::Edit {
                id,
                title,
                priority,
                cost,
            } => {
                let input = EditTaskUseCaseInput {
                    id: id.to_owned(),
                    title: title.to_owned(),
                    priority: priority.to_owned(),
                    cost: cost.to_owned(),
                };
                self.edit_task_usecase.execute(input).unwrap_or_else(|err| {
                    eprintln!("Failed to edit the task: {}.", err);
                    process::exit(1);
                });
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
