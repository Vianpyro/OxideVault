pub struct Data {
	// Path to the SQLite database file
	pub db_path: String,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Context<'a> = poise::Context<'a, Data, Error>;
