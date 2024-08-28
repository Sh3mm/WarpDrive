use std::collections::HashMap;
use serde::Deserialize;
use serde_json::json;
use rayon::prelude::*;
use time::OffsetDateTime;
use std::sync::mpsc::{Sender};
use crate::action::{Action, ActionType};


#[derive(Deserialize)]
pub struct RListResult {
    pub list: Vec<RFileInfo>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RFileInfo {
    #[serde(rename = "Path")]
    pub path: String,
    //#[serde(rename = "Name")]
    //pub name: String,
    #[serde(rename = "Size")]
    pub size: i64,
    // pub MimeType: String,
    #[serde(rename = "ModTime", with = "time::serde::rfc3339")]
    pub mod_time: OffsetDateTime,
    #[serde(rename = "IsDir")]
    pub is_dir: bool,
    // pub ID: String,
}


#[derive(Clone)]
pub struct RClone {
    local: String,
    remote: String
}


impl RClone {
    pub fn new(local: &str, remote: &str) -> Self {
        librclone::initialize();
        return RClone{
            local: local.to_string(),
            remote: remote.to_string()
        }
    }



    fn sort_actions(actions: &Vec<Action>) -> HashMap<ActionType, Vec<String>> {
        let mut map: HashMap<ActionType, Vec<String>> = HashMap::new();
        for action in actions {
            match map.get_mut(&action.action) {
                None => { map.insert(action.action.clone(), vec![action.path.clone()]); }
                Some(list) => { list.push(action.path.clone()); }
            }
        }
        return map;
    }

    fn batch_actions(actions: HashMap<ActionType, Vec<String>>, size: Option<usize>) -> HashMap<(ActionType, usize), Vec<String>> {
        let mut new_map = HashMap::new();
        actions.iter().for_each(|(k, v)|{
            match size {
                None => { new_map.insert((k.clone(), 0), v.clone());  }
                Some(i) => {
                    v.chunks(i).enumerate().for_each(|(i, v)|{
                        new_map.insert((k.clone(), i), Vec::from(v));
                    });
                }
            }
        });

        return new_map;
    }

    pub fn apply_actions(&self, actions: &Vec<Action>, pipe: Option<Sender<(bool, String)>>, thread_nb: usize, batch_size: Option<usize>) {
        rayon::ThreadPoolBuilder::new().num_threads(thread_nb).build_global().unwrap();

        let actions = Self::sort_actions(actions);
        let actions = Self::batch_actions(actions, batch_size);
        let lazy_result = actions.par_iter().map(
            |((a, _), list)| self.execute(a, list, &pipe)
        );

        lazy_result.reduce(|| Ok("".to_string()), |acc, r| {
            if r.is_ok() && acc.is_ok() { return acc; }
            if r.is_ok() && acc.is_err() { return acc; }
            if r.is_err() && acc.is_ok() { return Err(r.err().unwrap()); }
            else { Err(format!("{}\n{}", r.err().unwrap(), acc.err().unwrap()) ) }
        }).unwrap();
    }

    pub fn local_list(&self) -> Vec<RFileInfo> { Self::get_file_list(&self.local) }

    pub fn remote_list(&self) -> Vec<RFileInfo> { Self::get_file_list(&self.remote) }

    fn execute(&self, a: &ActionType, list: &Vec<String>, pipe: &Option<Sender<(bool, String)>>) -> Result<String, String> {
        if a == &ActionType::Nothing { return Ok("Noting to do".to_string()); }

        // sending to pipe starting signal for files
        match &pipe {
            None => {}
            Some(tx) => {
                list.iter().for_each( |s| tx.send((true, s.clone())).expect("Error while sending update") )
            }
        }

        // doing necessary action
        let res = match a {
            ActionType::DelLocal =>     { RClone::delete_files(&self.local, list) }
            ActionType::DelRemote =>    { RClone::delete_files(&self.remote, list) }
            ActionType::Local2Remote => { RClone::copy_files(&self.local, &self.remote, list) }
            ActionType::Remote2Local => { RClone::copy_files(&self.remote, &self.local, list) }
            _ => { Err(format!("An unexpected ActionType found during resolution ({a}). ").to_string()) }
        };

        // sending to pipe ending signal for files
        match &pipe {
            None => {}
            Some(tx) => {
                list.iter().for_each( |s| tx.send((false, s.clone())).expect("Error while sending update") )
            }
        }

        return res;
    }

    fn get_file_list(fs: &str) -> Vec<RFileInfo> {
        let res = librclone::rpc("operations/list",
            json!({
                "fs": fs, "remote": "",
                "opt": { "recurse": true },
                "_config": {"fastList": true}
            }).to_string()
        );

        let res: RListResult = serde_json::from_str(&res.unwrap()).unwrap();
        return res.list;
    }

    fn copy_files(from: &str, to: &str, files: &Vec<String>) -> Result<String, String> {
        librclone::rpc("sync/copy",
            json!({
                "srcFs": from, "dstFs": to,
                "_filter": { "IncludeRule": files },
                "_config": {"NoCheckDest": true}
            }).to_string()
        )
    }

    fn delete_files(from: &str, files: &Vec<String>) -> Result<String, String> {
        librclone::rpc("operations/delete",
            json!({
                "fs": from,
                "_filter": { "IncludeRule": files }
            }).to_string()
        )
    }
}
