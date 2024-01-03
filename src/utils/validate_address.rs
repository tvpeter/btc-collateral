
use bitcoin::{Address, Network, address::NetworkUnchecked};


pub fn validate_address(address: &String, network: Network) -> Address {
    let unchecked_address: Address<NetworkUnchecked> = address.parse().unwrap();

    unchecked_address
        .require_network(network)
        .expect("Error decoding address for given network")
}
