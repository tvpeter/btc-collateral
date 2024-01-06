use crate::config::set_network;
use crate::utils::validate_address::{self, validate_address};
use bitcoin::address::{NetworkChecked, NetworkUnchecked};
use bitcoin::hashes::Hash;
use bitcoin::{
	absolute::LockTime,
	blockdata::{
		transaction::{OutPoint, Transaction, TxIn, TxOut, Version},
		FeeRate,
	},
	Amount, ScriptBuf, Sequence, Txid, Witness,
};
use bitcoin::{Address, Network};
use std::str::FromStr;

const MAX_AMOUNT: u32 = 20;

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
	amount: u32,
	version: i32,
	inputs: Vec<TxnOutpoint>,
	change_address: String,
}

impl FundingTxn {
	fn new(
		address: String,
		amount: u32,
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

	pub fn validate_inputs(&self) -> (Address, Address) {
		let network = set_network();

		match self.version {
			1 => Version::ONE,
			2 => Version::TWO,
			_ => panic!("Unknown transaction version"),
		};

		//check the outpoints are valid

		let receiving_address = validate_address(&self.address, network);

		let change_address = validate_address(&self.change_address, network);

		(receiving_address, change_address)
	}

	pub fn create_unsigned_p2pkh_txn(&self) {
		let (receiving_address, change_address) = self.validate_inputs();

		let receiving_script_pubkey_hash = receiving_address.script_pubkey();
		let change_script_pubkey_hash = change_address.script_pubkey();
		let input_count = self.inputs.len().to_be_bytes();
		println!("The input count: {:?}", input_count);

		todo!();
	}
}
