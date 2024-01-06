use crate::config::set_network;
use crate::utils::bitcoind_rpc::get_outpoint_value;
use crate::utils::validate_address::validate_address;
use bitcoin::absolute::LockTime;
use bitcoin::address::{NetworkChecked, NetworkUnchecked};
use bitcoin::transaction::Version;
use bitcoin::{
	blockdata::{
		transaction::{OutPoint, Transaction, TxIn, TxOut},
		FeeRate,
	},
	Amount, ScriptBuf, Sequence, Txid, Witness,
};
use bitcoin::{Address, Network};
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
		let network = set_network();

		match self.version {
			1 => Version::ONE,
			2 => Version::TWO,
			_ => return Err("Unknown transaction version".to_string()),
		};

		let mut input_total = 0.0;
		match self.input_total() {
			Ok(amount) => {
				if amount < self.amount.into() {
					return Err(format!(
						"The given UTXO set do not have enough value for this transaction"
					));
				} else {
					input_total = amount;
				}
			}
			Err(error) => return Err(format!("{:?}", error)),
		};

		let tx_outputs = match self.calculate_outputs(network, input_total) {
			Ok(value) => value,
			Err(value) => return value,
		};

		let txn = Transaction {
			version: bitcoin::transaction::Version(self.version),
			lock_time: LockTime::ZERO,
			input: self.inputs,
			output: tx_outputs,
		};
	}

	fn calculate_outputs(
		&self,
		network: Network,
		input_total: f64,
	) -> Result<Vec<TxOut>, Result<Transaction, String>> {
		let receiving_address = match validate_address(&self.address, network) {
			Ok(address) => address,
			Err(err) => return Err(Err(format!("{:?}", err))),
		};
		let change_address = match validate_address(&self.change_address, network) {
			Ok(address) => address,
			Err(error) => return Err(Err(format!("{:?}", error))),
		};
		let receiving_script_pubkey_hash = receiving_address.script_pubkey();
		let change_script_pubkey_hash = change_address.script_pubkey();
		let balance = input_total - self.amount;
		let change_amount = balance - FEE_RATE;
		let mut tx_outputs = Vec::new();
		let output1 = TxOut {
			value: Amount::from_int_btc(self.amount),
			script_pubkey: receiving_script_pubkey_hash,
		};
		tx_outputs.push(output1);
		let output2 = TxOut {
			value: Amount::from_int_btc(change_amount),
			script_pubkey: change_script_pubkey_hash,
		};
		tx_outputs.push(output2);
		Ok(tx_outputs)
	}

	pub fn create_unsigned_p2pkh_txn(&self) {
		todo!();
	}
}
