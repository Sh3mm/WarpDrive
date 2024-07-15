use clap::Args;
use libwarp::configs::Config;
use std::fs::remove_dir_all;

use crate::cmds::Cmd;

#[derive(Args)]
pub struct CmdDelete {
    /// Name of the link to delete
    name: String,

    /// Use With Caution: Removes the local folder and all files in it
    #[arg(long, action=clap::ArgAction::SetFalse)]
    clean: bool
}


impl Cmd for CmdDelete {
    fn execute(&self) {
        let config = Config::load(&self.name);

        remove_dir_all(config.link_path).unwrap();

        if self.clean {
            remove_dir_all(config.local).unwrap();
        }
    }
}
