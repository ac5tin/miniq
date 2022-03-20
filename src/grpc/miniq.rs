use std::pin::Pin;

use chrono::{DateTime, NaiveDateTime, Utc};
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::queue::{self, Q};

pub mod mini_q {
    tonic::include_proto!("miniq");
}

// convert

impl Into<mini_q::Task> for queue::task::Task {
    fn into(self) -> mini_q::Task {
        let t: mini_q::Task = mini_q::Task {
            id: self.id.to_owned(),
            data: self.data.to_owned(),
            creation_date: Some(prost_types::Timestamp {
                seconds: self.creation_date.timestamp(),
                nanos: self.creation_date.timestamp_subsec_nanos() as i32,
            }),
            status: self.status.to_owned().into(),
        };
        t
    }
}

impl From<mini_q::Task> for queue::task::Task {
    fn from(t: mini_q::Task) -> Self {
        let timestamp = match t.creation_date {
            Some(t) => {
                let timestamp = NaiveDateTime::from_timestamp(t.seconds, t.nanos as u32);
                timestamp
            }
            None => Utc::now().naive_utc(),
        };

        Self {
            id: t.id.to_owned(),
            data: t.data.to_owned(),
            creation_date: DateTime::from_utc(timestamp, Utc),
            status: t.status.into(),
        }
    }
}

// Implement MiniQ gRPC server
pub struct MiniQServer {}

#[tonic::async_trait]
impl mini_q::mini_q_server::MiniQ for MiniQServer {
    type GetTasksStream = Pin<Box<dyn Stream<Item = Result<mini_q::Task, tonic::Status>> + Send>>;

    async fn add_task(
        &self,
        req: tonic::Request<mini_q::AddTaskRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let mut q = Q.lock().await;
        let r = req.into_inner();
        match q.add_task(&r.channel, r.data.to_owned()) {
            Ok(_) => Ok(tonic::Response::new(())),
            Err(err) => Err(tonic::Status::new(
                tonic::Code::Internal,
                format!("Failed to add task|Err: {}", err),
            )),
        }
    }

    async fn get_tasks(
        &self,
        request: tonic::Request<mini_q::GetTaskRequest>,
    ) -> Result<tonic::Response<Self::GetTasksStream>, tonic::Status> {
        // infinite stream of tasks

        //let stream = Arc::new(Mutex::new(tokio_stream::iter(rcv.iter())));

        let (tx, rx) = mpsc::channel(128);
        tokio::task::spawn(async move {
            let q = Q.lock().await;

            let rcv = match q.get_chan(&request.into_inner().name) {
                Ok(rcv) => rcv,
                Err(_) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        "Failed to get channel",
                    ))
                }
            };

            for task in rcv.iter() {
                let t: mini_q::Task = task.into();

                match tx.send(Result::<_, tonic::Status>::Ok(t)).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to send task to channel");
                        break;
                    }
                };
            }
            Ok(())
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(tonic::Response::new(
            Box::pin(output_stream) as Self::GetTasksStream
        ))
    }
}
