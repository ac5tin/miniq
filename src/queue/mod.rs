use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, sync};

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

    pub fn add_task(&self, data: &Vec<u8>) -> String {
        let task = task::Task::new(data);
        let id = task.id.clone();
        // let chan = self.ch.get(&id).unwrap().chan.clone();
        // chan.0.send(task).unwrap();
        id
    }
}
