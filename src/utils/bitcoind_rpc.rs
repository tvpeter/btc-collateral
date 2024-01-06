use bitcoin::{Amount, Txid};
use bitcoincore_rpc::{Auth, Client, Error, RpcApi};

pub fn connect_bitcoind() -> Client {
	let rpc_client = Client::new(
		"http://localhost:18443",
		Auth::UserPass("bitcoin".to_string(), "bitcoin".to_string()),
	)
	.unwrap();

	let best_block_hash = rpc_client.get_best_block_hash().unwrap();
	println!(
		"Connected to bitcoind. Best block hash: {}",
		best_block_hash
	);
	rpc_client
}

pub fn get_outpoint_value(txid: Txid, vout: u32) -> f64 {
	let rpc = connect_bitcoind();

	let outpoint_value = rpc.get_tx_out(&txid, vout, Some(false)).unwrap();

	let result = match outpoint_value {
		Some(amount) => amount,
		None => panic!("Error getting transaction amount for: {:?}", &txid),
	};

	let amount = result.value;
	amount.to_btc()
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
		let vout: u32 = 0;

		assert_eq!(get_outpoint_value(txid, vout), 1.56250000);
	}
}
