use core::fmt;
use std::collections::HashMap;
use time::OffsetDateTime;
use crate::rclone::{RFileInfo};
use crate::ledger::Ledger;

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ActionType {
    Nothing,
    Error(ErrorType),
    DelLocal,
    DelRemote,
    Local2Remote,
    Remote2Local,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionType::DelLocal =>     { write!(f, "Local Deletion") }
            ActionType::DelRemote =>    { write!(f, "Remote Deletion") }
            ActionType::Local2Remote => { write!(f, "copy Local -> Remote") }
            ActionType::Remote2Local => { write!(f, "copy Remote -> Local") }
            ActionType::Error(err) => {write!(f, "{}", err)}
            _ => unimplemented!()
        }
    }
}

impl ActionType {
    pub fn is_error(&self) -> bool {
        return match self { ActionType::Error(_) => true, _ => false }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ErrorType{
    TwoSideMod,
    ModAndDel,
    DelAndMod,
    TwoNew
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::TwoSideMod => { write!(f, "Conflicting modifications") }
            ErrorType::ModAndDel =>  { write!(f, "Modification & Deletion") }
            ErrorType::DelAndMod =>  { write!(f, "Deletion & Modification") }
            ErrorType::TwoNew =>     { write!(f, "Conflicting new files") }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    pub action: ActionType,
    pub path: String
}

impl Action {
    pub fn new(path: &String, action_type: ActionType) -> Self{
        return Action{
            path: path.clone(),
            action: action_type
        }
    }

    fn from(path: &String, local: &Option<OffsetDateTime>, remote: &Option<OffsetDateTime>, ledger: &Ledger) -> Self {
        let last_update = &ledger.update_time;
        match ledger.path_set.get(path) {
            // The ledger does not hold the file. This means it's new and should be added to the right destination
            // unless there are conflicting new files
            None => {
                // conflicting new files
                if local.is_some() && remote.is_some() {
                    return Self::new(path, ActionType::Error(ErrorType::TwoNew))
                }
                // new file in remote
                if local.is_none() && remote.is_some() {
                    return Self::new(path, ActionType::Remote2Local)
                }
                // new file in local
                if local.is_some() && remote.is_none() {
                    return Self::new(path, ActionType::Local2Remote)
                }

                else { panic!("Impossible scenario where a file exists but appears nowhere: \"{path}\"") }
            }
            // The ledger hold the file. This mean the action must be decided by comparing update times
            Some(_) => {
                // No local
                if local.is_none() {
                    let unwrap_remote = remote.unwrap();
                    // (no lo) + (re <= lu) -> DelRemote
                    if unwrap_remote.cmp(last_update).is_le() {
                        return Self::new(path, ActionType::DelRemote)
                    }
                    // (no lo) + (re > lu) -> Error
                    if unwrap_remote.cmp(last_update).is_gt() {
                        return Self::new(path, ActionType::Error(ErrorType::DelAndMod))
                    }
                }
                // remote is None
                if remote.is_none() {
                    let local_time = local.unwrap();
                    // (lo <= lu) + (no re) -> DelLocal
                    if local_time.cmp(last_update).is_le() {
                        return Self::new(path, ActionType::DelLocal)
                    }
                    // (re > lu) + (no lo) -> Error
                    if local_time.cmp(last_update).is_gt() {
                        return Self::new(path, ActionType::Error(ErrorType::ModAndDel))
                    }
                }
                let remote_time = remote.unwrap();
                let local_time = local.unwrap();

                // (re <= lu) + (lo > lu) -> Local2Remote
                if remote_time.cmp(last_update).is_le() & local_time.cmp(last_update).is_gt() {
                    return Self::new(path, ActionType::Local2Remote)
                }
                // (re > lu) + (lo <= lu) -> Local2Remote
                if remote_time.cmp(last_update).is_gt() & local_time.cmp(last_update).is_le() {

                    return Self::new(path, ActionType::Remote2Local)
                }
                // (re <= lu) + (lo <= lu) -> Nothing
                if remote_time.cmp(last_update).is_le() & local_time.cmp(last_update).is_le() {
                    return Self::new(path, ActionType::Nothing)
                }
                // (re > lu) + (lo > lu) -> Error
                if remote_time.cmp(last_update).is_gt() & local_time.cmp(last_update).is_gt() {
                    return Self::new(path, ActionType::Error(ErrorType::TwoSideMod))
                }
                else { panic!("impossible state reached") }
            }
        }
    }
}

pub fn gen_action_list(local: &Vec<RFileInfo>, remote: &Vec<RFileInfo>, ledger: &Ledger) -> Vec<Action> {
    let file_map = create_file_map(local, remote);
    return file_map.iter().map(
        |(p, times)| Action::from(p ,&times[0], &times[1], &ledger)
    ).collect::<Vec<Action>>()
}gen_action_list

fn create_file_map(l1: &Vec<RFileInfo>, l2: &Vec<RFileInfo>) -> HashMap<String, [Option<OffsetDateTime>; 2]> {
    let list_chain = Iterator::chain(
        l1.iter().map(|v| (0, v)),
        l2.iter().map(|v| (1, v))
    );

    let mut map = HashMap::new();
    for (i, v) in list_chain {
        // filters out folders
        if v.is_dir { continue; }

        let mut pair: [Option<OffsetDateTime>; 2] = map.get_mut(&v.path).unwrap_or(&mut [None, None]).clone();
        pair[i] = Some(v.mod_time);
        map.insert(v.path.clone(), pair);
    }
    return map;
}