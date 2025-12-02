mod types;
mod mojang;
mod db;
mod commands;
mod bot;

#[tokio::main]
async fn main() {
    if let Err(e) = bot::run().await {
        eprintln!("Error starting bot: {}", e);
        std::process::exit(1);
    }
}
