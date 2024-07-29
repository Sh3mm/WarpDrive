use std::{io, io::{Read, Write}};
use std::io::stdout;
use clap::Args;
use std::sync::mpsc;
use std::thread;
use termion::{clear, cursor};
use libwarp::{
    action::{ErrorType, ActionType, Action, gen_action_list},
    rclone::{RClone}, ledger::Ledger, configs::Config
};

use crate::cmds::Cmd;

#[derive(Args)]
pub struct CmdSync {
    /// Name of the link to synchronize
    name: String,

    /// runs the steps in parallel
    #[arg(short, long, action=clap::ArgAction::SetTrue)]
    parallel: bool,

    /// defines the thread count if run in parallel mode. does nothing otherwise
    #[arg(short, long, default_value_t=4)]
    thread_count: usize,

    /// defines the number of element to put in a single rclone request if run in parallel mode. does nothing otherwise
    #[arg(short, long)]
    batch_size: Option<usize>
}


impl Cmd for CmdSync {
    fn execute(&self) {
        let (tx, rx) = mpsc::channel();
        let config = Config::load(&self.name);
        let ledger = Ledger::load(&config.link_path);

        let rclone = RClone::new(&config.local, &config.remote);

        let local =  rclone.local_list();
        let remote = rclone.remote_list();

        let mut actions = gen_action_list(&local, &remote, &ledger);

        Self::handle_errors(&mut actions);

        let _actions = actions.clone();
        let parallel = self.parallel.clone();
        let batch_size = self.batch_size.clone();
        let thread_count = self.thread_count.clone();

        let rclone = thread::spawn(move || {
            match parallel {
                true => { rclone.apply_actions_par(&_actions, Some(tx), thread_count, batch_size); }
                false => { rclone.apply_actions(&_actions, Some(tx)); }
            }
        });

        let mut done: usize = 0;
        let total : usize = 2 * actions.iter().filter(|a| a.action != ActionType::Nothing).count();
        for (state, file) in rx {
            done += 1;
            Self::print(state, file, done, total);
            if done == total { break; }
        }

        rclone.join().unwrap();
        
        let new_ledger = ledger.updated_ledger(&actions);
        new_ledger.save(&config.link_path)
    }

}

impl CmdSync {
    pub fn new(name: &str) -> Self {
        Self{ name: name.to_string(), parallel: false, thread_count: 4, batch_size: None}
    }

    fn handle_errors(actions: &mut Vec<Action>) {
        let errors = actions.iter_mut().filter(|a| matches!(a.action, ActionType::Error(_)));
        for error in errors {
            println!("\n{} in file: {}", error.action,  error.path);
            loop {
                print!("keep the REMOTE or LOCAL? (r/l): ");
                stdout().flush().expect("");

                let mut buffer: [u8; 1] = [0];
                let res = io::stdin().read(&mut buffer);
                if res.is_err() { continue; }

                let input = buffer[0];
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

    fn print(state: bool, name: String, done: usize, total: usize) {
        let (c, r) = termion::terminal_size().unwrap();

        let prefix = match state { true => "starting", false => "finished" };
        println!("{}{}{} {}", cursor::Goto(1, r), clear::CurrentLine, prefix, name);

        let percent_space = usize::from(c - 8);
        let prc_progress = (done * percent_space) / total;
        let percent = (done * 100) / total;

        print!("{}[{: <percent_space$}] {:0>3}% ", cursor::Goto(1, r), format!("{:#>prc_progress$}", ""), percent);
        stdout().flush().unwrap();
    }
}