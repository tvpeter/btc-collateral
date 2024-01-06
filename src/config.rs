use actix_web::cookie::time::error;
use bitcoin::Network;
use config::{Config, ConfigError, File};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
	pub application_port: u16,
}

impl Settings {
	pub fn get_configuration() -> Result<Self, ConfigError> {
		let settings = Config::builder()
			.add_source(File::with_name("config"))
			.build()?;

		settings.try_deserialize()
	}
}

pub fn environment_vars() -> HashMap<&'static str, std::string::String> {
	dotenv().ok();
	let mut config_vars = HashMap::new();

	let env_vars = ["NETWORK", "NODE_URL", "RPC_USERNAME", "RPC_PASSWORD"];

	for env_var in env_vars {
		if env::var(env_var).is_err() {
			panic!("{:?} is not set", env_var);
		};
	}
	let network = env::var("NETWORK").unwrap();

	config_vars.insert("network", network);

	let node_url = env::var("NODE_URL").unwrap();
	config_vars.insert("node_url", node_url);

	let rpc_username = env::var("RPC_USERNAME").unwrap();
	config_vars.insert("rpc_username", rpc_username);

	let rpc_password = env::var("RPC_PASSWORD").unwrap();
	config_vars.insert("rpc_password", rpc_password);

	config_vars
}

pub fn set_network() -> Network {
	let env_vars = environment_vars();
	let network_var = env_vars.get("network").unwrap();
	let parsed_network = Network::from_str(network_var);

	match parsed_network {
		Ok(network) => network,
		Err(error) => panic!("Error parsing supplied network: {:?}", error),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_set_network() {
		assert_eq!(set_network(), Network::Regtest);
	}
}
