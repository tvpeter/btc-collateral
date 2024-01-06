use crate::config::environment_vars;
use bitcoin::Txid;
use bitcoincore_rpc::{Auth, Client, RpcApi};

pub fn connect_bitcoind() -> Client {
	let env_vars = environment_vars();
	let rpc_url = env_vars.get("node_url").unwrap();
	let rpc_username = env_vars.get("rpc_username").unwrap();
	let rpc_password = env_vars.get("rpc_password").unwrap();

	let rpc_client = Client::new(
		rpc_url,
		Auth::UserPass(rpc_username.to_string(), rpc_password.to_string()),
	)
	.unwrap();

	let best_block_hash = rpc_client.get_best_block_hash().unwrap();
	println!(
		"Connected to bitcoind. Best block hash: {}",
		best_block_hash
	);
	rpc_client
}

pub fn get_outpoint_value(txid: Txid, vout: u32) -> Result<f64, String> {
	let rpc = connect_bitcoind();

	let outpoint_value = rpc.get_tx_out(&txid, vout, Some(false)).unwrap();

	let result = match outpoint_value {
		Some(amount) => amount,
		None => return Err(format!("Error getting UTXO value for for txid: {:?}", txid)),
	};

	Ok(result.value.to_btc())
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use super::*;

	#[test]
	fn test_get_outpoint_value() {
		let txid =
			Txid::from_str("641641b49c028c02d150619214d27d384235d69864268b128f7b4cc802eed172")
				.expect("error getting transaction id");
		let valid_vout: u32 = 0;
		let invalid_vout: u32 = 1;

		assert_eq!(get_outpoint_value(txid, valid_vout), Ok(1.56250000));
		assert!(get_outpoint_value(txid, invalid_vout).is_err());
	}
}
