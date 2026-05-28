use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use parking_lot::RwLock;

use crate::error::{BrokerError, BrokerResult};

pub struct CursorState {
    pub client_id: String,
    pub topic_name: String,
    pub committed_offset: u64,
    pub fetch_offset: u64,
    pub last_activity_secs: u64,
}

pub struct CursorManager {
    dir: PathBuf,
    cursors: RwLock<HashMap<String, CursorState>>,
}

impl CursorManager {
    pub fn new(data_dir: &Path) -> BrokerResult<Self> {
        let dir = data_dir.join("cursors");
        fs::create_dir_all(&dir).map_err(BrokerError::from)?;

        let mut manager = Self {
            dir,
            cursors: RwLock::new(HashMap::new()),
        };

        manager.load_all()?;
        Ok(manager)
    }

    fn cursor_path(&self, client_id: &str) -> PathBuf {
        self.dir.join(format!("{}.json", client_id))
    }

    pub fn get_or_create(&self, client_id: &str, topic_name: &str) -> CursorState {
        let mut cursors = self.cursors.write();
        if let Some(state) = cursors.get(client_id) {
            return CursorState {
                client_id: state.client_id.clone(),
                topic_name: state.topic_name.clone(),
                committed_offset: state.committed_offset,
                fetch_offset: state.fetch_offset,
                last_activity_secs: state.last_activity_secs,
            };
        }

        let state = CursorState {
            client_id: client_id.to_string(),
            topic_name: topic_name.to_string(),
            committed_offset: 0,
            fetch_offset: 0,
            last_activity_secs: 0,
        };

        cursors.insert(client_id.to_string(), state);
        CursorState {
            client_id: client_id.to_string(),
            topic_name: topic_name.to_string(),
            committed_offset: 0,
            fetch_offset: 0,
            last_activity_secs: 0,
        }
    }

    pub fn update_fetch_offset(&self, client_id: &str, new_offset: u64) -> BrokerResult<()> {
        let mut cursors = self.cursors.write();
        let cursor = cursors
            .get_mut(client_id)
            .ok_or_else(|| BrokerError::NotFound(format!("cursor not found: {}", client_id)))?;

        cursor.fetch_offset = new_offset;
        Ok(())
    }

    pub fn commit_offset(&self, client_id: &str, offset: u64) -> BrokerResult<()> {
        let mut cursors = self.cursors.write();
        let cursor = cursors
            .get_mut(client_id)
            .ok_or_else(|| BrokerError::NotFound(format!("cursor not found: {}", client_id)))?;

        cursor.committed_offset = offset;
        self.save_one(client_id, cursor)?;
        Ok(())
    }

    fn save_one(&self, client_id: &str, state: &CursorState) -> BrokerResult<()> {
        let path = self.cursor_path(client_id);
        let json = format!(
            r#"{{"client_id":"{}","topic_name":"{}","committed_offset":{},"fetch_offset":{},"last_activity_secs":{}}}"#,
            state.client_id,
            state.topic_name,
            state.committed_offset,
            state.fetch_offset,
            state.last_activity_secs,
        );

        let mut file = fs::File::create(&path).map_err(BrokerError::from)?;
        file.write_all(json.as_bytes()).map_err(BrokerError::from)?;
        file.flush().map_err(BrokerError::from)?;

        Ok(())
    }

    pub fn save_all(&self) -> BrokerResult<()> {
        let cursors = self.cursors.read();
        for (client_id, state) in cursors.iter() {
            self.save_one(client_id, state)?;
        }
        Ok(())
    }

    fn load_all(&mut self) -> BrokerResult<()> {
        if !self.dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.dir).map_err(BrokerError::from)?;

        for entry in entries {
            let entry = entry.map_err(BrokerError::from)?;
            let path = entry.path();

            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }

            let mut file = fs::File::open(&path).map_err(BrokerError::from)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(BrokerError::from)?;

            if let Some(state) = Self::parse_cursor_json(&contents) {
                let mut cursors = self.cursors.write();
                cursors.insert(state.client_id.clone(), state);
            }
        }

        Ok(())
    }

    fn parse_cursor_json(json: &str) -> Option<CursorState> {
        let extract_u64 = |key: &str| -> Option<u64> {
            let start = json.find(key)? + key.len() + 2;
            let rest = &json[start..];
            let end = rest.find(['}', ',']).unwrap_or(rest.len());
            rest[..end].parse::<u64>().ok()
        };

        let extract_string = |key: &str| -> Option<String> {
            let start = json.find(key)? + key.len() + 3;
            let rest = &json[start..];
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        };

        let client_id = extract_string(r#""client_id""#)?;
        let topic_name = extract_string(r#""topic_name""#)?;
        let committed_offset = extract_u64(r#""committed_offset""#)?;
        let fetch_offset = extract_u64(r#""fetch_offset""#)?;
        let last_activity_secs = extract_u64(r#""last_activity_secs""#)?;

        Some(CursorState {
            client_id,
            topic_name,
            committed_offset,
            fetch_offset,
            last_activity_secs,
        })
    }
}
