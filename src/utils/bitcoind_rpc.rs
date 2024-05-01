use crate::constants::{environment_vars, set_network};
use anyhow::{anyhow, Result};
use bitcoin::{Network, Transaction, TxOut, Txid};
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

pub fn get_outpoint_value(txid: Txid, vout: u32, client: Option<&Client>) -> anyhow::Result<f64> {
	let outpoint_value = if set_network() == Network::Regtest && client.is_some() {
		let rpc = client.expect("Failed to obtain Bitcoin Core RPC client");
		rpc.get_tx_out(&txid, vout, Some(false))?
	} else {
		let rpc = connect_bitcoind();
		rpc.get_tx_out(&txid, vout, Some(false))?
	};

	let tx_result = match outpoint_value {
		Some(amount) => amount,
		None => return Err(anyhow!("Error getting UTXO value for for txid: {:?}", txid)),
	};

	Ok(tx_result.value.to_btc())
}

pub fn get_transaction_output(
	txid: Txid,
	vout: u32,
	client: Option<&Client>,
) -> Result<(bool, Option<TxOut>, Transaction), Error> {
	let txn = if set_network() == Network::Regtest && client.is_some() {
		let rpc = client.expect("Failed to obtain Bitcoin Core RPC client");
		rpc.get_raw_transaction(&txid, None)?
	} else {
		let rpc = connect_bitcoind();
		rpc.get_raw_transaction(&txid, None)?
	};

	let is_segwit_txn = !txn.input.iter().all(|input| input.witness.is_empty());

	let index = vout as usize;
	Ok((is_segwit_txn, txn.output.get(index).cloned(), txn))
}
#[cfg(test)]
mod test {
	use std::str::FromStr;

	use crate::utils::test_node::TestNode;
	use bitcoin::Amount;

	use super::*;

	#[test]
	#[ignore = "failing when run with all the tests but passes as a single or this module"]
	fn test_get_outpoint_value() {
		let client = TestNode::new().unwrap();
		let address_1 = client.new_address(None).unwrap();
		let address_2 = client.new_address(None).unwrap();
		let _ = client.generate_to_address(101, address_1.clone());

		let balance = client.get_balance().unwrap();

		assert_eq!(50.0, balance.to_btc());

		let txid = client.send(&address_2, Amount::from_int_btc(5)).unwrap();

		let _ = client.generate_to_address(10, address_1);

		let vout_index = client.get_vout(txid).unwrap();

		let outpoint_value =
			get_outpoint_value(txid, vout_index, Some(&client.bitcoind.client)).unwrap();

		assert_eq!(outpoint_value, 5.0);
	}

	#[test]
	fn test_get_transaction_output() {
		let txid =
			Txid::from_str("c770d364d87768dcf0778bf48f095c753e838329d6cc7a3b4fc759317d4efd08")
				.unwrap();
		let index = 0;
		let rawhex = get_transaction_output(txid, index, None).unwrap();

		println!("raw hex: {:?}", rawhex);
	}
}
