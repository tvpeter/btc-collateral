use crate::service::{health_check, wallet_service};
use actix_web::{dev::Server, web, App, HttpServer};
use bdk::bitcoin::Network;
use bdk::database::SqliteDatabase;
use bdk::{testutils, Wallet};
use sqlx::PgConnection;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub struct AppState {
	pub db: PgConnection,
	pub passkey: Mutex<String>,
	pub wallet: Arc<Mutex<Wallet<SqliteDatabase>>>,
}

pub fn run(listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
	// for initializing the wallet state
	let descriptors = testutils!(@descriptors (&"wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/0/*)"));

	let data = web::Data::new(AppState {
		passkey: Mutex::new(String::from("")),
		wallet: Arc::new(Mutex::new(
			Wallet::new(
				&descriptors.0,
				None,
				Network::Regtest,
				SqliteDatabase::new("test.db"),
			)
			.unwrap(),
		)),
		db: connection,
	});

	let server = HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.route("/health_check", web::get().to(health_check))
			.route(
				"/generate_mnemonic",
				web::get().to(wallet_service::generate_mnemonic),
			)
			.route(
				"/setup_wallet",
				web::post().to(wallet_service::create_or_recover_wallet),
			)
			.route("/get_address", web::get().to(wallet_service::get_address))
			.route("/get_balance", web::get().to(wallet_service::get_balance))
	})
	.listen(listener)?
	.run();
	Ok(server)
}
