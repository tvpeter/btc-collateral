use bitcoin::{address::NetworkUnchecked, Address, Network};

pub fn validate_address(address: &str, network: Network) -> Address {
	let unchecked_address: Address<NetworkUnchecked> = address.parse().unwrap();

	unchecked_address
		.require_network(network)
		.expect("Error decoding address for given network")
}
