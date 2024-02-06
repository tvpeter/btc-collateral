use crate::constants::environment_vars;
use bitcoin::{Transaction, TxOut, Txid};
use bitcoincore_rpc::{Auth, Client, Error, RpcApi};

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

pub fn get_transaction_output(
	txid: Txid,
	vout: u32,
) -> Result<(bool, Option<TxOut>, Transaction), Error> {
	let client = connect_bitcoind();

	let txn = client.get_raw_transaction(&txid, None)?;

	let is_segwit_txn = !txn.input.iter().all(|input| input.witness.is_empty());

	let index = vout as usize;
	Ok((is_segwit_txn, txn.output.get(index).cloned(), txn))
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

	#[test]
	fn test_get_transaction_output() {
		let txid =
			Txid::from_str("c770d364d87768dcf0778bf48f095c753e838329d6cc7a3b4fc759317d4efd08")
				.unwrap();
		let index = 0;
		let rawhex = get_transaction_output(txid, index).unwrap();

		println!("raw hex: {:?}", rawhex);
	}
}
