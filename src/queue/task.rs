use chrono::{DateTime, Utc};

#[derive(Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone)]
pub(crate) struct Task {
    pub id: String,
    pub data: Vec<u8>,
    pub creation_date: DateTime<Utc>,
    pub status: TaskStatus,
}

impl Task {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: data,
            creation_date: Utc::now(),
            status: TaskStatus::Pending,
        }
    }
}
