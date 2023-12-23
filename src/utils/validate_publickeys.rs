use bitcoin::PublicKey;

pub fn is_valid_pubkey(pubkey_bytes: &[u8]) -> bool {
    // Attempt to parse the public key
    match PublicKey::from_slice(pubkey_bytes) {
        Ok(_pubkey) => {
            true
        }
        Err(_) => false,
    }
}

