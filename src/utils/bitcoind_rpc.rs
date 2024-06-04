use crate::constants::{environment_vars, set_network};
use anyhow::{anyhow, Result};
use bitcoin::{consensus::deserialize, Network, Transaction, TxOut, Txid};
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
	let outpoint_value = if set_network() == Network::Regtest {
		let rpc = client.expect("Failed to obtain test client");
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
) -> Result<(bool, Transaction, TxOut), Error> {
	let txn = if set_network() == Network::Regtest {
		let rpc = client.expect("Failed to obtain Bitcoin Core RPC client");
		rpc.get_transaction(&txid, None)?
	} else {
		let rpc = connect_bitcoind();
		rpc.get_transaction(&txid, None)?
	};

	let raw_hex = txn.hex;

	let trn: Transaction = deserialize(&raw_hex)?;

	let output = &trn.output.get(vout as usize);

	let tx_out = match *output {
		Some(txn_output) => txn_output,
		None => {
			return Err(bitcoincore_rpc::Error::ReturnedError(
				"invalid index".to_owned(),
			))
		}
	};

	let is_segwit_txn = is_segwit(raw_hex)?;

	Ok((is_segwit_txn, trn.clone(), tx_out.clone()))
}

fn is_segwit(raw_tx: Vec<u8>) -> Result<bool, bitcoin::consensus::encode::Error> {
	let txn: Transaction = deserialize(&raw_tx)?;

	let is_segwit = txn.input.iter().any(|input| !input.witness.is_empty());

	Ok(is_segwit)
}

#[cfg(test)]
mod test {
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

	#[ignore = "fails when ran with others"]
	#[test]
	fn test_get_transaction_output() {
		let client = TestNode::new().unwrap();

		let address_1 = client.new_address(None).unwrap();
		let address_2 = client
			.new_address(Some(bitcoincore_rpc::json::AddressType::P2shSegwit))
			.unwrap();

		let _ = client.generate_to_address(101, address_1.clone());

		assert_eq!(50.0, client.get_balance().unwrap().to_btc());

		let txid = client.send(&address_2, Amount::from_int_btc(10)).unwrap();

		let _ = client.generate_to_address(10, address_1);

		let vout = client.get_vout(txid).unwrap();

		let (is_segwit, txn, txout) =
			get_transaction_output(txid, vout, Some(&client.bitcoind.client)).unwrap();

		assert!(is_segwit);
		assert_eq!(txn.txid(), txid);
		assert_eq!(txout.value, Amount::from_int_btc(10));
	}
}
