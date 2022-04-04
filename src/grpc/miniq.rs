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
            channel: self.channel.to_owned(),
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
            channel: t.channel.to_owned(),
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
        match q.add_task(&r.channel, r.data.to_owned()).await {
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
        let req = request.into_inner();
        // infinite stream of tasks
        let (tx, rx) = mpsc::channel(128);

        // stream new tasks
        tokio::task::spawn(async move {
            let req = req.clone();
            let (snd, rcv) = match Q.lock().await.get_chan(&req.channel) {
                Ok(ch) => ch,
                Err(_) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        "Failed to get channel",
                    ))
                }
            };
            // drop(q);

            while let Ok(task) = rcv.recv_async().await {
                if task.status != req.status.into() {
                    continue;
                }
                let t: mini_q::Task = task.clone().into();
                match tx.send(Result::<_, tonic::Status>::Ok(t)).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to send task to client, trying different client");
                        match snd.send_async(task.to_owned()).await {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Failed to send task to client, Err: {}", err);
                            }
                        };
                        break;
                    }
                };
            }
            //println!("Channel closed"); // debug
            Ok(())
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(tonic::Response::new(
            Box::pin(output_stream) as Self::GetTasksStream
        ))
    }

    async fn update_task(
        &self,
        request: tonic::Request<mini_q::UpdateTaskRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let mut q = Q.lock().await;
        let r = request.into_inner();
        match q
            .update_task_status(&r.channel, &r.id, r.status.into())
            .await
        {
            Ok(_) => Ok(tonic::Response::new(())),
            Err(err) => Err(tonic::Status::new(
                tonic::Code::Internal,
                format!("Failed to update task|Err: {}", err),
            )),
        }
    }
}
