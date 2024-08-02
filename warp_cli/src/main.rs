mod cmds;
use cmds::Cmd;

use clap::{Parser, Subcommand};
use cmds::{CmdCreate, CmdDelete, CmdList, CmdSync};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Warp {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a link between a local folder and a remote
    Create(CmdCreate),
    /// Deletes an existing link
    Delete(CmdDelete),
    /// Lists existing links
    List(CmdList),
    /// manually syncs a link
    Sync(CmdSync)
}

impl Commands {
    fn run(&self) {
        match &self {
            Commands::Create(d) => { d.execute() }
            Commands::Delete(d) => { d.execute() }
            Commands::List  (d) => { d.execute() }
            Commands::Sync  (d) => { d.execute() }
        }
    }
}

fn main() {
    let cli = Warp::parse();
    cli.command.run()
}