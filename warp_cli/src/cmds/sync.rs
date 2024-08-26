use std::{io, io::{Read, Write}};
use std::io::stdout;
use clap::Args;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use termion::{clear, cursor, color};
use warp::{
    action::{ErrorType, ActionType, Action, gen_action_list},
    rclone::{RClone, RFileInfo}, ledger::Ledger, configs::Config
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

        if config.is_err() { println!("Invalid name: '{}'", &self.name); return;}
        let config = config.unwrap();

        let ledger = Ledger::load(&config.link_path);

        let rclone = RClone::new(&config.local, &config.remote);

        let local =  rclone.local_list();

        let _rclone = rclone.clone();
        let remote_future = thread::spawn(move || _rclone.remote_list());
        Self::wait_for_remote(&remote_future);

        let remote = remote_future.join().unwrap();

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

        // getting the total number of steps to take
        let total : usize = 2 * actions.iter().filter(|a| a.action != ActionType::Nothing).count();
        let mut steps: usize = 0;
        for (state, file) in rx {
            steps += 1;
            Self::update_cli(state, file, steps, total);
            if steps == total { break; }
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

    fn update_cli(state: bool, name: String, done: usize, total: usize) {
        let (c, r) = termion::terminal_size().unwrap();

        let prefix = match state { true => "starting", false => "finished" };
        println!("{}{}{} {}", cursor::Goto(1, r), clear::CurrentLine, prefix, name);

        let percent_space = usize::from(c - 17);
        let prc_progress = (done * percent_space) / total;
        let percent = (done * 100) / total;

        print!(
            "{}{}{} Progress:{: >3}%{}{} [{: <percent_space$}]",
            cursor::Goto(1, r),
            color::Fg(color::Black), color::Bg(color::Green),
            percent,
            color::Fg(color::Reset), color::Bg(color::Reset),
            format!("{:#>prc_progress$}", "")
        );
        stdout().flush().unwrap();
    }

    fn wait_for_remote(remote_future: &JoinHandle<Vec<RFileInfo>>) {
        let mut state = "|";

        print!("getting remote file list. This may take a while... |");
        while !remote_future.is_finished() {
            state = match state { "|" => "/", "/" => "-",  "-" => "\\",  "\\" => "|",  _ => "|" };
            print!("{}{}", cursor::Left(1), state);
            stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(750))
        }
    }
}