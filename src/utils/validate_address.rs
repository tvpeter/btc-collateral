use bitcoin::{address::NetworkUnchecked, Address, Network};

pub fn validate_address(address: &str, network: Network) -> Result<Address, String> {
	let given_address: Result<Address<NetworkUnchecked>, _> = address.parse();

	let unchecked_address = match given_address {
		Ok(address) => address,
		Err(error) => {
			return Err(format!(
				"Error parsing address: {:?} => {:?}",
				address, error
			))
		}
	};

	let address = unchecked_address.require_network(network);

	match address {
		Ok(address) => Ok(address),
		Err(error) => Err(format!("Error parsing given address: {:?}", error)),
	}
}

#[cfg(test)]
mod test {
	use crate::constants::set_network;

	use super::*;

	#[test]
	fn test_validate_address() {
		let network = set_network();
		let valid_address = "bcrt1q20ey5k4xwrmryq6r3apw26yq2dy97spehr5cxt";
		let invalid_address = "bcrt1q20ey5k4xwrmryq6r3apw26yq2dy97spehr5cxte";

		assert!(validate_address(valid_address, network).is_ok());
		assert!(validate_address(invalid_address, network).is_err());
	}
}
