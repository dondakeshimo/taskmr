use rusqlite::Connection;
use std::fs;
use std::io;
use std::process;
use std::rc::Rc;

use taskmr::domain::task::ITaskRepository;
use taskmr::infra::sqlite::es_task_repository::TaskRepository as ESTaskRepository;
use taskmr::infra::sqlite::task_repository::TaskRepository;
use taskmr::presentation::command::cli::Cli;
use taskmr::presentation::printer::table::TablePrinter;
use taskmr::usecase::add_task_usecase::AddTaskUseCase;
use taskmr::usecase::close_task_usecase::CloseTaskUseCase;
use taskmr::usecase::edit_task_usecase::EditTaskUseCase;
use taskmr::usecase::list_task_usecase::ListTaskUseCase;


struct Parent {}

impl Parent {
    fn child(&self) -> Child {
        Child{ parent: self }
    }
}

struct Child<'p> {
    parent: &'p Parent,
}

impl<'p> Child<'p> {
    fn go_out(self) {
        println!("go out")
    }
}

struct Holder<'p> {
    parent: Parent,
    child: Option<Child<'p>>,
}

impl<'p> Holder<'p> {
    fn set_child(&'p mut self) {
        self.child = Some(self.parent.child());
    }

    fn go_out(&mut self) {
        let child = self.child.take();

        if let Some(c) = child {
            c.go_out();
        }
    }
}

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

    let es_task_repository =
        ESTaskRepository::new(Connection::open(&db_file_path).unwrap_or_else(|err| {
            eprintln!("Couldn't connect your task database: {}", err);
            process::exit(1)
        }));

    es_task_repository
        .create_table_if_not_exists()
        .unwrap_or_else(|err| {
            eprintln!("Failed to create tables on your database: {}", err);
            process::exit(1)
        });

    let rc_tr: Rc<dyn ITaskRepository> = Rc::new(task_repository);
    let add_task_usecase = AddTaskUseCase::new(Rc::clone(&rc_tr));
    let close_task_usecase = CloseTaskUseCase::new(Rc::clone(&rc_tr));
    let edit_task_usecase = EditTaskUseCase::new(Rc::clone(&rc_tr));
    let list_task_usecase = ListTaskUseCase::new(rc_tr);
    let table_printer = TablePrinter::new(io::stdout());
    let mut cli = Cli::new(
        add_task_usecase,
        close_task_usecase,
        edit_task_usecase,
        list_task_usecase,
        table_printer,
        es_task_repository,
    );
    cli.handle();
}
