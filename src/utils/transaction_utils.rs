use bitcoin::{
	absolute::LockTime, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction,
	TxIn, TxOut, Witness,
};
use round::round_down;

use super::{
	bitcoind_rpc::get_outpoint_value, get_feerate::get_mempool_feerate,
	validate_address::validate_address,
};
use crate::{
	constants::set_network,
	domain::funding_transaction::{TxnOutpoint, PRECISION},
};

pub fn get_outpoints_total(inputs: &Vec<TxnOutpoint>) -> Result<f64, String> {
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

pub trait Txn {
	fn calculate_inputs(inputs: &Vec<TxnOutpoint>) -> Result<Vec<TxIn>, String> {
		let mut tx_inputs = Vec::new();
		for tx_input in inputs {
			let outpoint = OutPoint {
				txid: tx_input.txid,
				vout: tx_input.vout,
			};

			let input_detail = TxIn {
				previous_output: outpoint,
				script_sig: ScriptBuf::new(),
				sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
				witness: Witness::new(),
			};

			tx_inputs.push(input_detail);
		}
		Ok(tx_inputs)
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

	fn hex_amounts(amount: f64, fees: f64, input_total: f64) -> Result<(Amount, Amount), String> {
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

	#[test]
	fn test_get_outpoints_total() {}
}
