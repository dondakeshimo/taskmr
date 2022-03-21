use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Clones repos
    #[clap(arg_required_else_help = true)]
    Clone {
        /// The remote to clone
        remote: String,
    },
    /// pushes things
    #[clap(arg_required_else_help = true)]
    Push {
        /// The remote to target
        remote: String,
    },
    /// adds things
    #[clap(arg_required_else_help = true)]
    Add {
        /// Stuff to add
        #[clap(required = true, parse(from_os_str))]
        path: Vec<PathBuf>,
    },
    #[clap(external_subcommand)]
    External(Vec<OsString>),
}

pub fn handle() {
    let args = Command::parse();

    match &args.command {
        SubCommands::Clone { remote } => {
            println!("Cloning {}", remote);
        }
        SubCommands::Push { remote } => {
            println!("Pushing to {}", remote);
        }
        SubCommands::Add { path } => {
            println!("Adding {:?}", path);
        }
        SubCommands::External(args) => {
            println!("Calling out to {:?} with {:?}", &args[0], &args[1..]);
        }
    }

    // Continued program logic goes here...
}
