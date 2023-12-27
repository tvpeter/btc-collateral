use bitcoin::PublicKey;

pub fn is_valid_pubkey(pubkey_bytes: &[u8]) -> bool {
	// Attempt to parse the public key
	match PublicKey::from_slice(pubkey_bytes) {
		Ok(_pubkey) => true,
		Err(_) => false,
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_is_valid_pubkey() {
		let valid_pubkey = PublicKey::from_str(
			"02ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b",
		)
		.expect("invalid pubkey");

		let invalid_pubkey =
			"02ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b111".to_string();

		assert!(is_valid_pubkey(&valid_pubkey.to_bytes()));

		assert!(!is_valid_pubkey(&invalid_pubkey.into_bytes()));
	}
}
