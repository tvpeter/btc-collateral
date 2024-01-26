use crate::utils::transaction_utils::{get_outpoints_total, Txn};
use bitcoin::absolute::LockTime;
use bitcoin::transaction::Version;
use bitcoin::{
	blockdata::transaction::{Transaction, TxOut},
	Txid,
};
use std::str::FromStr;

pub const PRECISION: i32 = 8;

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
	receiving_address: String,
	amount: f64,
	inputs: Vec<TxnOutpoint>,
	change_address: String,
}

impl FundingTxn {
	fn new(
		receiving_address: String,
		amount: f64,
		inputs: Vec<TxnOutpoint>,
		change_address: String,
	) -> Self {
		Self {
			receiving_address,
			amount,
			inputs,
			change_address,
		}
	}

	pub fn construct_trxn(&self) -> Result<Transaction, String> {
		let mut input_total = 0.0;
		match get_outpoints_total(&self.inputs) {
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

		let tx_inputs = FundingTxn::calculate_inputs(&self.inputs)?;

		let initial_output = self.calculate_outputs(input_total, 0.0)?;
		let fees = FundingTxn::calculate_fees(initial_output, tx_inputs.clone())?;

		let tx_outputs = match self.calculate_outputs(input_total, fees) {
			Ok(value) => value,
			Err(error) => return Err(format!("{:?}", error)),
		};

		Ok(Transaction {
			version: Version::TWO,
			lock_time: LockTime::ZERO,
			input: tx_inputs,
			output: tx_outputs,
		})
	}

	fn calculate_outputs(&self, input_total: f64, fees: f64) -> Result<Vec<TxOut>, String> {
		let (receiving_spkh, change_spkh) =
			FundingTxn::derive_script_pubkeys(&self.receiving_address, &self.change_address)?;

		let (amount_in_hex, change_amount_hex) =
			FundingTxn::hex_amounts(self.amount, fees, input_total)?;

		let mut tx_outputs = Vec::new();
		let output1 = TxOut {
			value: amount_in_hex,
			script_pubkey: receiving_spkh,
		};
		tx_outputs.push(output1);
		let output2 = TxOut {
			value: change_amount_hex,
			script_pubkey: change_spkh,
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

impl Txn for FundingTxn {}

#[cfg(test)]
mod test {
	use bitcoin::Amount;
	use bitcoincore_rpc::RawTx;
	use round::round_down;

	use crate::{domain::funding_transaction::FundingTxn, utils::get_feerate::get_mempool_feerate};

	use super::*;

	fn funding_txn() -> FundingTxn {
		let mut txinputs = Vec::new();

		let outpoint1 = TxnOutpoint::create_outpoint(
			"0de1989117a98627fb8d350d4e568c8ff7ee7e627463a7631ff754680424290b".to_owned(),
			0,
		)
		.unwrap();
		txinputs.push(outpoint1);

		let outpoint2 = TxnOutpoint::create_outpoint(
			"a8a1ef53cd9e25277880f097ab1203e8a98edbf99e3c03272a43dbe36d0dd2dc".to_owned(),
			0,
		)
		.unwrap();
		txinputs.push(outpoint2);

		FundingTxn::new(
			"2My2o4T4ong11WcGnyyNDqaqoU3NhS1kagJ".to_owned(),
			2.56,
			txinputs,
			"bcrt1qq935ysfqnlj9k4jd88hjj093xu00s9ge0a7l5m".to_owned(),
		)
	}

	#[test]
	fn test_create_txn() {
		let new_txn = funding_txn();

		let txn = new_txn.create_txn().unwrap();

		println!("raw hex: {}", txn.raw_hex());
		assert_eq!(txn.version, Version::TWO);
		assert!(!txn.is_coinbase());
		assert!(!txn.raw_hex().is_empty());
		assert_eq!(txn.lock_time, LockTime::ZERO);
		assert_eq!(txn.output.len(), 2);
		assert_eq!(txn.input.len(), 2);
	}

	#[test]
	fn test_txn_fees() {
		let new_txn = funding_txn();
		let input_total = get_outpoints_total(&new_txn.inputs).unwrap();

		let txn = new_txn.create_txn().unwrap();

		let inputs = FundingTxn::calculate_inputs(&new_txn.inputs).unwrap();
		let tx_outputs = new_txn.calculate_outputs(input_total, 0.0).unwrap();
		let computed_fees = FundingTxn::calculate_fees(tx_outputs, inputs).unwrap();

		let v_size = txn.vsize();
		let fee_rate = get_mempool_feerate().unwrap();
		let input_len = txn.input.len();

		let total_size = v_size + (input_len * 72);

		let fees = fee_rate.fastest_fee * total_size;

		let fees_in_btc = round_down(
			Amount::from_sat(fees.try_into().unwrap()).to_btc(),
			PRECISION,
		);

		assert_eq!(computed_fees, fees_in_btc);
	}
}
