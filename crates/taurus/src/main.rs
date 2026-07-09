mod app;
mod client;
mod config;
mod telemetry;

#[tokio::main]
async fn main() {
    app::run().await;
}
