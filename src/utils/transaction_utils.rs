use super::{
	bitcoind_rpc::get_outpoint_value, get_feerate::MempoolSpaceFeeRate,
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
		let value = outpoint_value.map_err(|e| format!("{:?}", e))?;
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

	fn calculate_fees(
		tx_outputs: Vec<TxOut>,
		tx_inputs: Vec<TxIn>,
		fees: &MempoolSpaceFeeRate,
	) -> Result<f64, String> {
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
		let total_fees = fees.fastest_fee * final_size;
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

		let amount_in_hex = Amount::from_btc(amount)
			.map_err(|err| format!("Error parsing given amount: {}", err))?;

		let change_amount_hex = Amount::from_btc(change_amount)
			.map_err(|e| format!("Error parsing change amount: {:?}", e))?;

		Ok((amount_in_hex, change_amount_hex))
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::domain::{self};
	use bitcoin::Txid;
	use std::str::FromStr;

	fn tx_outpoints() -> Vec<OutPoint> {
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

		txinputs
	}

	#[test]
	fn test_convert_txn_hex_to_base64() {
		let txn_hex = "70736274ff010071".to_string();
		let result = "cHNidP8BAHE=".to_string();
		let base64 = convert_txn_hex_to_base64(txn_hex).unwrap();
		println!("result length: {:?}", result.chars().count());
		println!("base64 length: {:?}", base64.chars().count());
		assert_eq!(base64, result);
	}

	#[test]
	fn test_get_outpoints_total() {
		let outpoints = [
			OutPoint {
				txid: Txid::from_str(
					"0de1989117a98627fb8d350d4e568c8ff7ee7e627463a7631ff754680424290b",
				)
				.unwrap(),
				vout: 0,
			},
			//  1.56250000
			OutPoint {
				txid: Txid::from_str(
					"f965f67e86b658aae279ac01714a0aa8a78501d8d2b0463b8f298addd47ff0ba",
				)
				.unwrap(),
				vout: 0,
			},
			//  1.56250000
			OutPoint {
				txid: Txid::from_str(
					"c770d364d87768dcf0778bf48f095c753e838329d6cc7a3b4fc759317d4efd08",
				)
				.unwrap(),
				vout: 0,
			},
		];
		// total = 4.6875

		let outpoints_total = get_outpoints_total(&outpoints);
		assert_eq!(outpoints_total, Ok(4.6875));
	}

	#[test]
	fn test_calculate_inputs() {
		let txinputs = tx_outpoints();

		let inputs = domain::funding_transaction::FundingTxn::calculate_inputs(&txinputs);

		assert_eq!(inputs.len(), 2);
		assert_eq!(
			inputs.first(),
			Some(TxIn {
				previous_output: OutPoint {
					txid: txinputs[0].txid,
					vout: txinputs[0].vout,
				},
				script_sig: ScriptBuf::new(),
				sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
				witness: Witness::new()
			})
			.as_ref()
		)
	}

	#[test]
	fn test_derive_script_pubkeys() {
		let receiving_address =
			"bcrt1qt8aseu8nm4zah5sdj44gedqmuty3t32k59959vu7k6t72dy8n82qqhrec3".to_owned();
		let change_address = "bcrt1qq935ysfqnlj9k4jd88hjj093xu00s9ge0a7l5m".to_owned();

		let derived_spks = domain::funding_transaction::FundingTxn::derive_script_pubkeys(
			&receiving_address,
			&change_address,
		);
		let (receiving_spk, change_spk) = derived_spks.unwrap();
		assert_eq!(
			receiving_spk.to_hex_string(),
			"002059fb0cf0f3dd45dbd20d956a8cb41be2c915c556a14b42b39eb697e5348799d4"
		);
		assert_eq!(
			change_spk.to_hex_string(),
			"001401634241209fe45b564d39ef293cb1371ef81519"
		);
	}

	#[test]
	fn test_amount_hex() {
		let input_total = 4.6875;
		let fees = 0.0000453;
		let tx_amount = 2.56;

		let (derived_amount, change_amount) =
			domain::funding_transaction::FundingTxn::amount_in_hex(tx_amount, fees, input_total)
				.unwrap();

		assert_eq!(derived_amount.to_btc(), 2.56000000);
		assert_eq!(change_amount.to_btc(), 2.12745470);
	}
}
