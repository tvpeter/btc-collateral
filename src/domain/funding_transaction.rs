use bitcoin::address::{NetworkChecked, NetworkUnchecked};
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

const MAX_AMOUNT: u32 = 5;

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

pub struct FundingTxn {
	address: String,
	amount: u32,
	version: i32,
	inputs: Vec<TxnOutpoint>,
	network: Network,
}

impl FundingTxn {
	fn new(
		address: String,
		amount: u32,
		version: i32,
		inputs: Vec<TxnOutpoint>,
		network: Network,
	) -> Self {
		Self {
			address,
			amount,
			version,
			inputs,
			network,
		}
	}

	pub fn validate_inputs(&self) -> ScriptBuf {
		match self.version {
			1 => Version::ONE,
			2 => Version::TWO,
			_ => panic!("Unknown transaction version"),
		};

		//check the outpoints are valid

		let address = self.validate_address();

		address.script_pubkey()
	}

	pub fn validate_address(&self) -> Address {
		let unchecked_address: Address<NetworkUnchecked> = self.address.parse().unwrap();

		unchecked_address
			.require_network(self.network)
			.expect("Error decoding address for given network")
	}
}
