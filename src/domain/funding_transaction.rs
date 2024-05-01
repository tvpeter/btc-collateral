use crate::constants::set_network;
use crate::utils::get_feerate::get_mempool_feerate;
use crate::utils::transaction_utils::{get_outpoints_total, Txn};
use bitcoin::absolute::LockTime;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxOut};
use bitcoin::transaction::Version;
use bitcoin::Network;
use bitcoincore_rpc::Client;

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

	pub fn construct_trxn(&self, client: Option<&Client>) -> Result<Transaction, String> {
		let input_total = if set_network() == Network::Regtest && client.is_some() {
			let rpc_client = client.expect("No client supplied");
			get_outpoints_total(&self.inputs, Some(rpc_client)).map_err(|e| format!("{:?}", e))?
		} else {
			get_outpoints_total(&self.inputs, None).map_err(|e| format!("{:?}", e))?
		};

		if input_total < self.amount {
			return Err(format!("Insufficient amount provided: {}", input_total));
		}

		let fee_rates = get_mempool_feerate().map_err(|e| format!("{:?}", e))?;
		let tx_inputs = FundingTxn::calculate_inputs(&self.inputs);

		let initial_output = self.calculate_outputs(input_total, 0.0)?;
		let fees = FundingTxn::calculate_fees(initial_output, tx_inputs.clone(), &fee_rates)?;

		let tx_outputs = self
			.calculate_outputs(input_total, fees)
			.map_err(|err| format!("{:?}", err))?;

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
	use super::*;
	use crate::utils::test_node::TestNode;
	use crate::{domain::funding_transaction::FundingTxn, utils::get_feerate::MempoolSpaceFeeRate};
	use bitcoin::Amount;
	use bitcoincore_rpc::RawTx;
	use round::round_down;

	#[test]
	#[ignore = "failing when run with all the tests but passes as a single or this module"]
	fn test_create_txn() {
		let client = TestNode::new().unwrap();
		let address_1 = client.new_address(None).unwrap();
		let address_2 = client.new_address(None).unwrap();
		let receiving_address = client.new_address(None).unwrap();
		let change_address = client.new_address(None).unwrap();

		let _ = client.generate_to_address(101, address_1.clone());

		let balance = client.get_balance().unwrap();

		assert_eq!(50.0, balance.to_btc());

		let txid = client.send(&address_2, Amount::from_int_btc(5)).unwrap();

		let _ = client.generate_to_address(10, address_1);

		let vout_index = client.get_vout(txid).unwrap();

		let mut txinputs = Vec::new();

		let outpoint1 = OutPoint::new(txid, vout_index);
		txinputs.push(outpoint1);

		let fdn_txn = FundingTxn::new(
			receiving_address.to_string(),
			2.56,
			txinputs,
			change_address.to_string(),
		);

		let txn = fdn_txn
			.construct_trxn(Some(&client.bitcoind.client))
			.unwrap();
		assert_eq!(txn.version, Version::TWO);
		assert!(!txn.is_coinbase());
		assert!(!txn.raw_hex().is_empty());
		assert_eq!(txn.lock_time, LockTime::ZERO);
		assert_eq!(txn.output.len(), 2);
		assert_eq!(txn.input.len(), 1);
	}

	#[test]
	#[ignore = "failing when run with all the tests but passes as a single or this module"]
	fn test_txn_fees() {
		let client = TestNode::new().unwrap();
		let address_1 = client.new_address(None).unwrap();
		let address_2 = client.new_address(None).unwrap();
		let receiving_address = client.new_address(None).unwrap();
		let change_address = client.new_address(None).unwrap();

		let amount_to_spend = 2.56;

		let _ = client.generate_to_address(101, address_1.clone());

		let balance = client.get_balance().unwrap();

		assert_eq!(50.0, balance.to_btc());

		let txid = client.send(&address_2, Amount::from_int_btc(5)).unwrap();

		let _ = client.generate_to_address(10, address_1);

		let vout_index = client.get_vout(txid).unwrap();

		let mut txinputs = Vec::new();

		let outpoint1 = OutPoint::new(txid, vout_index);
		txinputs.push(outpoint1);

		let fdn_txn = FundingTxn::new(
			receiving_address.to_string(),
			amount_to_spend,
			txinputs,
			change_address.to_string(),
		);
		let input_total =
			get_outpoints_total(&fdn_txn.inputs, Some(&client.bitcoind.client)).unwrap();

		let txn_details = fdn_txn
			.construct_trxn(Some(&client.bitcoind.client))
			.unwrap();

		let inputs = FundingTxn::calculate_inputs(&fdn_txn.inputs);
		let tx_outputs = fdn_txn.calculate_outputs(input_total, 0.0).unwrap();
		let fee_rate = MempoolSpaceFeeRate {
			fastest_fee: 15,
			half_hour_fee: 14,
			hour_fee: 13,
			economy_fee: 12,
			minimum_fee: 10,
		};
		let computed_fees = FundingTxn::calculate_fees(tx_outputs, inputs, &fee_rate).unwrap();

		let v_size = txn_details.vsize();

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
