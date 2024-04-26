use anyhow::Ok;
use bitcoin::BlockHash;
use bitcoincore_rpc::{json::GetBlockchainInfoResult, RpcApi};
use bitcoind::BitcoinD;

pub struct TestNode {
	client: BitcoinD,
}

impl TestNode {
	pub fn new() -> anyhow::Result<Self> {
		let node = BitcoinD::from_downloaded()?;

		let client = TestNode { client: node };

		Ok(client)
	}

	pub fn get_blockchain_info(&self) -> anyhow::Result<GetBlockchainInfoResult> {
		Ok(self.client.client.get_blockchain_info()?)
	}

	pub fn new_address(
		&self,
		address_type: Option<bitcoincore_rpc::json::AddressType>,
	) -> anyhow::Result<bitcoin::Address> {
		Ok(self
			.client
			.client
			.get_new_address(Default::default(), address_type.or(Default::default()))?
			.assume_checked())
	}

	pub fn generate_to_address(
		&self,
		block_num: u64,
		address: bitcoin::Address,
	) -> anyhow::Result<Vec<BlockHash>> {
		Ok(self
			.client
			.client
			.generate_to_address(block_num, &address)?)
	}
}

#[cfg(test)]
mod test {

	use crate::TestNode;

	fn get_test_node() -> TestNode {
		TestNode::new().unwrap()
	}

	#[test]
	fn test_blockchain_info() {
		let client = get_test_node();

		assert_eq!(0, client.get_blockchain_info().unwrap().blocks);
	}

	#[test]
	fn test_get_new_address() {
		let client = get_test_node();

		let address = client
			.new_address(Some(bitcoincore_rpc::json::AddressType::P2shSegwit))
			.unwrap();

		assert_eq!(address.address_type(), Some(bitcoin::AddressType::P2sh));
	}

	#[test]
	fn test_generate_to_address() {
		let client = get_test_node();

		let address = client
			.new_address(Some(bitcoincore_rpc::json::AddressType::Bech32m))
			.unwrap();

		client.generate_to_address(10, address).unwrap();

		assert_eq!(10, client.get_blockchain_info().unwrap().blocks);
	}
}
