#![feature(once_cell)]

use dotenv::dotenv;
use std::{env, net::ToSocketAddrs};

mod grpc;
mod queue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //
    {
        dotenv().ok();
    }
    println!("Hello, world!");

    // gRPC server
    {
        let port = env::var("PORT").unwrap_or("8080".to_owned());
        let miniq_server = grpc::miniq::MiniQServer {};
        println!("Starting gRPC server on port {}", port); // debug
        tonic::transport::Server::builder()
            .add_service(grpc::miniq::mini_q::mini_q_server::MiniQServer::new(
                miniq_server,
            ))
            .serve(
                format!("[::1]:{}", port)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap(),
            )
            .await?;
    }
    Ok(())
}
