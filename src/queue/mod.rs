use self::task::{Task, TaskStatus};
use flume::{Receiver, Sender};
use std::{
    collections::HashMap,
    lazy::SyncLazy,
    sync::{self},
};
use tokio::sync::Mutex;

pub mod task;

pub struct Queue {
    tasks: sync::RwLock<Vec<task::Task>>,
    ch: HashMap<String, (Sender<Task>, Receiver<Task>)>, // channel_id -> sender
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            tasks: sync::RwLock::new(Vec::new()),
            ch: HashMap::new(),
        }
    }

    // Add new task to queue
    pub async fn add_task(
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
            .or_insert_with(|| flume::unbounded());

        match chan.0.send_async(task.to_owned()).await {
            Ok(_) => {}
            Err(err) => return Err(format!("Failed to send task to channel|Err: {}", err).into()),
        };

        // success
        Ok(x)
    }

    pub fn get_chan(
        &mut self,
        chan_name: &str,
    ) -> Result<Receiver<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let chan = self
            .ch
            .entry(chan_name.to_owned())
            .or_insert_with(|| flume::unbounded());

        let rcv = chan.1.clone();
        Ok(rcv)
    }

    pub async fn update_task_status(
        &mut self,
        chan_name: &str,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // update task array
        let tasks = match self.tasks.get_mut() {
            Ok(tasks) => tasks,
            Err(_) => return Err("Failed to update task status".into()),
        };
        let task = tasks.iter_mut().find(|t| t.id == task_id);
        let t = match task {
            Some(t) => {
                t.status = status;
                t
            }
            None => return Err("Failed to update task status".into()),
        };
        // stream task
        let chan = self
            .ch
            .entry(chan_name.to_owned())
            .or_insert_with(|| flume::unbounded());

        match chan.0.send_async(t.to_owned()).await {
            Ok(_) => {}
            Err(err) => return Err(format!("Failed to send task to channel|Err: {}", err).into()),
        };
        Ok(())
    }
}

// singleton Queue
pub static Q: SyncLazy<Mutex<Queue>> = SyncLazy::new(|| Mutex::new(Queue::new()));

// test
#[cfg(test)]
mod tests {

    use core::time;
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::{task::TaskStatus, Queue};

    #[tokio::test]
    async fn add_task() {
        {
            // receive task channel
            let q = Arc::new(Mutex::new(Queue::new()));

            let q1 = Arc::clone(&q);

            // print received Tasks
            let j = tokio::task::spawn(async move {
                println!("spawned green thread"); //debug
                                                  //let mut qq = q1.lock().await;
                let rcv = q1.lock().await.get_chan("test_chan").unwrap();
                println!("receiving"); //debug
                                       // infinite loop
                                       // async iterator
                while let Ok(t) = rcv.recv_async().await {
                    println!("received task with id: {}", t.id);
                    break; // break loop after receiving 1 task
                }
                println!("received, exiting"); //debug
                return;
            });
            // pause to wait for green thread to start
            {
                tokio::time::sleep(time::Duration::from_secs(1)).await;
            }
            {
                // add task
                let data = "test123abc".as_bytes().to_vec();
                let q = Arc::clone(&q);
                let mut qq = q.lock().await;
                println!("Sending new task");
                assert_eq!(qq.add_task("test_chan", data).await.is_err(), false);
            }
            j.await.unwrap();
        }
    }

    #[tokio::test]
    async fn update_task() {
        {
            // receive task channel
            let q = Arc::new(Mutex::new(Queue::new()));

            let q1 = Arc::clone(&q);

            // print received Tasks
            let j = tokio::task::spawn(async move {
                println!("spawned green thread"); //debug
                                                  //let mut qq = q1.lock().await;
                let rcv = q1.lock().await.get_chan("test_chan").unwrap();
                println!("receiving"); //debug
                                       // infinite loop
                                       // async iterator
                while let Ok(t) = rcv.recv_async().await {
                    if t.status == TaskStatus::Completed {
                        println!("received Completed task with id: {}", t.id);
                        break; // break loop after receiving 1 task
                    }
                }
                println!("received, exiting"); //debug
                return;
            });
            // pause to wait for green thread to start
            {
                tokio::time::sleep(time::Duration::from_secs(1)).await;
            }
            {
                // add task
                let data = "test123abc".as_bytes().to_vec();
                let q = Arc::clone(&q);
                let mut qq = q.lock().await;
                println!("Sending new task");
                let task_add_res = qq.add_task("test_chan", data).await;
                assert_eq!(task_add_res.is_err(), false);
                let task_id = task_add_res.unwrap();
                assert_eq!(
                    qq.update_task_status("test_chan", &task_id, TaskStatus::Completed)
                        .await
                        .is_err(),
                    false
                );
            }
            j.await.unwrap();
        }
    }
}
