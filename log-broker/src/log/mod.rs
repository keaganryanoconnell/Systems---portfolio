pub mod index;
pub mod segment;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use index::SegmentIndex;
use parking_lot::Mutex;
use segment::{MessageHeader, SegmentConfig, SegmentFile};

use crate::error::{BrokerError, BrokerResult};

pub struct TopicLog {
    pub topic_id: u32,
    pub topic_name: String,
    dir: PathBuf,
    segments: Vec<SegmentFile>,
    index: SegmentIndex,
    active_segment_idx: usize,
    config: SegmentConfig,
}

impl TopicLog {
    pub fn create(data_dir: &Path, topic_name: &str, config: SegmentConfig) -> BrokerResult<Self> {
        let topic_id = hash_topic_name(topic_name);
        let topic_hex = format!("{:08x}", topic_id);
        let dir = data_dir.join("topics").join(&topic_hex);

        std::fs::create_dir_all(&dir).map_err(BrokerError::from)?;

        let mut segments = Vec::new();
        let segment = SegmentFile::create(&dir, 0, config.clone())?;
        let base_offset = segment.base_offset;
        segments.push(segment);

        let index = SegmentIndex::new(base_offset);

        Ok(Self {
            topic_id,
            topic_name: topic_name.to_string(),
            dir,
            segments,
            index,
            active_segment_idx: 0,
            config,
        })
    }

    pub fn open(data_dir: &Path, topic_name: &str, config: SegmentConfig) -> BrokerResult<Self> {
        let topic_id = hash_topic_name(topic_name);
        let topic_hex = format!("{:08x}", topic_id);
        let dir = data_dir.join("topics").join(&topic_hex);

        if !dir.exists() {
            return Err(BrokerError::NotFound(format!(
                "topic dir not found: {:?}",
                dir
            )));
        }

        let mut existing_segments = Self::discover_segments(&dir)?;
        if existing_segments.is_empty() {
            let segment = SegmentFile::create(&dir, 0, config.clone())?;
            existing_segments.push((0, segment));
        }

        existing_segments.sort_by_key(|(base, _)| *base);

        let mut segments = Vec::new();
        let mut index = SegmentIndex::new(existing_segments[0].0);
        let mut last_offset = existing_segments[0].0;

        for (base_offset, mut seg) in existing_segments {
            let entries = SegmentFile::scan_for_rebuild(&dir, base_offset)?;
            for (hdr, pos) in &entries {
                index.add(hdr.offset, *pos, hdr.key_len, hdr.value_len);
                last_offset = hdr.offset + 1;
            }

            seg.message_count = entries.len() as u64;

            segments.push(seg);
        }

        let active_idx = segments
            .iter()
            .position(|s| !s.sealed)
            .map_or_else(
                || {
                    let new_base = last_offset;
                    let new_seg = SegmentFile::create(&dir, new_base, config.clone()).ok()?;
                    segments.push(new_seg);
                    Some(segments.len() - 1)
                },
                Some,
            )
            .ok_or_else(|| {
                BrokerError::InvalidArgument("could not determine active segment".into())
            })?;

        Ok(Self {
            topic_id,
            topic_name: topic_name.to_string(),
            dir,
            segments,
            index,
            active_segment_idx: active_idx,
            config,
        })
    }

    fn discover_segments(dir: &Path) -> BrokerResult<Vec<(u64, SegmentFile)>> {
        let mut segments = Vec::new();

        if !dir.exists() {
            return Ok(segments);
        }

        let entries = std::fs::read_dir(dir).map_err(BrokerError::from)?;

        for entry in entries {
            let entry = entry.map_err(BrokerError::from)?;
            let path = entry.path();
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !fname.ends_with(".log") {
                continue;
            }

            let base_str = fname.trim_end_matches(".log");
            if let Ok(base) = base_str.parse::<u64>() {
                let seg = SegmentFile::open(dir, base, SegmentConfig::default())?;
                segments.push((base, seg));
            }
        }

        Ok(segments)
    }

    pub fn append(&mut self, key: &[u8], value: &[u8]) -> BrokerResult<u64> {
        let offset = self.index.next_offset();

        if self.segments[self.active_segment_idx].sealed {
            let new_segment = SegmentFile::create(&self.dir, offset, self.config.clone())?;
            self.segments.push(new_segment);
            self.active_segment_idx = self.segments.len() - 1;
        }

        let pos = self.segments[self.active_segment_idx].append(offset, key, value)?;
        self.index
            .add(offset, pos, key.len() as u32, value.len() as u32);

        Ok(offset)
    }

    pub fn fetch(&self, offset: u64) -> BrokerResult<(MessageHeader, Vec<u8>, Vec<u8>)> {
        let pos = self.index.find_position(offset)?;

        for seg in &self.segments {
            let seg_end = seg.base_offset + seg.message_count;
            if offset >= seg.base_offset && offset < seg_end {
                return seg.read_message_at(pos);
            }
            if seg.base_offset <= offset && seg.sealed && seg_end == seg.base_offset {
                continue;
            }
        }

        let active = &self.segments[self.active_segment_idx];
        active.read_message_at(pos)
    }

    pub fn earliest_offset(&self) -> Option<u64> {
        self.index.earliest_offset()
    }

    pub fn latest_offset(&self) -> Option<u64> {
        self.index.latest_offset()
    }

    pub fn next_offset(&self) -> u64 {
        self.index.next_offset()
    }
}

pub struct LogManager {
    pub topics: Mutex<HashMap<u32, TopicLog>>,
    data_dir: PathBuf,
    config: SegmentConfig,
}

impl LogManager {
    pub fn new(data_dir: &Path, config: SegmentConfig) -> Self {
        Self {
            topics: Mutex::new(HashMap::new()),
            data_dir: data_dir.to_path_buf(),
            config,
        }
    }

    pub fn get_or_create_topic(&self, topic_name: &str) -> BrokerResult<u32> {
        let topic_id = hash_topic_name(topic_name);
        let mut topics = self.topics.lock();

        if topics.contains_key(&topic_id) {
            return Ok(topic_id);
        }

        let topic_log = TopicLog::create(&self.data_dir, topic_name, self.config.clone())?;
        topics.insert(topic_id, topic_log);

        Ok(topic_id)
    }

    pub fn topic_exists(&self, topic_name: &str) -> bool {
        let topic_id = hash_topic_name(topic_name);
        let topics = self.topics.lock();
        topics.contains_key(&topic_id)
    }

    pub fn append(&self, topic_name: &str, key: &[u8], value: &[u8]) -> BrokerResult<u64> {
        let topic_id = hash_topic_name(topic_name);
        let mut topics = self.topics.lock();

        let topic = if let Some(t) = topics.get_mut(&topic_id) {
            t
        } else {
            let new_topic = TopicLog::create(&self.data_dir, topic_name, self.config.clone())?;
            topics.insert(topic_id, new_topic);
            topics
                .get_mut(&topic_id)
                .ok_or(BrokerError::TopicNotFound(topic_id))?
        };

        topic.append(key, value)
    }

    pub fn fetch(
        &self,
        topic_name: &str,
        offset: u64,
    ) -> BrokerResult<(MessageHeader, Vec<u8>, Vec<u8>)> {
        let topic_id = hash_topic_name(topic_name);
        let topics = self.topics.lock();

        let topic = topics
            .get(&topic_id)
            .ok_or(BrokerError::TopicNotFound(topic_id))?;

        topic.fetch(offset)
    }

    pub fn get_topic_offsets(&self, topic_name: &str) -> BrokerResult<(Option<u64>, u64)> {
        let topic_id = hash_topic_name(topic_name);
        let topics = self.topics.lock();

        let topic = topics
            .get(&topic_id)
            .ok_or(BrokerError::TopicNotFound(topic_id))?;

        let earliest = topic.earliest_offset();
        let latest = topic.next_offset();

        Ok((earliest, latest))
    }
}

pub fn hash_topic_name(name: &str) -> u32 {
    let mut hash: u32 = 0x811C_9DC5;
    for byte in name.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}
