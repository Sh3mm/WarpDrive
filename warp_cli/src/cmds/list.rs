use clap::Args;
use libwarp::configs::{Config};

use crate::cmds::Cmd;

#[derive(Args)]
pub struct CmdList {}


impl Cmd for CmdList {
    fn execute(&self) {

        println!("{:12}| {}", "Name", "Paths");
        let names = Config::get_all_names();
        for name in names{
            let config = Config::load(&name);

            println!("{:-<27}", "");
            println!("{:12}| {}", &name, &config.local);
            println!("{:12}| {}", "", &config.remote);
        }
    }
}
