use crate::constants::set_network;
use crate::utils::bitcoind_rpc::get_outpoint_value;
use crate::utils::validate_address::validate_address;
use bitcoin::absolute::LockTime;
use bitcoin::transaction::Version;
use bitcoin::{
	blockdata::transaction::{OutPoint, Transaction, TxOut},
	Amount, Txid,
};
use bitcoin::{ScriptBuf, Sequence, TxIn, Witness};
use round::round_down;
use std::str::FromStr;

const FEE_RATE: f64 = 0.00072;
const PRECISION: i32 = 8;

#[derive(Debug, Clone)]
pub struct TxnOutpoint {
	pub txid: Txid,
	pub vout: u32,
}

impl TxnOutpoint {
	fn create_outpoint(txid: String, vout: u32) -> Result<Self, String> {
		let given_txid = Txid::from_str(&txid);

		let txid = match given_txid {
			Ok(txid) => txid,
			Err(err) => return Err(format!("Error parsing given txid id: {}", err)),
		};

		Ok(Self { txid, vout })
	}
}

#[derive(Debug, Clone)]
pub struct FundingTxn {
	address: String,
	amount: f64,
	version: i32,
	inputs: Vec<TxnOutpoint>,
	change_address: String,
}

impl FundingTxn {
	fn new(
		address: String,
		amount: f64,
		version: i32,
		inputs: Vec<TxnOutpoint>,
		change_address: String,
	) -> Self {
		Self {
			address,
			amount,
			version,
			inputs,
			change_address,
		}
	}

	pub fn input_total(&self) -> Result<f64, String> {
		let inputs = &self.inputs;
		let mut inputs_total: f64 = 0.0;

		for input in inputs {
			let outpoint_value = get_outpoint_value(input.txid, input.vout);
			let value = match outpoint_value {
				Ok(amount) => amount,
				Err(err) => return Err(format!("{:?}", err)),
			};

			inputs_total += value;
		}

		Ok(inputs_total)
	}

	pub fn construct_trxn(&self) -> Result<Transaction, String> {
		match self.version {
			1 => Version::ONE,
			2 => Version::TWO,
			_ => return Err("Unknown transaction version".to_string()),
		};

		let mut input_total = 0.0;
		match self.input_total() {
			Ok(amount) => {
				if amount < self.amount {
					return Err(
						"The given UTXO set do not have enough value for this transaction"
							.to_string(),
					);
				} else {
					input_total = amount;
				}
			}
			Err(error) => return Err(format!("{:?}", error)),
		};

		let tx_outputs = match self.calculate_outputs(input_total) {
			Ok(value) => value,
			Err(error) => return Err(format!("{:?}", error)),
		};

		let tx_inputs = match self.calculate_inputs() {
			Ok(value) => value,
			Err(value) => return Err(value),
		};

		Ok(Transaction {
			version: Version(self.version),
			lock_time: LockTime::ZERO,
			input: tx_inputs,
			output: tx_outputs,
		})
	}

	fn calculate_inputs(&self) -> Result<Vec<TxIn>, String> {
		let mut tx_inputs = Vec::new();
		for tx_input in &self.inputs {
			let outpoint = OutPoint {
				txid: tx_input.txid,
				vout: tx_input.vout,
			};

			let script_sig = ScriptBuf::from_hex("");

			let derived_sig = match script_sig {
				Ok(sig) => sig,
				Err(error) => return Err(format!("Error converting from hex: {:?}", error)),
			};

			let witness_data = Witness::new();

			let input_detail = TxIn {
				previous_output: outpoint,
				script_sig: derived_sig,
				sequence: Sequence::MAX,
				witness: witness_data,
			};

			tx_inputs.push(input_detail);
		}
		Ok(tx_inputs)
	}

	fn calculate_outputs(&self, input_total: f64) -> Result<Vec<TxOut>, String> {
		let network = set_network();
		let receiving_address = match validate_address(&self.address, network) {
			Ok(address) => address,
			Err(err) => return Err(format!("{:?}", err)),
		};
		let change_address = match validate_address(&self.change_address, network) {
			Ok(address) => address,
			Err(error) => return Err(format!("{:?}", error)),
		};
		let receiving_script_pubkey_hash = receiving_address.script_pubkey();
		let change_script_pubkey_hash = change_address.script_pubkey();

		let input_amount = round_down(input_total, PRECISION);
		let balance = round_down(input_amount - self.amount, PRECISION);
		let change_amount = round_down(balance - FEE_RATE, PRECISION);

		let amount_in_hex = match Amount::from_btc(self.amount) {
			Ok(amt) => amt,
			Err(error) => return Err(format!("Error parsing given amount: {:?}", error)),
		};

		let change_amount_hex = match Amount::from_btc(change_amount) {
			Ok(amt) => amt,
			Err(err) => return Err(format!("Error parsing change amount: {:?}", err)),
		};

		let mut tx_outputs = Vec::new();
		let output1 = TxOut {
			value: amount_in_hex,
			script_pubkey: receiving_script_pubkey_hash,
		};
		tx_outputs.push(output1);
		let output2 = TxOut {
			value: change_amount_hex,
			script_pubkey: change_script_pubkey_hash,
		};
		tx_outputs.push(output2);
		Ok(tx_outputs)
	}

	fn create_txn(&self) -> Result<Transaction, String> {
		let txn = self.construct_trxn();

		let result_txn = match txn {
			Ok(txn) => txn,
			Err(err) => return Err(err),
		};

		Ok(result_txn)
	}
}

#[cfg(test)]
mod test {
	use bitcoincore_rpc::RawTx;

	use super::*;

	#[test]
	fn test_create_txn() {
		let mut txinputs = Vec::new();

		let outpoint1 = TxnOutpoint::create_outpoint(
			"0de1989117a98627fb8d350d4e568c8ff7ee7e627463a7631ff754680424290b".to_owned(),
			0,
		);

		match outpoint1 {
			Ok(outpoint) => {
				txinputs.push(outpoint);
			}
			Err(error) => panic!("{:?}", error),
		}

		let outpoint2 = TxnOutpoint::create_outpoint(
			"a8a1ef53cd9e25277880f097ab1203e8a98edbf99e3c03272a43dbe36d0dd2dc".to_owned(),
			0,
		);

		match outpoint2 {
			Ok(outpoint) => {
				txinputs.push(outpoint);
			}
			Err(error) => panic!("{:?}", error),
		}

		let new_txn = FundingTxn::new(
			"2My2o4T4ong11WcGnyyNDqaqoU3NhS1kagJ".to_owned(),
			2.56,
			2,
			txinputs,
			"bcrt1qq935ysfqnlj9k4jd88hjj093xu00s9ge0a7l5m".to_owned(),
		);

		let construct_txn = new_txn.create_txn();

		let txn = match construct_txn {
			Ok(ntxs) => ntxs,
			Err(error) => panic!("Error creating transaction: {:?}", error),
		};

		assert_eq!(txn.version, Version::TWO);
		assert!(!txn.is_coinbase());
		assert!(!txn.is_lock_time_enabled());
		assert!(!txn.raw_hex().is_empty());
		println!("raw tx: {}", txn.raw_hex());
	}
}
