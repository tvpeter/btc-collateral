use crate::utils::transaction_utils::{get_outpoints_total, Txn};
use bitcoin::absolute::LockTime;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxOut};
use bitcoin::transaction::Version;

pub const PRECISION: i32 = 8;

#[derive(Debug, Clone)]
pub struct FundingTxn {
	receiving_address: String,
	amount: f64,
	inputs: Vec<OutPoint>,
	change_address: String,
}

impl FundingTxn {
	pub fn new(
		receiving_address: String,
		amount: f64,
		inputs: Vec<OutPoint>,
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
		let input_total;
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

		let tx_inputs = FundingTxn::calculate_inputs(&self.inputs);

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
			FundingTxn::amount_in_hex(self.amount, fees, input_total)?;

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
}

impl Txn for FundingTxn {}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use bitcoin::{Amount, Txid};
	use bitcoincore_rpc::RawTx;
	use round::round_down;

	use crate::{domain::funding_transaction::FundingTxn, utils::get_feerate::get_mempool_feerate};

	use super::*;

	fn funding_txn() -> FundingTxn {
		let mut txinputs = Vec::new();

		let outpoint1 = OutPoint::new(
			Txid::from_str("c770d364d87768dcf0778bf48f095c753e838329d6cc7a3b4fc759317d4efd08")
				.unwrap(),
			0,
		);
		txinputs.push(outpoint1);

		let outpoint2 = OutPoint::new(
			Txid::from_str("641641b49c028c02d150619214d27d384235d69864268b128f7b4cc802eed172")
				.unwrap(),
			0,
		);
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
		let txn = funding_txn().construct_trxn().unwrap();
		println!("raw hex: {}", txn.raw_hex());
		assert_eq!(txn.version, Version::TWO);
		assert!(!txn.is_coinbase());
		assert!(!txn.raw_hex().is_empty());
		assert_eq!(txn.lock_time, LockTime::ZERO);
		assert_eq!(txn.output.len(), 2);
		assert_eq!(txn.input.len(), 2);
	}

	#[ignore]
	#[test]
	fn test_txn_fees() {
		let txn = funding_txn();
		let input_total = get_outpoints_total(&txn.inputs).unwrap();

		let txn_details = txn.construct_trxn().unwrap();

		let inputs = FundingTxn::calculate_inputs(&txn.inputs);
		let tx_outputs = txn.calculate_outputs(input_total, 0.0).unwrap();
		let computed_fees = FundingTxn::calculate_fees(tx_outputs, inputs).unwrap();

		let v_size = txn_details.vsize();
		let fee_rate = get_mempool_feerate().unwrap();
		let input_len = txn_details.input.len();

		let total_size = v_size + (input_len * 72);

		let fees = fee_rate.fastest_fee * total_size;

		let fees_in_btc = round_down(
			Amount::from_sat(fees.try_into().unwrap()).to_btc(),
			PRECISION,
		);

		assert_eq!(computed_fees, fees_in_btc);
	}
}
