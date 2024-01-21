use crate::constants::set_network;
use crate::utils::bitcoind_rpc::get_outpoint_value;
use crate::utils::get_feerate::get_mempool_feerate;
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

const PRECISION: i32 = 8;

#[derive(Debug, Clone)]
pub struct TxnOutpoint {
	pub txid: Txid,
	pub vout: u32,
}

impl TxnOutpoint {
	pub fn create_outpoint(txid: String, vout: u32) -> Result<Self, String> {
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
	pub fn new(
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
			let utxo_amount = outpoint_value?;
			inputs_total += utxo_amount;
		}

		Ok(inputs_total)
	}

	pub fn construct_trxn(&self) -> Result<Transaction, String> {
		match self.version {
			1 => Version::ONE,
			2 => Version::TWO,
			_ => return Err("Unknown transaction version".to_string()),
		};

		let input_total: f64;
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

		let tx_inputs = self.calculate_inputs()?;

		let fees = self.calculate_fees(tx_inputs.clone(), input_total)?;

		let tx_outputs = self.calculate_outputs(input_total, fees)?;

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

			let witness_data = Witness::new();

			let input_detail = TxIn {
				previous_output: outpoint,
				script_sig: ScriptBuf::new(),
				sequence: Sequence::MAX,
				witness: witness_data,
			};

			tx_inputs.push(input_detail);
		}
		Ok(tx_inputs)
	}

	fn calculate_outputs(&self, input_total: f64, fees: f64) -> Result<Vec<TxOut>, String> {
		let network = set_network();
		let receiving_address = validate_address(&self.address, network)?;
		let change_address = validate_address(&self.change_address, network)?;

		let receiving_script_pubkey_hash = receiving_address.script_pubkey();
		let change_script_pubkey_hash = change_address.script_pubkey();

		let (amount_in_hex, change_amount_hex) = self.input_amounts(fees, input_total)?;

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

	fn input_amounts(&self, fees: f64, input_total: f64) -> Result<(Amount, Amount), String> {
		let input_amount = round_down(input_total, PRECISION);
		let balance = round_down(input_amount - self.amount, PRECISION);
		let change_amount = round_down(balance - fees, PRECISION);
		let amount_in_hex = match Amount::from_btc(self.amount) {
			Ok(amt) => amt,
			Err(error) => return Err(format!("Error parsing given amount: {:?}", error)),
		};
		let change_amount_hex = match Amount::from_btc(change_amount) {
			Ok(amt) => amt,
			Err(err) => return Err(format!("Error parsing change amount: {:?}", err)),
		};
		Ok((amount_in_hex, change_amount_hex))
	}

	pub fn create_txn(&self) -> Result<Transaction, String> {
		let txn = self.construct_trxn()?;

		Ok(txn)
	}

	fn calculate_fees(&self, tx_inputs: Vec<TxIn>, input_total: f64) -> Result<f64, String> {
		let tx_outputs = self.calculate_outputs(input_total, 0.0)?;

		let initial_transaction = Transaction {
			version: Version(self.version),
			lock_time: LockTime::ZERO,
			input: tx_inputs,
			output: tx_outputs,
		};

		let txn_initial_size = initial_transaction.vsize();
		let input_length = initial_transaction.input.len();

		// worse-case size for a signature is 72-bytes
		let final_size = txn_initial_size + (input_length * 72);
		let fees = get_mempool_feerate()?;

		let total_fees = fees.fastest_fee * final_size;
		let fee_rate = Amount::from_sat(total_fees.try_into().unwrap());

		Ok(fee_rate.to_btc())
	}
}

#[cfg(test)]
mod test {
	use bitcoincore_rpc::RawTx;

	use super::*;

	fn funding_txn() -> FundingTxn {
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

		FundingTxn::new(
			"2My2o4T4ong11WcGnyyNDqaqoU3NhS1kagJ".to_owned(),
			2.56,
			2,
			txinputs,
			"bcrt1qq935ysfqnlj9k4jd88hjj093xu00s9ge0a7l5m".to_owned(),
		)
	}

	#[test]
	fn test_create_txn() {
		let new_txn = funding_txn();

		let construct_txn = new_txn.create_txn();

		let txn = match construct_txn {
			Ok(ntxs) => ntxs,
			Err(error) => panic!("Error creating transaction: {:?}", error),
		};

		println!("txn: {}", txn.raw_hex());

		assert_eq!(txn.version, Version::TWO);
		assert!(!txn.is_coinbase());
		assert!(!txn.is_lock_time_enabled());
		assert!(!txn.raw_hex().is_empty());
		assert_eq!(txn.lock_time, LockTime::ZERO);
		assert_eq!(txn.output.len(), 2);
		assert_eq!(txn.input.len(), 2);
	}

	#[test]
	fn test_txn_fees() {
		let new_txn = funding_txn();
		let input_total = new_txn.input_total().unwrap();

		let txn = new_txn.create_txn().unwrap();

		let inputs = new_txn.calculate_inputs().unwrap();
		let computed_fees = new_txn.calculate_fees(inputs, input_total).unwrap();

		let v_size = txn.vsize();
		let fee_rate = get_mempool_feerate().unwrap();
		let input_len = txn.input.len();

		let total_size = v_size + (input_len * 72);

		let fees = fee_rate.fastest_fee * total_size;

		let fees_in_btc = round_down(
			Amount::from_sat(fees.try_into().unwrap()).to_btc(),
			PRECISION,
		);

		assert_eq!(computed_fees, fees_in_btc)
	}
}
