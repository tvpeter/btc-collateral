use crate::service::{create_user, health_check, wallet_service};
use actix_web::{dev::Server, web, App, HttpServer};
use bdk::bitcoin::Network;
use bdk::database::SqliteDatabase;
use bdk::{testutils, Wallet};
use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub struct AppState {
	pub db: PgPool,
	pub passkey: Mutex<String>,
	pub wallet: Arc<Mutex<Wallet<SqliteDatabase>>>,
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
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
		db: db_pool.clone(),
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
			.route("/create_user", web::post().to(create_user))
	})
	.listen(listener)?
	.run();
	Ok(server)
}
