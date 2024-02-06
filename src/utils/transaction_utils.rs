use super::{
	bitcoind_rpc::get_outpoint_value, get_feerate::get_mempool_feerate,
	validate_address::validate_address,
};
use crate::{constants::set_network, domain::funding_transaction::PRECISION};
use base64::{engine::general_purpose, Engine as _};
use bitcoin::{
	absolute::LockTime, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction,
	TxIn, TxOut, Witness,
};
use round::round_down;

pub fn get_outpoints_total(inputs: &[OutPoint]) -> Result<f64, String> {
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

/// transaction hex (txn_hex) should be in hex format
pub fn convert_txn_hex_to_base64(txn_hex: String) -> Result<String, String> {
	let txn_hex_bytes = hex::decode(txn_hex).expect("Failed to decode given transaction hex");

	Ok(general_purpose::STANDARD.encode(txn_hex_bytes))
}

pub trait Txn {
	fn calculate_inputs(inputs: &[OutPoint]) -> Vec<TxIn> {
		inputs
			.iter()
			.map(|input| TxIn {
				previous_output: OutPoint {
					txid: input.txid,
					vout: input.vout,
				},
				script_sig: ScriptBuf::new(),
				sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
				witness: Witness::new(),
			})
			.collect::<Vec<TxIn>>()
	}

	fn calculate_fees(tx_outputs: Vec<TxOut>, tx_inputs: Vec<TxIn>) -> Result<f64, String> {
		let initial_transaction = Transaction {
			version: Version::TWO,
			lock_time: LockTime::ZERO,
			input: tx_inputs,
			output: tx_outputs,
		};

		let txn_initial_size = initial_transaction.vsize();
		let input_length = initial_transaction.input.len();

		// worse-case size for a signature is 72-bytes
		let final_size = txn_initial_size + (input_length * 72);
		let fees = get_mempool_feerate();
		let fee_rate = match fees {
			Ok(fees) => fees,
			Err(error) => return Err(format!("{:?}", error)),
		};

		let total_fees = fee_rate.fastest_fee * final_size;
		let fee_rate = Amount::from_sat(total_fees.try_into().unwrap());

		Ok(fee_rate.to_btc())
	}

	fn derive_script_pubkeys(
		receiving_address: &str,
		change_address: &str,
	) -> Result<(ScriptBuf, ScriptBuf), String> {
		let network = set_network();
		let receiving_address = validate_address(receiving_address, network)?;
		let change_address = validate_address(change_address, network)?;

		let receiving_script_pubkey_hash = receiving_address.script_pubkey();
		let change_script_pubkey_hash = change_address.script_pubkey();

		Ok((receiving_script_pubkey_hash, change_script_pubkey_hash))
	}

	fn amount_in_hex(amount: f64, fees: f64, input_total: f64) -> Result<(Amount, Amount), String> {
		let input_amount = round_down(input_total, PRECISION);
		let balance = round_down(input_amount - amount, PRECISION);
		let change_amount = round_down(balance - fees, PRECISION);
		let amount_in_hex = match Amount::from_btc(amount) {
			Ok(amt) => amt,
			Err(error) => return Err(format!("Error parsing given amount: {:?}", error)),
		};
		let change_amount_hex = match Amount::from_btc(change_amount) {
			Ok(amt) => amt,
			Err(err) => return Err(format!("Error parsing change amount: {:?}", err)),
		};
		Ok((amount_in_hex, change_amount_hex))
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_convert_txn_hex_to_base64() {
		let txn_hex = "70736274ff010071".to_string();
		let result = "cHNidP8BAHE=".to_string();
		let base64 = convert_txn_hex_to_base64(txn_hex).unwrap();
		println!("result length: {:?}", result.chars().count());
		println!("base64 length: {:?}", base64.chars().count());
		assert_eq!(base64, result);
	}
}
