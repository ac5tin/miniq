use crossbeam_channel::{Receiver, Sender};
use std::{
    collections::HashMap,
    lazy::SyncLazy,
    sync::{self, Mutex},
};

use self::task::Task;

mod task;

struct QueueChan {
    id: String,
    chan: (Sender<Task>, Receiver<Task>),
    status: task::TaskStatus,
}

pub struct Queue {
    tasks: sync::RwLock<Vec<task::Task>>,
    ch: HashMap<String, QueueChan>,
}

impl Queue {
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
        data: Vec<u8>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let task = task::Task::new(data);
        // add task to queue
        let x = match self.tasks.get_mut() {
            Ok(tasks) => {
                let task_id = task.id.clone();
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
        match chan.chan.0.send(task) {
            Ok(_) => {}
            Err(err) => return Err(format!("Failed to send task to channel|Err: {}", err).into()),
        };

        // success
        Ok(x)
    }
}

// singleton Queue
pub static Q: SyncLazy<Mutex<Queue>> = SyncLazy::new(|| Mutex::new(Queue::new()));

// test
#[cfg(test)]
mod tests {

    use super::Queue;

    #[test]
    fn add_task() {
        let mut q: Queue = Queue::new();
        let qq = &mut q;
        let data = "test123abc".as_bytes().to_vec(); // should live longer than q
        assert_eq!(qq.add_task("test_chan", data).is_err(), false);
    }
}
