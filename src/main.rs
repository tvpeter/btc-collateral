use actix_web::body::MessageBody;
use bitcoin::PublicKey;
use btc_collateral::utils::bitcoind_rpc;
use btc_collateral::{config::Settings, domain::generate_address::PartiesPublicKeys, startup::run};
use dotenv::dotenv;
use std::net::TcpListener;
use std::str::FromStr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();

	let settings = Settings::get_configuration().expect("failed to read config");
	let address = format!("127.0.0.1:{}", settings.application_port);
	let listener = TcpListener::bind(address).expect("Failed to bind random port");
	run(listener)?.await
}

