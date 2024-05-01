use anyhow::Ok;
use bitcoin::{Amount, BlockHash, Txid};
use bitcoincore_rpc::{
	json::{GetBlockchainInfoResult, GetTransactionResultDetailCategory},
	RpcApi,
};
use bitcoind::{exe_path, tempfile::TempDir, BitcoinD, Conf};

#[derive(Debug)]
pub struct TestNode {
	pub bitcoind: BitcoinD,
}

impl TestNode {
	pub fn new() -> anyhow::Result<Self> {
		let mut conf = Conf::default();
		let datadir = TempDir::new()?;
		conf.staticdir = Some(datadir.path().to_path_buf());

		// let node = BitcoinD::from_downloaded()?;

		let node = BitcoinD::with_conf(exe_path()?, &conf)?;

		let client = TestNode { bitcoind: node };

		Ok(client)
	}

	pub fn get_blockchain_info(&self) -> anyhow::Result<GetBlockchainInfoResult> {
		Ok(self.bitcoind.client.get_blockchain_info()?)
	}

	pub fn new_address(
		&self,
		address_type: Option<bitcoincore_rpc::json::AddressType>,
	) -> anyhow::Result<bitcoin::Address> {
		Ok(self
			.bitcoind
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
			.bitcoind
			.client
			.generate_to_address(block_num, &address)?)
	}

	pub fn send(&self, address: &bitcoin::Address, amount: Amount) -> anyhow::Result<Txid> {
		let client = &self.bitcoind.client;

		Ok(client.send_to_address(
			address,
			amount,
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
		)?)
	}

	pub fn get_balance(&self) -> anyhow::Result<Amount> {
		Ok(self
			.bitcoind
			.client
			.get_balance(Default::default(), Default::default())?)
	}

	pub fn get_vout(&self, txid: Txid) -> anyhow::Result<u32> {
		let tx_details = self.bitcoind.client.get_transaction(&txid, None)?;

		let vout = tx_details
			.details
			.iter()
			.filter_map(|item| {
				if item.category == GetTransactionResultDetailCategory::Receive {
					Some(item.vout)
				} else {
					None
				}
			})
			.collect::<Vec<u32>>();
		Ok(vout[0])
	}
}

#[cfg(test)]
mod test {

	use super::*;

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
	#[ignore = "failing when run with all the tests but passes as a single or this module"]
	#[test]
	fn test_sending_to_address() {
		let client = get_test_node();

		let address = client
			.new_address(Some(bitcoincore_rpc::json::AddressType::Bech32m))
			.unwrap();

		let _ = client.generate_to_address(101, address);

		let balance = client
			.bitcoind
			.client
			.get_balance(Default::default(), Default::default())
			.unwrap();

		assert_eq!(50.0, balance.to_btc());

		let new_address = client
			.new_address(Some(bitcoincore_rpc::json::AddressType::Bech32m))
			.unwrap();

		let txid = client.send(&new_address, Amount::from_int_btc(10)).unwrap();

		assert_ne!(None, Some(txid));
		drop(client);
	}
}
