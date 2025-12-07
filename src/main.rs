mod types;
mod mojang;
mod database;
mod commands;
mod bot;
mod mc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot::run().await
}
