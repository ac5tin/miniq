use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, lazy::SyncLazy, sync};

use self::task::Task;

mod task;

struct QueueChan<'a> {
    id: String,
    chan: (Sender<Task<'a>>, Receiver<Task<'a>>),
    status: task::TaskStatus,
}

pub struct Queue<'a> {
    tasks: sync::RwLock<Vec<task::Task<'a>>>,
    ch: HashMap<String, QueueChan<'a>>,
}

impl<'a> Queue<'a> {
    pub fn new() -> Self {
        Queue {
            tasks: sync::RwLock::new(Vec::new()),
            ch: HashMap::new(),
        }
    }

    // Add new task to queue
    pub fn add_task(
        &mut self,
        chan_name: &str,
        data: &'a Vec<u8>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let task = task::Task::new(data);
        // add task to queue
        let x = match self.tasks.get_mut() {
            Ok::<&mut Vec<Task<'a>>, _>(tasks) => {
                let task_id = task.id.clone().to_owned();
                tasks.push(task.clone());
                task_id
            }
            Err(_) => return Err("Failed to add task".into()),
        };
        // send task through channels
        let chan = self
            .ch
            .entry(chan_name.to_owned())
            .or_insert_with(|| QueueChan {
                id: chan_name.to_owned(),
                chan: crossbeam_channel::unbounded(),
                status: task::TaskStatus::Pending,
            });
        match chan.chan.0.send(task.clone()) {
            Ok(_) => {}
            Err(err) => return Err(format!("Failed to send task to channel|Err: {}", err).into()),
        };

        // success
        Ok(x)
    }
}

// singleton Queue
pub static Q: SyncLazy<Queue> = SyncLazy::new(|| Queue::new());
