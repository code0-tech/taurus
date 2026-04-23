mod app;
mod client;
mod config;

#[tokio::main]
async fn main() {
    app::run().await;
}
