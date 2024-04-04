use crate::constants::set_network;
use bitcoin::opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_2, OP_PUSHNUM_3};
use bitcoin::script::Builder;
use bitcoin::{Address, PublicKey, ScriptBuf};

#[derive(Debug, Clone)]
pub struct PartiesPublicKeys {
	pub borrower_pubkey: PublicKey,
	pub lender_pubkey: PublicKey,
	pub service_pubkey: PublicKey,
}

impl PartiesPublicKeys {
	pub fn new(
		borrower_pubkey: PublicKey,
		lender_pubkey: PublicKey,
		service_pubkey: PublicKey,
	) -> Self {
		Self {
			borrower_pubkey,
			lender_pubkey,
			service_pubkey,
		}
	}

	///Redeem_script: OP_2  [pubkey1] [pubkey2] [pubkey3] OP_3 OP_CHECKMULTISIG
	pub fn redeem_script(&self) -> ScriptBuf {
		Builder::new()
			.push_opcode(OP_PUSHNUM_2)
			.push_key(&self.borrower_pubkey)
			.push_key(&self.lender_pubkey)
			.push_key(&self.service_pubkey)
			.push_opcode(OP_PUSHNUM_3)
			.push_opcode(OP_CHECKMULTISIG)
			.into_script()
	}

	/// P2WSH: OP_HASH160 <20-byte hash of redeem script> OP_EQUAL
	pub fn create_p2wsh_address(&self) -> Address {
		Address::p2wsh(&self.redeem_script(), set_network())
	}
}

#[cfg(test)]
mod tests {

	use bitcoin::AddressType;
	use std::str::FromStr;

	use super::*;

	fn valid_publickeys() -> PartiesPublicKeys {
		PartiesPublicKeys {
			borrower_pubkey: PublicKey::from_str(
				"02f0eaa04e609b0044ef1fe09a350dc4b744a5a8604a6fa77bc9bf6443ea50739f",
			)
			.expect("invalid borrower pubkey"),
			lender_pubkey: PublicKey::from_str(
				"037c60db011a840523f216e7198054ef071c5acd3d4b466cf2658b7faf30c11e33",
			)
			.expect("invalid lender pubkey"),
			service_pubkey: PublicKey::from_str(
				"02ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b",
			)
			.expect("invalid service pubkey"),
		}
	}

	#[test]
	fn test_redeem_script() {
		let combined_keys = valid_publickeys();
		assert_eq!(combined_keys.redeem_script().to_hex_string(), "522102f0eaa04e609b0044ef1fe09a350dc4b744a5a8604a6fa77bc9bf6443ea50739f21037c60db011a840523f216e7198054ef071c5acd3d4b466cf2658b7faf30c11e332102ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b53ae");
	}

	#[test]
	fn test_create_p2wsh_address() {
		let valid_instance = valid_publickeys();
		let result = valid_instance.create_p2wsh_address();
		let network = set_network();
		assert_eq!(result.network(), &network);
		assert_eq!(result.address_type(), Some(AddressType::P2wsh));
	}
}
