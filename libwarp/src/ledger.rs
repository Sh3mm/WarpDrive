use std::collections::{HashMap, HashSet};
use std::fs::{File, create_dir_all};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use std::path::Path;
use crate::action::{Action, ActionType};

#[derive(Deserialize, Serialize)]
pub struct Ledger {
    pub path_map: HashMap<String, LedgerInfo>
}

#[derive(Deserialize, Serialize)]
pub struct LedgerInfo {
    #[serde(with = "time::serde::rfc3339")]
    pub last_update: OffsetDateTime
}

impl Ledger {
    pub fn new() -> Self {
        return Ledger{
            path_map: Default::default()
        };
    }

    pub fn ledger_from(actions: &Vec<Action>) -> Self {
        let update_time = OffsetDateTime::now_utc();
        let path_map: HashMap<String, LedgerInfo> = HashMap::from_iter(actions.iter().filter_map(|action|{
            return match action.action {
                ActionType::DelLocal => { None }
                ActionType::DelRemote => { None }
                ActionType::Local2Remote => { Some((action.path.clone(), LedgerInfo{last_update: update_time})) }
                ActionType::Remote2Local => { Some((action.path.clone(), LedgerInfo{last_update: update_time})) }
                _ => { Some((action.path.clone(), LedgerInfo{last_update: update_time})) }
            }
        }));

        return Self{path_map}
    }

    pub fn load(link_path: &str) -> Self {
        let path = Path::new(link_path).join("ledger.json");

        let ledger: Ledger =  serde_json::from_reader(
            File::open(&path).unwrap()
        ).expect("Error while reading ledger");

        return ledger
    }

    pub fn save(&self, link_path: &str) {
        create_dir_all(link_path).expect("Unable to create configs/link folder");
        let path = Path::new(link_path).join("ledger.json");

        serde_json::to_writer(
            File::create(path).unwrap(),
            &self
        ).expect("Error while saving ledger");
    }

    pub fn update_ledger(&mut self, file: &str, action: ActionType) {
        match action {
            ActionType::DelLocal | ActionType::DelRemote => {
                self.path_map.remove(file);
            }
            ActionType::Local2Remote | ActionType::Remote2Local => {
                self.path_map.insert(file.to_owned(), LedgerInfo{last_update: OffsetDateTime::now_utc()});
            }
            _ => panic!("unexpected actionType")
        }
    }
}