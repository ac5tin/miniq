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

    pub fn get_chan(
        &self,
        chan_name: &str,
    ) -> Result<&Receiver<Task>, Box<dyn std::error::Error + Send + Sync>> {
        match self.ch.get(chan_name) {
            Some(ch) => Ok(&ch.chan.1),
            None => Err("".into()),
        }
    }
}

// singleton Queue
pub static Q: SyncLazy<Mutex<Queue>> = SyncLazy::new(|| Mutex::new(Queue::new()));

// test
#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::Queue;

    #[tokio::test]
    async fn add_task() {
        {
            // receive task channel
            let q = Arc::new(Mutex::new(Queue::new()));

            let q1 = Arc::clone(&q);

            // print received Tasks
            let j = tokio::task::spawn(async move {
                let qq = q1.lock().await;
                let rcv = qq.get_chan("test_chan").unwrap();
                // infinite loop
                for t in rcv.iter() {
                    println!("received task with id: {}", t.id);
                    break; // break loop after receiving 1 task
                }
                return;
            });
            {
                // add task
                let data = "test123abc".as_bytes().to_vec();
                let q = Arc::clone(&q);
                let mut qq = q.lock().await;
                println!("Sending new task");
                assert_eq!(qq.add_task("test_chan", data).is_err(), false);
            }
            j.await.unwrap();
        }
    }
}
