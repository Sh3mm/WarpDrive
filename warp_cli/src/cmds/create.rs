use clap::Args;
use crate::cmds::Cmd;
use crate::cmds::CmdSync;

use warp::{ledger::Ledger, configs::Config};

#[derive(Args)]
pub struct CmdCreate {
    /// Name of the config to create
    name: String,
    /// remote rclone path for the config (ex: remote:/location/)
    remote: String,
    /// local path for the link config (ex: ~/location/). The default value is the curent folder
    #[arg(short, long, default_value="./")]
    local: String,
    /// If not set, the creation will also automatically sync
    #[arg(short, long, action=clap::ArgAction::SetTrue)]
    no_sync: bool
}


impl Cmd for CmdCreate {
    fn execute(&self) {
        let names = Config::get_all_names();
        if names.contains(&self.name) {
            panic!("Name {} already exists", &self.name)
        }

        let configs = Config::new(&self.name, &self.local, &self.remote, 0);
        let ledger = Ledger::new();

        ledger.save(&configs.link_path);
        configs.save();

        if !self.no_sync { CmdSync::new(&self.name).execute(); }
    }
}