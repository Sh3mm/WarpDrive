use std::{io, io::{Read, Write}};
use clap::Args;
use libwarp::{
    action::{ErrorType, ActionType, Action, gen_action_list},
    rclone::RClone, ledger::Ledger, configs::Config
};

use crate::cmds::Cmd;

fn handle_errors(actions: &mut Vec<Action>) {
    let errors = actions.iter_mut().filter(|a| matches!(a.action, ActionType::Error(_)));
    for error in errors {
        println!("{} in file: {}", error.action,  error.path);
        loop {
            print!("\nkeep the REMOTE or LOCAL? (r/l): ");
            io::stdout().flush().expect("");

            let mut buffer: [u8; 1] = [0];
            let res = io::stdin().read(&mut buffer);
            if res.is_err() { continue; }

            let input = buffer[0];
            println!("{}", input);
            match input {
                0x6C => {
                    // if the local action is chosen, and it was a deletion, the deletion needs to
                    // be propagated to the remote
                    error.action = if matches!(error.action, ActionType::Error(ErrorType::DelAndMod)) {
                        ActionType::DelRemote
                    } else { // otherwise, the local has a file and it needs to be copied to the remote
                        ActionType::Local2Remote
                    };
                    break;
                }
                0x72 => {
                    // if the remote action is chosen, and it was a deletion, the deletion needs to
                    // be propagated to the local
                    error.action = if matches!(error.action, ActionType::Error(ErrorType::ModAndDel)) {
                        ActionType::DelLocal
                    } else { // otherwise, the local has a file and it needs to be copied to the remote
                        ActionType::Remote2Local
                    };
                    break;
                }
                _ => { continue; }
            }
        }
    }
}


#[derive(Args)]
pub struct CmdSync {
    /// Name of the link to synchronize
    name: String,

    /// runs the steps in parallel
    #[arg(short, long, action=clap::ArgAction::SetFalse)]
    parallel: bool
}


impl Cmd for CmdSync {
    fn execute(&self) {
        let config = Config::load(&self.name);
        let ledger = Ledger::load(&config.link_path);

        let rclone = RClone::new(&config.local, &config.remote);

        let local =  rclone.local_list();
        let remote = rclone.remote_list();

        let mut actions = gen_action_list(&local, &remote, &ledger);
        println!("{:?}", actions);

        handle_errors(&mut actions);

        match self.parallel {
            true => { rclone.apply_actions_par(&actions); }
            false => { rclone.apply_actions(&actions); }
        }

        let new_ledger = ledger.updated_ledger(&actions);
        new_ledger.save(&config.link_path)
    }
}

impl CmdSync {
    pub fn new(name: &str) -> Self {
        Self{ name: name.to_string(), parallel: false}
    }
}