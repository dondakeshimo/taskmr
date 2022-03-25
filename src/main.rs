use rusqlite::Connection;

use taskmr::infra::sqlite::task_repository::TaskRepository;
use taskmr::presentation::command::cli::Cli;
use taskmr::usecase::add_task_usecase::AddTaskUseCase;

fn main() {
    let task_repository = TaskRepository::new(Connection::open_in_memory().unwrap());
    task_repository.create_table_if_not_exists().unwrap();
    let add_task_usecase = AddTaskUseCase::new(Box::new(task_repository));
    let cli = Cli::new(add_task_usecase);
    cli.handle();
}
