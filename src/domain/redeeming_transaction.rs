use crate::domain::funding_transaction::TxnOutpoint;
use crate::utils::transaction_utils::{get_outpoints_total, Txn};
use bitcoin::absolute::LockTime;
use bitcoin::bip32::Xpub;
use bitcoin::blockdata::fee_rate;
use bitcoin::psbt::Input;
use bitcoin::transaction::Version;
use bitcoin::{Psbt, Transaction, TxOut};
use std::collections::BTreeMap;
use std::str::FromStr;

pub const PRECISION: i32 = 8;

#[derive(Debug, Clone)]
pub struct RedeemingTxnPSBT {
	receiving_address: String,
	amount: f64,
	inputs: Vec<TxnOutpoint>,
	// we might charge a fee of 0.025% on the redemption amount
	change_address: String,
}

impl RedeemingTxnPSBT {
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

		let tx_inputs = RedeemingTxnPSBT::calculate_inputs(&self.inputs)?;

		let initial_output = self.calculate_outputs(input_total, 0.0)?;
		let fees = RedeemingTxnPSBT::calculate_fees(initial_output, tx_inputs.clone())?;

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
			RedeemingTxnPSBT::derive_script_pubkeys(&self.receiving_address, &self.change_address)?;

		let (amount_in_hex, change_amount_hex) =
			RedeemingTxnPSBT::hex_amounts(self.amount, fees, input_total)?;

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

	fn create_psbt(&self) -> Result<Psbt, String> {
		let unsigned_txn = self.construct_trxn()?;
		Ok(Psbt {
			unsigned_tx: unsigned_txn,
			xpub: Default::default(),
			version: 0,
			proprietary: BTreeMap::new(),
			unknown: BTreeMap::new(),
			inputs: vec![],
			outputs: vec![],
		})
	}
}

impl Txn for RedeemingTxnPSBT {}

#[cfg(test)]
mod tests {
	use super::RedeemingTxnPSBT;
	use crate::domain::funding_transaction::TxnOutpoint;
	use bitcoincore_rpc::RawTx;

	fn redeem_txn() -> RedeemingTxnPSBT {
		let tx_input = vec![TxnOutpoint::create_outpoint(
			"a39122aefe9563c17426bd468d2b650467475ea4c3bb538d0091d2552f6468d3".to_owned(),
			1,
		)
		.unwrap()];

		RedeemingTxnPSBT::new(
			"bcrt1qeygjhsgt5sumtlqnyfu58harh3737z96m0zmqv".to_string(),
			1.8,
			tx_input,
			"bcrt1q8ucxfsyajsdghspzpn8mx8m7gyfv0c8jfn60m7".to_string(),
		)
	}

	#[test]
	fn test_create_psbt() {
		let redeem_txn = redeem_txn();

		let psbt = redeem_txn.create_psbt();

		let psbt = match psbt {
			Ok(psbt) => psbt,
			Err(error) => panic!("Error: {:?}", error),
		};

		println!("psbt: {:?}", psbt.serialize_hex());
	}
}
