use rusqlite::Connection;
use std::fs;
use std::process;

use taskmr::infra::sqlite::task_repository::TaskRepository;
use taskmr::presentation::command::cli::Cli;
use taskmr::usecase::add_task_usecase::AddTaskUseCase;

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

    let add_task_usecase = AddTaskUseCase::new(Box::new(task_repository));
    let cli = Cli::new(add_task_usecase);
    cli.handle();
}
