mod types;
mod mojang;
mod db;
mod commands;
mod bot;
mod mc_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot::run().await
}
