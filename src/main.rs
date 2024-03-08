use btc_collateral::{config::Settings, startup::run};
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let settings = Settings::get_configuration().expect("failed to read config");
	let connection = PgConnection::connect(&settings.database.connection_string())
		.await
		.expect("Failed to connect to postgres");
	let address = format!("127.0.0.1:{}", settings.application_port);
	let listener = TcpListener::bind(address).expect("Failed to bind random port");
	run(listener, connection)?.await
}
