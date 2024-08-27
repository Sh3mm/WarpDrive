use clap::Args;
use warp::configs::Config;
use std::fs::remove_dir_all;
use crate::cmds::Cmd;

#[derive(Args)]
pub struct CmdDelete {
    /// Name of the config to delete
    name: String,

    /// Use With Caution: Removes the local folder and all files in it
    #[arg(long, action=clap::ArgAction::SetTrue)]
    clean: bool
}


impl Cmd for CmdDelete {
    fn execute(&self) {
        let config = Config::load(&self.name);

        if config.is_err() { println!("Invalid name: '{}'", &self.name); return;}
        let config = config.unwrap();

        remove_dir_all(config.link_path).unwrap();
        if self.clean { remove_dir_all(config.local).unwrap(); }
    }
}
