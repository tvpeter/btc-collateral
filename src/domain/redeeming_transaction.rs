use crate::domain::funding_transaction::TxnOutpoint;
use crate::utils::transaction_utils::Txn;
use std::str::FromStr;

pub const PRECISION: i32 = 8;

#[derive(Debug, Clone)]
pub struct RedeemingTxnPSBT {
	receiving_address: String,
	amount: f64,
	inputs: Vec<TxnOutpoint>,
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
}

impl Txn for RedeemingTxnPSBT {}
