#![feature(once_cell)]

use std::net::ToSocketAddrs;

mod grpc;
mod queue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");

    // gRPC server
    {
        let miniq_server = grpc::miniq::MiniQServer {};
        tonic::transport::Server::builder()
            .add_service(grpc::miniq::mini_q::mini_q_server::MiniQServer::new(
                miniq_server,
            ))
            .serve("[::1]:8080".to_socket_addrs().unwrap().next().unwrap())
            .await?;
    }
    Ok(())
}
