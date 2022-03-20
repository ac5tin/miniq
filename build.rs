fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tonic_build::configure()
        .build_server(true)
        // .out_dir("src/queue/proto")
        .compile(&["src/queue/proto/queue.proto"], &["."])?;
    Ok(())
}
