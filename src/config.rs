use config::{Config, ConfigError, File};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
	pub application_port: u16,
	pub database: DatabaseSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: String,
	pub port: u16,
	pub host: String,
	pub database_name: String,
}

impl Settings {
	pub fn get_configuration() -> Result<Self, ConfigError> {
		let settings = Config::builder()
			// read the default configuration file
			.add_source(File::with_name("config"))
			.build()?;

		settings.try_deserialize()
	}
}

impl DatabaseSettings {
	pub fn connection_string(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}/{}",
			self.username, self.password, self.host, self.port, self.database_name
		)
	}
}
