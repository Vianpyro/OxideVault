mod types;
mod mojang;
mod db;
mod commands;
mod bot;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    if let Err(e) = runtime.block_on(bot::run()) {
        eprintln!("Error starting bot: {}", e);
        std::process::exit(1);
    }
}
