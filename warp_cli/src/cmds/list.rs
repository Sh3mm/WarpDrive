use clap::Args;
use warp::configs::{Config};

use crate::cmds::Cmd;

#[derive(Args)]
pub struct CmdList {}


impl Cmd for CmdList {
    fn execute(&self) {

        println!("{:12}| {}", "Name", "Paths");
        let names = Config::get_all_names();
        for name in names{
            let config = Config::load(&name);
            if config.is_err() {
                println!("Error while trying to open configs for: '{name}'");
                return;
            }

            let config = config.unwrap();
            println!("{:-<27}", "");
            println!("{:12}| {}", &name, &config.local);
            println!("{:12}| {}", "", &config.remote);
        }
    }
}
