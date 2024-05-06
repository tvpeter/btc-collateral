use crate::utils::bitcoind_rpc::get_transaction_output;
use crate::utils::get_feerate::get_mempool_feerate;
use crate::utils::transaction_utils::{get_outpoints_total, Txn};
use bitcoin::absolute::LockTime;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::psbt::{Input, Output};
use bitcoin::transaction::Version;
use bitcoin::{Psbt, Transaction, TxOut};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RedeemingTxnPSBT {
	pub receiving_address: String,
	pub amount: f64,
	pub inputs: Vec<OutPoint>,
	// we might charge a fee of 0.025% on the redemption amount
	pub change_address: String,
}

impl RedeemingTxnPSBT {
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

		let tx_inputs = RedeemingTxnPSBT::calculate_inputs(&self.inputs);

		let initial_output = self.calculate_outputs(input_total, 0.0)?;
		let fee_rates = get_mempool_feerate().unwrap();
		let fees = RedeemingTxnPSBT::calculate_fees(initial_output, tx_inputs.clone(), &fee_rates)?;

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

	fn create_psbt_inputs(&self) -> Result<Vec<Input>, String> {
		let mut inputs = Vec::new();

		for input in &self.inputs {
			let (segwit_tx_status, tx_outpout, txn) =
				get_transaction_output(input.txid, input.vout)
					.expect("Error getting transaction details");
			if segwit_tx_status {
				inputs.push(Input {
					witness_utxo: Some(tx_outpout.unwrap()),
					..Default::default()
				});
			} else {
				inputs.push(Input {
					non_witness_utxo: Some(txn),
					..Default::default()
				});
			}
		}

		Ok(inputs)
	}

	fn create_psbt_outputs(&self) -> Result<Vec<Output>, String> {
		let (receiving_spkh, change_spkh) =
			RedeemingTxnPSBT::derive_script_pubkeys(&self.receiving_address, &self.change_address)?;

		Ok(vec![
			Output {
				redeem_script: Some(receiving_spkh),
				..Default::default()
			},
			Output {
				redeem_script: Some(change_spkh),
				..Default::default()
			},
		])
	}

	fn calculate_outputs(&self, input_total: f64, fees: f64) -> Result<Vec<TxOut>, String> {
		let (receiving_spkh, change_spkh) =
			RedeemingTxnPSBT::derive_script_pubkeys(&self.receiving_address, &self.change_address)?;

		let (amount_in_hex, change_amount_hex) =
			RedeemingTxnPSBT::amount_in_hex(self.amount, fees, input_total)?;

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

	pub fn create_psbt(&self) -> Result<Psbt, String> {
		let unsigned_txn = self.construct_trxn()?;
		let inputs = self.create_psbt_inputs()?;
		let outputs = self.create_psbt_outputs()?;

		Ok(Psbt {
			unsigned_tx: unsigned_txn,
			xpub: Default::default(),
			version: 0,
			proprietary: BTreeMap::new(),
			unknown: BTreeMap::new(),
			inputs,
			outputs,
		})
	}
}

impl Txn for RedeemingTxnPSBT {}

#[cfg(test)]
mod tests {
	use super::RedeemingTxnPSBT;
	use crate::utils::transaction_utils::convert_txn_hex_to_base64;
	use bitcoin::{blockdata::transaction::OutPoint, Txid};
	use std::str::FromStr;

	fn redeem_txn() -> RedeemingTxnPSBT {
		let tx_input = vec![OutPoint::new(
			Txid::from_str("a39122aefe9563c17426bd468d2b650467475ea4c3bb538d0091d2552f6468d3")
				.unwrap(),
			1,
		)];

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

		let b64 = convert_txn_hex_to_base64(psbt.serialize_hex()).unwrap();

		println!("psbt: {:?}", b64);
	}
}
