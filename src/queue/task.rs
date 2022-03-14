use chrono::{DateTime, Utc};

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub(crate) struct Task<'a> {
    pub id: String,
    pub data: &'a Vec<u8>,
    pub creation_date: DateTime<Utc>,
    pub status: TaskStatus,
}

impl<'a> Task<'a> {
    pub fn new(data: &'a Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: data,
            creation_date: Utc::now(),
            status: TaskStatus::Pending,
        }
    }
}
