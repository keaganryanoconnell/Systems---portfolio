use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Delete { key: String },
}

#[derive(Default, Debug, Clone)]
pub struct KeyValueStore {
    state: HashMap<String, String>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub fn apply(&mut self, command: &Command) -> Option<String> {
        match command {
            Command::Set { key, value } => self.state.insert(key.clone(), value.clone()),
            Command::Delete { key } => self.state.remove(key),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.state.get(key)
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        self.state.clone()
    }
}
