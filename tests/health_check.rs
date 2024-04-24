use btc_collateral::config::{DatabaseSettings, Settings};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

#[derive(Debug)]
pub struct TestApp {
	pub address: String,
	pub db_pool: PgPool,
}

#[ignore]
#[tokio::test]
async fn health_check_works() {
	// Arrange
	let app = spawn_app().await;
	//perform http requests against our application
	let client = reqwest::Client::new();

	// Act
	let response = client
		.get(&format!("{}/health_check", &app.address))
		.send()
		.await
		.expect("Failed to execute request");

	assert!(response.status().is_success());
}

// launch app in the background ~somehow~
async fn spawn_app() -> TestApp {
	let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
	let port = listener.local_addr().unwrap().port();
	let address = format!("http://127.0.0.1:{}", port);
	let mut configuration = Settings::get_configuration().expect("Failed to read config");
	configuration.database.database_name = Uuid::new_v4().to_string();
	let connection_pool = configure_database(&configuration.database).await;

	let server = btc_collateral::startup::run(listener, connection_pool.clone())
		.expect("Failed to bind address");
	// launch the server as a background task
	let _ = tokio::spawn(server).await;

	println!("PRINTING PORT --> http://127.0.0.1:{}", port);

	TestApp {
		address,
		db_pool: connection_pool,
	}
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
	// create db
	let mut connection = PgConnection::connect(&config.connection_string_without_db())
		.await
		.expect("Failed to connect to Postgres");

	connection
		// .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
		.execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
		.await
		.expect("Failed to create database");

	// migrate db
	let connection_pool = PgPool::connect(&config.connection_string())
		.await
		.expect("Failed to connect to Postgres");

	sqlx::migrate!("./migrations")
		.run(&connection_pool)
		.await
		.expect("Failed to migrate database");

	connection_pool
}

#[tokio::test]
async fn create_user_returns_200() {
	let app = spawn_app().await;
	let client = reqwest::Client::new();

	let body = "username=tobi&email=testuser%40example.com&phone=1234567890&password=Password123";

	// let brody = reqwest::Body::from(body);

	let response = client
		.post(&format!("{}/create_user", &app.address))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.send()
		.await
		.expect("Failed to execute create user request");

	assert_eq!(200, response.status().as_u16());

	let db_user = sqlx::query!("SELECT * FROM \"user\" WHERE username = 'tobi'")
		.fetch_one(&app.db_pool)
		.await
		.expect("Failed to fetch user from database");

	assert_eq!(db_user.email, "testuser@example.com".to_string());
	assert_eq!(db_user.phone, Some("1234567890".to_string())); // Convert string literal to Option<String>
}
