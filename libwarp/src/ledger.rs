use std::collections::HashSet;
use std::fs::{File, create_dir_all};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use std::path::Path;
use crate::action::{Action, ActionType};

#[derive(Deserialize, Serialize)]
pub struct Ledger {
    #[serde(with = "time::serde::rfc3339")]
    pub update_time: OffsetDateTime,
    pub path_set: HashSet<String>
}

impl Ledger {
    pub fn new() -> Self {
        return Ledger{
            update_time: OffsetDateTime::now_utc().replace_year(0).unwrap(),
            path_set: Default::default()
        };
    }

    pub fn load(link_path: &str) -> Self {
        let path = Path::new(link_path).join("ledger.json");

        let ledger: Ledger =  serde_json::from_reader(
            File::open(&path).unwrap()
        ).expect("Error while reading ledger");

        return ledger
    }

    pub fn save(self, link_path: &str) {
        create_dir_all(link_path).expect("Unable to create configs/link folder");
        let path = Path::new(link_path).join("ledger.json");

        serde_json::to_writer(
            File::create(path).unwrap(),
            &self
        ).expect("Error while saving ledger");
    }

    pub fn updated_ledger(&self, actions: &Vec<Action>) -> Self {
        let path_set = HashSet::from_iter(actions.iter().filter_map(|action|{
            return match action.action {
                ActionType::DelLocal => { None }
                ActionType::DelRemote => { None }
                ActionType::Local2Remote => { Some(action.path.clone()) }
                ActionType::Remote2Local => { Some(action.path.clone()) }
                _ => { Some(action.path.clone()) }
            }
        }));
        let update_time = OffsetDateTime::now_utc();

        return Self{path_set, update_time}
    }
}