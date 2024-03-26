use btc_collateral::config::Settings;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[ignore]
#[tokio::test]
async fn health_check_works() {
	// Arrange
	let address = spawn_app().await;
	//perform http requests against our application
	let client = reqwest::Client::new();

	// Act
	let response = client
		.get(&format!("{}/health_check", &address))
		.send()
		.await
		.expect("Failed to execute request");

	assert!(response.status().is_success());
}

// launch app in the background ~somehow~
async fn spawn_app() -> String {
	let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
	let port = listener.local_addr().unwrap().port();

	let configuration = Settings::get_configuration().expect("Failed to read config");
	let connection_pool = PgConnection::connect(&configuration.database.connection_string())
		.await
		.expect("Failed to connect to Postgres");

	let server =
		btc_collateral::startup::run(listener, connection_pool).expect("Failed to bind address");
	// launch the server as a background task
	let _ = tokio::spawn(server).await;

	println!("PRINTING PORT --> http://127.0.0.1:{}", port);
	// return application address to the caller
	format!("http://127.0.0.1:{}", port)
}
