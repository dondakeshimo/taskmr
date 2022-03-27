use rusqlite::Connection;
use std::fs;
use std::io;
use std::process;
use std::rc::Rc;

use taskmr::domain::task::ITaskRepository;
use taskmr::infra::sqlite::task_repository::TaskRepository;
use taskmr::presentation::command::cli::Cli;
use taskmr::presentation::printer::table::TablePrinter;
use taskmr::usecase::add_task_usecase::AddTaskUseCase;
use taskmr::usecase::list_task_usecase::ListTaskUseCase;

fn main() {
    let mut db_file_path = dirs::config_dir().unwrap_or_else(|| {
        eprintln!("Couldn't find out config directory.");
        process::exit(1)
    });
    db_file_path.push("taskmr");
    fs::create_dir_all(&db_file_path).unwrap_or_else(|err| {
        eprintln!(
            "Couldn't create taskmr directory in your config directory: {}",
            err
        );
        process::exit(1)
    });
    db_file_path.push("taskmr.db");

    let task_repository =
        TaskRepository::new(Connection::open(&db_file_path).unwrap_or_else(|err| {
            eprintln!("Couldn't connect your task database: {}", err);
            process::exit(1)
        }));

    task_repository
        .create_table_if_not_exists()
        .unwrap_or_else(|err| {
            eprintln!("Failed to create tables on your database: {}", err);
            process::exit(1)
        });

    let rc_tr: Rc<dyn ITaskRepository> = Rc::new(task_repository);
    let add_task_usecase = AddTaskUseCase::new(Rc::clone(&rc_tr));
    let list_task_usecase = ListTaskUseCase::new(rc_tr);
    let table_printer = TablePrinter::new(io::stdout());
    let mut cli = Cli::new(add_task_usecase, list_task_usecase, table_printer);
    cli.handle();
}
