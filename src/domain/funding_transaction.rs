use crate::config::set_network;
use crate::utils::bitcoind_rpc::get_outpoint_value;
use crate::utils::validate_address::validate_address;
use bitcoin::absolute::LockTime;
use bitcoin::transaction::Version;
use bitcoin::{
	blockdata::transaction::{OutPoint, Transaction, TxOut},
	Amount, Txid,
};
use bitcoin::{ScriptBuf, Sequence, TxIn, Witness};
use std::ptr::null;
use std::str::FromStr;

const MAX_AMOUNT: u32 = 20;
const FEE_RATE: f64 = 0.00072;

#[derive(Debug, Clone)]
pub struct TxnOutpoint {
	pub txid: Txid,
	pub vout: u32,
}

impl TxnOutpoint {
	fn create_outpoint(txid: String, vout: u32) -> OutPoint {
		OutPoint {
			txid: Txid::from_str(&txid).expect("Please supply a valid transaction id"),
			vout,
		}
	}
}

pub struct TxOutput {
	//supply amount in BTC
	amount: Amount,
	address: String,
}

impl TxOutput {
	fn new(amount: u32, address: String) -> Self {
		if amount > MAX_AMOUNT {
			panic!("The amount supplied exceed maximum allowed amount");
		}
		let tx_amount = Amount::from_btc(amount.into()).expect("supply valid transaction amount");

		Self {
			amount: tx_amount,
			address,
		}
	}
}

#[derive(Debug, Clone)]
pub struct FundingTxn {
	address: String,
	amount: u64,
	version: i32,
	inputs: Vec<TxnOutpoint>,
	change_address: String,
}

impl FundingTxn {
	fn new(
		address: String,
		amount: u64,
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
				if amount < self.amount as f64 {
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

	fn calculate_inputs(&self) -> Result<Vec<TxIn>,  String> {
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
		let balance = input_total as u64 - self.amount;
		let change_amount = balance as f64 - FEE_RATE;
		let mut tx_outputs = Vec::new();
		let output1 = TxOut {
			value: Amount::from_int_btc(self.amount),
			script_pubkey: receiving_script_pubkey_hash,
		};
		tx_outputs.push(output1);
		let output2 = TxOut {
			value: Amount::from_int_btc(change_amount as u64),
			script_pubkey: change_script_pubkey_hash,
		};
		tx_outputs.push(output2);
		Ok(tx_outputs)
	}
}
