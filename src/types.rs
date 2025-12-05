pub struct Data {
	// Path to the SQLite database file
	pub db_path: String,
	// HTTP client for making API requests
	pub http_client: reqwest::Client,
	// Minecraft server address to query
	pub mc_server_address: String,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Context<'a> = poise::Context<'a, Data, Error>;
