use chrono::{DateTime, Utc};

// Task Status
#[derive(Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Delete,
}

impl Into<i32> for TaskStatus {
    fn into(self) -> i32 {
        match self {
            TaskStatus::Pending => 0,
            TaskStatus::Running => 1,
            TaskStatus::Completed => 2,
            TaskStatus::Failed => 3,
            TaskStatus::Delete => 4,
        }
    }
}

impl From<i32> for TaskStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => TaskStatus::Pending,
            1 => TaskStatus::Running,
            2 => TaskStatus::Completed,
            3 => TaskStatus::Failed,
            4 => TaskStatus::Delete,
            _ => TaskStatus::Pending,
        }
    }
}

// Task
#[derive(Clone)]
pub struct Task {
    pub id: String,
    pub data: Vec<u8>,
    pub creation_date: DateTime<Utc>,
    pub status: TaskStatus,
    pub channel: String,
}

impl Task {
    pub fn new(data: Vec<u8>, channel: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: data,
            creation_date: Utc::now(),
            status: TaskStatus::Pending,
            channel: channel,
        }
    }
}
