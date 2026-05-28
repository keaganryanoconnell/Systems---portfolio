use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroTask {
    pub id: u64,
    pub name: String,
    pub payload_type: String,
    pub data_range: RangeSpec,
    pub partition_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeSpec {
    pub start: u64,
    pub end: u64,
}

impl RangeSpec {
    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTask {
    pub id: u64,
    pub macro_id: u64,
    pub partition: u32,
    pub range: RangeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: u64,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task: MicroTask,
    pub state: TaskState,
    pub assigned_node: Option<u32>,
    pub retry_count: u32,
    pub max_retries: u32,
}

pub fn split_workload(macro_task: &MacroTask) -> Vec<MicroTask> {
    let range_size = macro_task.data_range.size();
    let partition_size = range_size / macro_task.partition_count as u64;

    let mut tasks = Vec::with_capacity(macro_task.partition_count as usize);

    for p in 0..macro_task.partition_count {
        let start = macro_task.data_range.start + (p as u64 * partition_size);
        let end = if p == macro_task.partition_count - 1 {
            macro_task.data_range.end
        } else {
            start + partition_size
        };

        tasks.push(MicroTask {
            id: (macro_task.id << 16) | (p as u64),
            macro_id: macro_task.id,
            partition: p,
            range: RangeSpec { start, end },
        });
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_even_partitions() {
        let mt = MacroTask {
            id: 1,
            name: "test".into(),
            payload_type: "prime".into(),
            data_range: RangeSpec {
                start: 0,
                end: 1000,
            },
            partition_count: 10,
        };

        let micros = split_workload(&mt);
        assert_eq!(micros.len(), 10);
        assert_eq!(micros[0].range.start, 0);
        assert_eq!(micros[0].range.end, 100);
        assert_eq!(micros[9].range.start, 900);
        assert_eq!(micros[9].range.end, 1000);
    }
}
